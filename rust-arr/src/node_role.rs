//! Validated deployment roles. Subscriber initialization never runs on a coordinator.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeRole {
    Coordinator,
    Storage,
    Metadata,
}

impl NodeRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Coordinator => "coordinator",
            Self::Storage => "storage",
            Self::Metadata => "metadata",
        }
    }

    pub fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "coordinator" => Ok(Self::Coordinator),
            "storage" => Ok(Self::Storage),
            "metadata" => Ok(Self::Metadata),
            _ => Err("RAWARR_ROLE must be coordinator, storage, or metadata"),
        }
    }
    pub fn from_env() -> Result<Self, &'static str> {
        Self::parse(&std::env::var("RAWARR_ROLE").map_err(|_| "Set RAWARR_ROLE explicitly")?)
    }
    pub fn validate_media(self, configured: bool) -> Result<(), &'static str> {
        match (self, configured) {
            (Self::Storage, false) => {
                Err("storage requires RAWARR_LIBRARY_PATH and a read-only media mount")
            }
            (Self::Coordinator | Self::Metadata, true) => {
                Err("only storage may configure RAWARR_LIBRARY_PATH")
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_node_roles.rs"]
mod tests;
