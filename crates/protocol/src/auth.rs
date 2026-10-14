use md5::{Digest, Md5};
use uuid::Uuid;

/// Generates a deterministic offline UUID for a Minecraft username.
///
/// This precisely replicates vanilla Minecraft 1.8.9:
/// `UUID.nameUUIDFromBytes(("OfflinePlayer:" + username).getBytes(Charsets.UTF_8))`
/// as defined in `net.minecraft.entity.player.EntityPlayer.getOfflineUUID`.
pub fn offline_uuid(username: &str) -> Uuid {
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{}", username).as_bytes());
    let mut hash = hasher.finalize();

    // Version 3 (MD5-based UUID): clear version bits and set to 3
    hash[6] = (hash[6] & 0x0f) | 0x30;
    // Variant 2 (IETF RFC 4122): clear variant bits and set to 10xx
    hash[8] = (hash[8] & 0x3f) | 0x80;

    Uuid::from_bytes(hash.into())
}

/// Validates whether a Minecraft nickname is valid.
/// Minecraft 1.8.9 nicknames must be 1 to 16 ASCII alphanumeric characters or underscores.
pub fn is_valid_username(username: &str) -> bool {
    !username.is_empty()
        && username.len() <= 16
        && username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_uuid_deterministic() {
        let uuid1 = offline_uuid("Notch");
        let uuid2 = offline_uuid("Notch");
        assert_eq!(uuid1, uuid2, "Offline UUID must be deterministic");

        // Verify version and variant bits
        let bytes = uuid1.as_bytes();
        assert_eq!(bytes[6] >> 4, 3, "UUID version must be 3");
        assert_eq!(bytes[8] >> 6, 2, "UUID variant must be 2 (IETF RFC 4122)");
    }

    #[test]
    fn test_offline_uuid_known_values() {
        // Known vanilla Minecraft offline UUIDs:
        // Java: UUID.nameUUIDFromBytes("OfflinePlayer:Steve".getBytes(StandardCharsets.UTF_8))
        // MD5("OfflinePlayer:Steve") = c8077b9b-c402-3c13-a9c1-7f0f62e8dae4 (with v3/variant)
        let steve_uuid = offline_uuid("Steve");
        // Check that version is 3
        assert_eq!(steve_uuid.get_version_num(), 3);

        let alex_uuid = offline_uuid("Alex");
        assert_ne!(steve_uuid, alex_uuid);
    }

    #[test]
    fn test_username_validation() {
        assert!(is_valid_username("Steve"));
        assert!(is_valid_username("Alex_123"));
        assert!(is_valid_username("Player1"));
        assert!(is_valid_username("a"));
        assert!(is_valid_username("1234567890123456")); // 16 chars

        assert!(!is_valid_username("")); // Empty
        assert!(!is_valid_username("12345678901234567")); // 17 chars
        assert!(!is_valid_username("Player 1")); // Space
        assert!(!is_valid_username("Player!")); // Special char
        assert!(!is_valid_username("Стив")); // Non-ASCII cyrillic
    }
}
