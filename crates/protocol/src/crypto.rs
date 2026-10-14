use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128;

/// Minecraft 1.8.9 symmetric stream cipher (AES-128/CFB8/NoPadding).
///
/// Follows NIST SP 800-38A Section 6.3 and MCP `CryptManager.createNetCipherInstance`:
/// - Key: 128-bit (16-byte) shared secret
/// - IV: Identical 128-bit (16-byte) shared secret
/// - Mode: CFB with 8-bit (1 byte) feedback
///
/// In CFB8 mode, both encryptor and decryptor run the underlying AES block
/// cipher in ENCRYPT mode on the 16-byte shift register, XOR the byte with the first
/// byte of the keystream, and shift in the ciphertext byte.
pub struct Aes128Cfb8Cipher {
    cipher: Aes128,
    enc_state: [u8; 16],
    dec_state: [u8; 16],
}

impl Aes128Cfb8Cipher {
    /// Creates a new cipher instance initialized with a 16-byte shared secret key and IV.
    pub fn new(shared_secret: &[u8; 16]) -> Self {
        let key = GenericArray::from_slice(shared_secret);
        let cipher = Aes128::new(key);
        Self {
            cipher,
            enc_state: *shared_secret,
            dec_state: *shared_secret,
        }
    }

    /// Encrypts bytes in-place.
    pub fn encrypt(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            let mut block = GenericArray::clone_from_slice(&self.enc_state);
            self.cipher.encrypt_block(&mut block);
            let ciphertext_byte = *byte ^ block[0];
            *byte = ciphertext_byte;

            self.enc_state.copy_within(1..16, 0);
            self.enc_state[15] = ciphertext_byte;
        }
    }

    /// Decrypts bytes in-place.
    pub fn decrypt(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            let mut block = GenericArray::clone_from_slice(&self.dec_state);
            self.cipher.encrypt_block(&mut block);
            let ciphertext_byte = *byte;
            let plaintext_byte = ciphertext_byte ^ block[0];
            *byte = plaintext_byte;

            self.dec_state.copy_within(1..16, 0);
            self.dec_state[15] = ciphertext_byte;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfb8_stream_roundtrip() {
        let key = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];
        let mut cipher1 = Aes128Cfb8Cipher::new(&key);
        let mut cipher2 = Aes128Cfb8Cipher::new(&key);

        let stream1 = b"Minecraft 1.8.9 Protocol 47";
        let stream2 = b"Streaming packets through AES-128 CFB8";

        let mut enc1 = stream1.to_vec();
        cipher1.encrypt(&mut enc1);
        assert_ne!(enc1, stream1);

        let mut enc2 = stream2.to_vec();
        cipher1.encrypt(&mut enc2);
        assert_ne!(enc2, stream2);

        let mut dec1 = enc1.clone();
        cipher2.decrypt(&mut dec1);
        assert_eq!(dec1, stream1);

        let mut dec2 = enc2.clone();
        cipher2.decrypt(&mut dec2);
        assert_eq!(dec2, stream2);
    }
}
