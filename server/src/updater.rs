use anyhow::anyhow;

use drogue_ajour_protocol::{Command, Status};

use crate::index::{FirmwareSpec, FirmwareStatus, Index};
use crate::oci::OciClient;

pub struct Updater {
    index: Index,
    oci: OciClient,
}

impl Updater {
    pub fn new(index: Index, oci: OciClient) -> Self {
        Self { oci, index }
    }
    pub async fn process(
        &mut self,
        application: &str,
        device: &str,
        status: Status,
    ) -> Result<Command, anyhow::Error> {
        if let Some(spec) = self.index.latest_version(application, device).await? {
            match spec {
                FirmwareSpec::OCI {
                    image,
                    image_pull_policy,
                } => match self.oci.fetch_metadata(&image, image_pull_policy).await {
                    Ok(metadata) => {
                        // Update firmware status
                        let firmware_status = FirmwareStatus::new(&status, &metadata);
                        if let Err(e) = self
                            .index
                            .update_state(application, device, firmware_status)
                            .await
                        {
                            log::warn!(
                                "Error updating status of device {}/{}: {:?}",
                                application,
                                device,
                                e
                            );
                        }

                        log::debug!("Got metadata: {:?}", metadata);

                        if status.version == metadata.version {
                            Ok(Command::new_sync(&status.version, None))
                        } else {
                            let mut offset = 0;
                            let mut mtu = 512;
                            if let Some(m) = status.mtu {
                                mtu = m as usize;
                            }
                            if let Some(update) = status.update {
                                if update.version == metadata.version {
                                    offset = update.offset as usize;
                                }
                            }

                            if offset < metadata.size as usize {
                                let firmware = self
                                    .oci
                                    .fetch_firmware(&image, &metadata, image_pull_policy)
                                    .await?;

                                let to_copy = core::cmp::min(firmware.len() - offset, mtu);
                                let block = &firmware[offset..offset + to_copy];

                                log::trace!(
                                    "Sending firmware block offset {} size {}",
                                    offset,
                                    block.len()
                                );
                                Ok(Command::new_write(&metadata.version, offset as u32, block))
                            } else {
                                let data = hex::decode(&metadata.checksum)?;
                                Ok(Command::new_swap(&metadata.version, &data))
                            }
                        }
                    }
                    Err(e) => {
                        let firmware_status = FirmwareStatus::error(&status, e.to_string());
                        if let Err(e) = self
                            .index
                            .update_state(application, device, firmware_status)
                            .await
                        {
                            log::warn!(
                                "Error updating status of device {}/{}: {:?}",
                                application,
                                device,
                                e
                            );
                        }
                        Err(e.into())
                    }
                },
                FirmwareSpec::HAWKBIT => {
                    todo!("hawkbit firmware spec no yet supported")
                }
            }
        } else {
            Err(anyhow!("Unable to find latest version for {}", application))
        }
    }
}
