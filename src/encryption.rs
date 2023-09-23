use aes_gcm::{
    aead::{consts::U12, Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};

pub const NONCE_SIZE: usize = 12usize;

pub struct EncryptedMessage {
    nonce: Nonce<U12>,
    ciphertext: Vec<u8>,
}

impl EncryptedMessage {
    fn new(nonce: Nonce<U12>, ciphertext: Vec<u8>) -> Self {
        Self { nonce, ciphertext }
    }

    pub fn ciphertext(&self) -> &[u8] {
        self.ciphertext.as_ref()
    }

    pub fn nonce(&self) -> Nonce<U12> {
        self.nonce
    }
}

#[derive(Debug)]
pub struct EncryptError;

impl From<aes_gcm::Error> for EncryptError {
    fn from(_: aes_gcm::Error) -> Self {
        return Self;
    }
}

pub type DecryptedMessage = Vec<u8>;

#[derive(Debug)]
pub struct DecryptError;

impl From<aes_gcm::Error> for DecryptError {
    fn from(_: aes_gcm::Error) -> Self {
        return Self;
    }
}

pub struct CipherKey([u8; 32]); 

impl CipherKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        return Self(bytes);
    }
}

impl From<&[u8]> for CipherKey {
    fn from(value: &[u8]) -> Self {
        return Self::new(hmac_sha256::Hash::hash(value));
    }
}

impl AsRef<[u8; 32]> for CipherKey {
    fn as_ref(&self) -> &[u8; 32] {
        return &self.0;
    }
}

pub struct Cipher {
    aes: Aes256Gcm,
}

impl Cipher {
    pub fn new(key: &CipherKey) -> Self {
        let key: &[u8; 32] = key.as_ref();
        return Self {
            aes: Aes256Gcm::new(key.into()),
        };
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedMessage, EncryptError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let ciphertext = self.aes.encrypt(&nonce, plaintext.as_ref())?;

        return Ok(EncryptedMessage::new(nonce, ciphertext));
    }

    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &Nonce<U12>,
    ) -> Result<DecryptedMessage, DecryptError> {
        let plaintext = self.aes.decrypt(nonce, ciphertext.as_ref())?;

        return Ok(plaintext);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_decrypt() {
        let secret = b"my super secret key";
        let og_plaintext = b"plaintext message";
        let cipher = Cipher::new(&secret.as_slice().into());
        let msg = cipher.encrypt(og_plaintext).expect("encrypt error");
        let plaintext = cipher
            .decrypt(&msg.ciphertext, &msg.nonce)
            .expect("decrypt error");

        assert_eq!(&plaintext, og_plaintext);
    }

    #[test]
    fn different_nonce_cant_decrypt() {
        let secret = b"my super secret key";
        let og_plaintext = b"plaintext message";
        let cipher = Cipher::new(&secret.as_slice().into());
        let msg = cipher.encrypt(og_plaintext).expect("encrypt error");

        let wrong_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        cipher.decrypt(&msg.ciphertext, &wrong_nonce).unwrap_err();
    }

    #[test]
    fn different_key_cant_decrypt() {
        let secret = b"my super secret key";
        let og_plaintext = b"plaintext message";
        let cipher = Cipher::new(&secret.as_slice().into());
        let msg = cipher.encrypt(og_plaintext).expect("encrypt error");

        let wrong_secret = b"1234";
        let wrong_cipher = Cipher::new(&wrong_secret.as_slice().into());
        wrong_cipher
            .decrypt(&msg.ciphertext, &msg.nonce)
            .unwrap_err();
    }
}
