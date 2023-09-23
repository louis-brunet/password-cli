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

// TODO: I think this is unused
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
    // pub fn new(key_input: &CipherKeyInput) -> Self {
    // pub fn new(key_input: &[&[u8]]) -> Self {
    pub fn new(key: &CipherKey) -> Self {
    // pub fn new(key: &impl AsRef<[u8; 32]>) -> Self {
        // todo!("finish the refactor of this signature (undo changes ?)");
        // assert!(!key_input.is_empty());
        //
        // let mut key_hasher = hmac_sha256::Hash::new();
        // for key in key_input {
        //     key_hasher.update(key);
        // }
        // let key_bytes = key_hasher.finalize();

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

// pub fn test() {
//     // // The encryption key can be generated randomly:
//     // let key = Aes256Gcm::generate_key(OsRng);
//     //
//     // Transformed from a byte array:
//     // let key: &[u8; 32] = &[42; 32];
//     // let key: &Key<Aes256Gcm> = key.into();
//     //
//     let hashed = hmac_sha256::Hash::hash(&[123u8]);
//     println!(
//         "testing hash: {:?} --> (len = {}) {:?}",
//         123,
//         hashed.len(),
//         hashed
//     );
//
//     test_cipher(b"plaintext message");
//     test_cipher(b"aaaaaaaaaaaaaaaaa");
//     test_cipher(b"aaaaaaaaaaaaaaaaaaaaaaaaaaa");
//     test_cipher(b"plaintext messageplaintext messageplaintext messageplaintext messageplaintext messageplaintext messageplaintext message");
//
//     // // Note that you can get byte array from slice using the `TryInto` trait:
//     // let key: &[u8] = &[42; 32];
//     // let key: [u8; 32] = key.try_into().unwrap();
//     //
//     // // Alternatively, the key can be transformed directly from a byte slice
//     // // (panicks on length mismatch):
//     // let key = Key::<Aes256Gcm>::from_slice(&key);
//
//     // let cipher = Aes256Gcm::new(key);
//     // let msg = encrypt(og_plaintext, &cipher).expect("encrypt error");
//     // let plaintext = decrypt(msg.ciphertext.as_slice(), &cipher, &msg.nonce).expect("decrypt error");
//     //
//     // let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
//     // let ciphertext = cipher.encrypt(&nonce, og_plaintext.as_ref()).unwrap();
//     // let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();
// }
//
// fn test_cipher(og_plaintext: &[u8]) {
//     let secret = b"my super secret key";
//     let cipher = Cipher::new(&secret.as_slice().into());
//     let msg = cipher.encrypt(og_plaintext).expect("encrypt error");
//     let plaintext = cipher
//         .decrypt(&msg.ciphertext, &msg.nonce)
//         .expect("decrypt error");
//
//     assert_eq!(&plaintext, og_plaintext);
//
//     println!(
//         "\ntesting cipher: (len = {} ){:?}\nwith nonce {:?}\n --> (length={}) {:?}\n --> {:?}",
//         og_plaintext.len(),
//         og_plaintext,
//         msg.nonce,
//         msg.ciphertext.len(),
//         msg.ciphertext,
//         String::from_utf8(plaintext).unwrap(),
//     );
// }

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
