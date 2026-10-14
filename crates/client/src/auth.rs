use anyhow::{bail, Result};
use protocol::{is_valid_username, offline_uuid};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Offline player profile containing validated username and computed offline UUID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineProfile {
    pub username: String,
    pub uuid: Uuid,
}

impl OfflineProfile {
    /// Creates a new offline profile, validating the nickname and computing the vanilla 1.8.9 offline UUID.
    pub fn new(username: &str) -> Result<Self> {
        let trimmed = username.trim();
        if !is_valid_username(trimmed) {
            bail!(
                "Invalid username '{}'. Must be 1-16 ASCII alphanumeric characters or underscores.",
                trimmed
            );
        }

        let uuid = offline_uuid(trimmed);
        Ok(Self {
            username: trimmed.to_string(),
            uuid,
        })
    }
}

impl Default for OfflineProfile {
    fn default() -> Self {
        Self::new("Steve").expect("Default username 'Steve' must be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_profile_creation() {
        let profile = OfflineProfile::new("Player_1").expect("Should be valid");
        assert_eq!(profile.username, "Player_1");
        assert_eq!(profile.uuid.get_version_num(), 3);
    }

    #[test]
    fn test_invalid_profile_creation() {
        assert!(OfflineProfile::new("").is_err());
        assert!(OfflineProfile::new("TooLongUsernameForClient123").is_err());
        assert!(OfflineProfile::new("Invalid Nick!").is_err());
    }

    #[test]
    fn test_default_profile() {
        let default_profile = OfflineProfile::default();
        assert_eq!(default_profile.username, "Steve");
    }
}
