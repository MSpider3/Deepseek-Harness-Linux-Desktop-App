use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{bail, Context, Result};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::HashMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize, Default)]
struct EncryptedVault {
    salt: String,
    nonce: String,
    ciphertext: String,
}

#[derive(Clone)]
pub struct SecretStore {
    vault_file: PathBuf,
    secrets: Arc<RwLock<HashMap<String, String>>>,
    machine_key: [u8; 32],
}

impl SecretStore {
    pub fn init<P: AsRef<Path>>(base_data_dir: P) -> Result<Self> {
        let vault_file = base_data_dir.as_ref().join("vault.enc");
        let machine_key = Self::derive_machine_key();

        let mut store = Self {
            vault_file,
            secrets: Arc::new(RwLock::new(HashMap::new())),
            machine_key,
        };

        if store.vault_file.exists() {
            store.load()?;
        }

        Ok(store)
    }

    pub fn in_memory() -> Self {
        Self {
            vault_file: PathBuf::from("/dev/null"),
            secrets: Arc::new(RwLock::new(HashMap::new())),
            machine_key: [42u8; 32],
        }
    }

    fn derive_machine_key() -> [u8; 32] {
        let machine_id = fs::read_to_string("/etc/machine-id")
            .unwrap_or_else(|_| "default-deepseek-harness-linux-seed-key".to_string());
        let user = std::env::var("USER").unwrap_or_else(|_| "dsh-user".to_string());
        let salt = b"deepseek-harness-linux-secret-service-salt-2026";
        
        let mut key = [0u8; 32];
        let password = format!("{}:{}", machine_id.trim(), user);
        pbkdf2_hmac::<Sha1>(password.as_bytes(), salt, 10_000, &mut key);
        key
    }

    fn load(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.vault_file)
            .with_context(|| format!("Failed to read secret vault {:?}", self.vault_file))?;

        let vault: EncryptedVault = serde_json::from_str(&content)
            .context("Failed to parse encrypted secret vault")?;

        let nonce_bytes = hex::decode(&vault.nonce).context("Invalid nonce hex")?;
        let ciphertext = hex::decode(&vault.ciphertext).context("Invalid ciphertext hex")?;

        let cipher = Aes256Gcm::new_from_slice(&self.machine_key)
            .map_err(|e| anyhow::anyhow!("Cipher init error: {}", e))?;
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        let decrypted = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("Failed to decrypt secret vault. Authentication failed."))?;

        let map: HashMap<String, String> = serde_json::from_slice(&decrypted)
            .context("Failed to deserialize secrets map")?;

        *self.secrets.write().unwrap() = map;
        Ok(())
    }

    fn save(&self) -> Result<()> {
        if self.vault_file == Path::new("/dev/null") {
            return Ok(());
        }

        let map = self.secrets.read().unwrap();
        let serialized = serde_json::to_vec(&*map)?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&self.machine_key)
            .map_err(|e| anyhow::anyhow!("Cipher init error: {}", e))?;

        let ciphertext = cipher
            .encrypt(nonce, serialized.as_ref())
            .map_err(|e| anyhow::anyhow!("Vault encryption failed: {}", e))?;

        let vault = EncryptedVault {
            salt: "sha1_pbkdf2_v1".to_string(),
            nonce: hex::encode(nonce_bytes),
            ciphertext: hex::encode(ciphertext),
        };

        if let Some(parent) = self.vault_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&vault)?;
        fs::write(&self.vault_file, json)?;

        #[cfg(unix)]
        {
            let permissions = fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&self.vault_file, permissions);
        }

        Ok(())
    }

    pub fn set_secret(&self, key: &str, secret: &str) -> Result<()> {
        if key.trim().is_empty() {
            bail!("Secret key cannot be empty");
        }
        {
            let mut map = self.secrets.write().unwrap();
            map.insert(key.to_string(), secret.to_string());
        }
        self.save()
    }

    pub fn get_secret(&self, key: &str) -> Result<Option<String>> {
        let map = self.secrets.read().unwrap();
        Ok(map.get(key).cloned())
    }

    pub fn delete_secret(&self, key: &str) -> Result<bool> {
        let removed = {
            let mut map = self.secrets.write().unwrap();
            map.remove(key).is_some()
        };
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn has_secret(&self, key: &str) -> bool {
        let map = self.secrets.read().unwrap();
        map.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_store_in_memory() {
        let store = SecretStore::in_memory();
        store.set_secret("openai_key", "sk-test-12345").unwrap();
        assert_eq!(store.get_secret("openai_key").unwrap().as_deref(), Some("sk-test-12345"));
        assert!(store.has_secret("openai_key"));
        assert!(!store.has_secret("missing"));
        assert!(store.delete_secret("openai_key").unwrap());
        assert!(!store.has_secret("openai_key"));
    }
}
