/// Credential vault — passwords are encrypted with AES-256-GCM (argon2id key)
/// and stored directly in the SQLite `password` column (`__AES__:base64`).
///
/// The OS keychain is attempted as a **secondary** best-effort store, but is
/// never relied upon as the primary path because it falls back to a non-persistent
/// `MockCredential` in unsigned debug builds.

use crate::crypto;

const KEYCHAIN_SERVICE: &str = "terminalz";

/// Prefix constants for the `password` column in SQLite.
const PREFIX_AES: &str = "__AES__";     // "__AES__:base64_blob" → primary, always works
const PREFIX_KEYCHAIN: &str = "__KC__"; // "__KC__:conn_id"     → legacy / secondary

pub struct Vault {
    encryption_key: [u8; 32],
}

impl Vault {
    pub fn new(app_data_dir: &str) -> Self {
        Self {
            encryption_key: crypto::derive_key(app_data_dir),
        }
    }

    // ----------------------------------------------------------------
    //  Store
    // ----------------------------------------------------------------

    /// Encrypt a password and return the SQLite column value.
    ///
    /// **Always** stores as `__AES__:base64` — the AES path is the only one
    /// that works reliably across dev / release / all platforms.
    /// The keychain is also updated as a best-effort secondary copy.
    pub fn store(&self, connection_id: i64, password: &str) -> Result<String, String> {
        let encrypted = crypto::encrypt(password, &self.encryption_key)?;
        // Keychain is nice-to-have, never a hard requirement.
        let _ = try_keychain_store(connection_id, &encrypted);
        Ok(format!("{}:{}", PREFIX_AES, encrypted))
    }

    /// Encrypt only (no keychain / no prefix) — used during save for new
    /// connections that don't have an id yet.
    pub fn encrypt_aes(&self, password: &str) -> Result<String, String> {
        crypto::encrypt(password, &self.encryption_key)
    }

    // ----------------------------------------------------------------
    //  Load
    // ----------------------------------------------------------------

    /// Retrieve and decrypt a password.
    ///
    /// Handles both `__AES__:base64` (primary) and `__KC__:id` (legacy) entries.
    pub fn load(&self, connection_id: i64, db_value: &str) -> Result<String, String> {
        let encrypted = if let Some(rest) = db_value.strip_prefix(&format!("{}:", PREFIX_AES)) {
            rest.to_string()
        } else if db_value.starts_with(&format!("{}:", PREFIX_KEYCHAIN)) {
            // Legacy keychain-only reference — try to load, then migrate to AES.
            let enc = try_keychain_load(connection_id)?;
            // The caller should re-save this as AES.
            enc
        } else {
            return Err("unknown vault prefix in password field".into());
        };

        crypto::decrypt(&encrypted, &self.encryption_key)
    }

    // ----------------------------------------------------------------
    //  Delete
    // ----------------------------------------------------------------

    /// Remove a stored password from the OS keychain (best-effort).
    pub fn delete(&self, connection_id: i64, db_value: &str) {
        if db_value.starts_with(PREFIX_KEYCHAIN) {
            let _ = keyring::Entry::new(KEYCHAIN_SERVICE, &format!("conn_{}", connection_id))
                .and_then(|e| e.delete_credential());
        }
    }
}

// ----------------------------------------------------------------
//  OS keychain helpers (private)
// ----------------------------------------------------------------

fn try_keychain_store(connection_id: i64, encrypted: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &format!("conn_{}", connection_id))
        .map_err(|e| format!("keyring init: {}", e))?;
    entry
        .set_password(encrypted)
        .map_err(|e| format!("keyring store: {}", e))
}

fn try_keychain_load(connection_id: i64) -> Result<String, String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &format!("conn_{}", connection_id))
        .map_err(|e| format!("keyring init: {}", e))?;
    entry.get_password().map_err(|e| {
        let msg = e.to_string();
        if msg.contains("No matching entry") || msg.contains("not found") {
            format!(
                "Keychain entry missing for connection {} — \
                 the password was stored in your OS keychain but the entry no longer exists.\n\
                 This can happen after a keychain reset, macOS upgrade, or moving to a new machine.\n\
                 Fix: right-click the host → Edit → enter the password → Save.",
                connection_id,
            )
        } else {
            format!("keyring load: {}", msg)
        }
    })
}
