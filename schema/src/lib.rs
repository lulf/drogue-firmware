use drogue_client::{core::v1::Conditions, dialect, Section};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ImagePullPolicy {
    Always,
    IfNotPresent,
}

impl Default for ImagePullPolicy {
    fn default() -> Self {
        Self::IfNotPresent
    }
}

dialect!(FirmwareSpec [Section::Spec => "firmware"]);

#[derive(Serialize, Deserialize, Debug)]
pub enum FirmwareSpec {
    #[serde(rename = "oci")]
    OCI {
        image: String,
        #[serde(rename = "imagePullPolicy", default = "Default::default")]
        image_pull_policy: ImagePullPolicy,
        build: Option<FirmwareBuildSpec>,
    },
    #[serde(rename = "hawkbit")]
    HAWKBIT { controller: String },
    #[serde(rename = "file")]
    FILE { name: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareBuildSpec {
    image: String,
    source: FirmwareBuildSource,
    env: Vec<FirmwareBuildEnv>,
    args: Vec<String>,
    artifact: FirmwareBuildArtifact,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareBuildEnv {
    name: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FirmwareBuildSource {
    #[serde(rename = "git")]
    GIT { uri: String, project: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareBuildArtifact {
    path: String,
}

dialect!(FirmwareStatus [Section::Status => "firmware"]);

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FirmwareStatus {
    pub conditions: Conditions,
    pub current: String,
    pub target: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}