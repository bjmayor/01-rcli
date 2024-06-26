use std::{fs, io::Read, path::Path};

use crate::{get_reader, process_genpass, TextSignFormat};
use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use chacha20poly1305::aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit};

pub trait TextSign {
    /// Sign the data from the reader and return the signature
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    fn verify(&self, reader: impl Read, signature: &[u8]) -> Result<bool>;
}

pub trait TextEncryptor {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextDecryptor {
    fn decrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait KeyLoader {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
}
pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

pub struct ChaCha20Poly1305 {
    key: [u8; 32],
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let signature = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
    };
    let signature = URL_SAFE_NO_PAD.encode(signature);
    Ok(signature)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    signature: &str,
) -> anyhow::Result<bool> {
    let mut reader = get_reader(input)?;
    let signature = URL_SAFE_NO_PAD.decode(signature)?;
    let verified = match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key)?;
            verifier.verify(&mut reader, &signature)?
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &signature)?
        }
    };
    Ok(verified)
}

pub fn process_generate_key(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

pub fn process_text_encrypt(input: &str, key: &str) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let encryptor = ChaCha20Poly1305::load(key)?;
    let encrypted = encryptor.encrypt(&mut reader)?;
    let encrypted = URL_SAFE_NO_PAD.encode(encrypted);
    Ok(encrypted)
}

pub fn process_text_decrypt(input: &str, key: &str) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let encrypted = URL_SAFE_NO_PAD.decode(buf)?;
    let decryptor = ChaCha20Poly1305::load(key)?;
    let decrypted = decryptor.decrypt(&mut &encrypted[..])?;
    let decrypted = String::from_utf8(decrypted)?;
    Ok(decrypted)
}

impl ChaCha20Poly1305 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[0..32];
        let key = key.try_into().unwrap();
        let signer = ChaCha20Poly1305::new(key);
        Ok(signer)
    }
}

impl KeyLoader for ChaCha20Poly1305 {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl TextEncryptor for ChaCha20Poly1305 {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let cipher = chacha20poly1305::ChaCha20Poly1305::new(&self.key.into());
        let nonce = chacha20poly1305::ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let encrypted = cipher
            .encrypt(&nonce, buf.as_ref())
            .map_err(|e| anyhow::anyhow!("Error encrypting data: {}", e))?;
        let mut buf = Vec::new();
        buf.extend_from_slice(&nonce);
        buf.extend_from_slice(&encrypted);
        Ok(buf)
    }
}

impl TextDecryptor for ChaCha20Poly1305 {
    fn decrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let cipher = chacha20poly1305::ChaCha20Poly1305::new(&self.key.into());
        if buf.len() < 12 {
            return Err(anyhow::anyhow!("Invalid data"));
        }
        let nonce = &buf[0..12];
        let encrypted = &buf[12..];
        let decrypted = cipher
            .decrypt(GenericArray::from_slice(nonce), encrypted)
            .map_err(|e| anyhow::anyhow!("Error decrypting data: {}", e))?;
        Ok(decrypted)
    }
}
impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, mut reader: impl Read, signature: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let buf = blake3::keyed_hash(&self.key, &buf);
        let hash = buf.as_bytes();
        Ok(hash == signature)
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[0..32];
        let key = key.try_into().unwrap();
        let signer = Blake3::new(key);
        Ok(signer)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = self.key.sign(&buf);
        Ok(sig.to_bytes().to_vec())
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = Signature::from_bytes(sig.try_into()?);
        let ret = self.key.verify(&buf, &sig).is_ok();
        Ok(ret)
    }
}

impl Ed25519Signer {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        Ok(Ed25519Signer::new(key))
    }
}

impl Ed25519Verifier {
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?)?;
        Ok(Ed25519Verifier::new(key))
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key();
        let sk = sk.to_bytes().to_vec();
        let pk = pk.to_bytes().to_vec();
        Ok(vec![sk, pk])
    }
}
impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_sign_verify() -> Result<()> {
        let blake3 = Blake3::load("fixtures/blake3.txt")?;
        let data = b"Hello, World!";
        let sig = blake3.sign(&mut &data[..])?;
        assert!(blake3.verify(&mut &data[..], &sig)?);

        Ok(())
    }
    #[test]
    fn test_ed25519_sign_verify() -> Result<()> {
        let signer = Ed25519Signer::load("fixtures/ed25519.sk")?;
        let verifier = Ed25519Verifier::load("fixtures/ed25519.pk")?;
        let data = b"Hello, World!";
        let sig = signer.sign(&mut &data[..])?;
        assert!(verifier.verify(&mut &data[..], &sig)?);

        Ok(())
    }

    #[test]
    fn test_chacha20poly1305_encrypt_decrypt() -> Result<()> {
        let key = ChaCha20Poly1305::load("fixtures/chacha20poly1305.txt")?;
        let data = b"Hello, World!";
        let encrypted = key.encrypt(&mut &data[..])?;
        let decrypted = key.decrypt(&mut &encrypted[..])?;
        assert_eq!(data, decrypted.as_slice());
        Ok(())
    }
}
