use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::RngCore;

/// Taille de la clé AES-256 (32 bytes)
const KEY_SIZE: usize = 32;

/// Taille du nonce (12 bytes pour GCM)
const NONCE_SIZE: usize = 12;

/// Dérive une clé de chiffrement depuis un mot de passe maître
/// Utilise Argon2id (recommandé OWASP 2024)
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_SIZE]> {
    let argon2 = Argon2::default();
    let salt_string =
        SaltString::encode_b64(salt).map_err(|e| anyhow::anyhow!("Erreur encodage salt: {}", e))?;

    let hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| anyhow::anyhow!("Erreur dérivation de clé: {}", e))?;

    let hash_bytes = hash.hash.context("Hash manquant")?;
    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&hash_bytes.as_bytes()[..KEY_SIZE]);

    Ok(key)
}

/// Génère un salt aléatoire cryptographiquement sûr
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Chiffre des données avec AES-256-GCM
pub fn encrypt(data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());

    // Génère un nonce aléatoire
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Chiffre les données
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| anyhow::anyhow!("Erreur de chiffrement: {}", e))?;

    // Format: [nonce || ciphertext]
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Déchiffre des données avec AES-256-GCM
pub fn decrypt(encrypted_data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
    if encrypted_data.len() < NONCE_SIZE {
        anyhow::bail!("Données chiffrées invalides (trop courtes)");
    }

    let cipher = Aes256Gcm::new(key.into());

    // Extrait le nonce
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Déchiffre
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Erreur de déchiffrement: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = b"Hello, World!";
        let salt = generate_salt();
        let key = derive_key("test_password", &salt).unwrap();

        let encrypted = encrypt(data, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }
}
