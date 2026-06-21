/// AES-256-GCM encryption / decryption for the password vault.
///
/// Key derivation uses **argon2id** (memory-hard KDF, OWASP-recommended) so
/// that brute-forcing the encryption key from a stolen database file is
/// impractical even when the attacker knows the hostname and app-data path.

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand::RngCore;

const VAULT_SALT: &[u8] = b"terminalz-password-vault-v2";

// ----------------------------------------------------------------
//  Key Derivation (argon2id, OWASP minimum)
// ----------------------------------------------------------------
//
// | Param       | Value    | Notes                        |
// |-------------|----------|------------------------------|
// | Memory      | 19 MiB   | OWASP minimum for argon2id   |
// | Iterations  | 2        | OWASP minimum                |
// | Parallelism | 1        | single-threaded startup path |

const ARGON_M_COST: u32 = 19_456; // KiB
const ARGON_T_COST: u32 = 2;
const ARGON_P_COST: u32 = 1;

/// Derive a 32-byte AES key from machine-unique factors using argon2id.
///
/// Input:  `hostname || ":" || app_data_dir`
/// Salt:   `VAULT_SALT`
/// Output: 32-byte key for AES-256-GCM
pub fn derive_key(app_data_dir: &str) -> [u8; 32] {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();
    let password = format!("{}:{}", hostname, app_data_dir);

    let params = Params::new(ARGON_M_COST, ARGON_T_COST, ARGON_P_COST, Some(32))
        .expect("argon2 params should be valid");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), VAULT_SALT, &mut key)
        .expect("argon2 key derivation failed");
    key
}

// ----------------------------------------------------------------
//  AES-256-GCM encrypt / decrypt
// ----------------------------------------------------------------

/// Encrypt `plain` with the given 256-bit key.
/// Returns `base64(nonce || ciphertext)`.
pub fn encrypt(plain: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plain.as_bytes())
        .map_err(|e| format!("encryption failed: {}", e))?;
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ct);
    Ok(BASE64.encode(&result))
}

/// Decrypt `base64(nonce || ciphertext)` with the given 256-bit key.
pub fn decrypt(encoded: &str, key: &[u8; 32]) -> Result<String, String> {
    let data = BASE64
        .decode(encoded)
        .map_err(|e| format!("base64 decode failed: {}", e))?;
    if data.len() < 12 {
        return Err("ciphertext too short".into());
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&data[..12]);
    let plain = cipher
        .decrypt(nonce, &data[12..])
        .map_err(|e| format!("decryption failed: {}", e))?;
    String::from_utf8(plain).map_err(|e| format!("utf-8 decode failed: {}", e))
}
