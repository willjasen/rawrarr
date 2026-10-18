use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone)]
pub struct AgeVault {
    identity: String,
    recipient: String,
    data_dir: PathBuf,
    config_path: PathBuf,
}

impl Drop for AgeVault {
    fn drop(&mut self) {
        self.identity.zeroize();
        self.recipient.zeroize();
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PersistedConfig {
    pub radarr_url: String,
    pub radarr_api_key: String,
}

pub struct EncryptedArtwork {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

impl Drop for PersistedConfig {
    fn drop(&mut self) {
        self.radarr_url.zeroize();
        self.radarr_api_key.zeroize();
    }
}

#[derive(Debug)]
pub enum VaultError {
    InvalidIdentity,
    AgeUnavailable,
    EncryptionFailed,
    DecryptionFailed,
    StorageFailed,
    InvalidConfig,
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentity => "Invalid post-quantum age identity",
            Self::AgeUnavailable => "age CLI is unavailable",
            Self::EncryptionFailed => "Could not encrypt settings",
            Self::DecryptionFailed => "Could not decrypt settings",
            Self::StorageFailed => "Could not access encrypted settings storage",
            Self::InvalidConfig => "Encrypted settings have an invalid format",
        })
    }
}

impl std::error::Error for VaultError {}

impl AgeVault {
    pub fn data_dir() -> PathBuf {
        env::var_os("RAWARR_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data"))
    }

    pub fn config_path() -> PathBuf {
        Self::data_dir().join("config.age")
    }

    fn artwork_path(&self, tmdb_id: u64) -> PathBuf {
        self.data_dir
            .join("metadata")
            .join("movies")
            .join(format!("{tmdb_id}.aged"))
    }

    pub fn artwork_exists(&self, tmdb_id: u64) -> bool {
        self.artwork_path(tmdb_id).is_file()
    }

    pub fn store_artwork(
        &self,
        tmdb_id: u64,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), VaultError> {
        const MAX_ARTWORK_BYTES: usize = 8 * 1024 * 1024;
        if tmdb_id == 0
            || bytes.is_empty()
            || bytes.len() > MAX_ARTWORK_BYTES
            || !matches!(content_type, "image/jpeg" | "image/png" | "image/webp")
        {
            return Err(VaultError::InvalidConfig);
        }
        let path = self.artwork_path(tmdb_id);
        let directory = path.parent().ok_or(VaultError::StorageFailed)?;
        fs::create_dir_all(directory).map_err(|_| VaultError::StorageFailed)?;
        if let Some(metadata_directory) = directory.parent() {
            set_private_directory_permissions(metadata_directory)?;
        }
        set_private_directory_permissions(directory)?;
        let temporary = temporary_path(&path);
        let mut plaintext =
            Zeroizing::new(Vec::with_capacity(content_type.len() + 1 + bytes.len()));
        plaintext.extend_from_slice(content_type.as_bytes());
        plaintext.push(b'\n');
        plaintext.extend_from_slice(bytes);
        let result = Command::new("age")
            .args(["--recipient", &self.recipient, "--output"])
            .arg(&temporary)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| VaultError::AgeUnavailable)
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin
                        .write_all(plaintext.as_slice())
                        .map_err(|_| VaultError::EncryptionFailed)?;
                }
                child.wait().map_err(|_| VaultError::EncryptionFailed)
            });
        match result {
            Ok(status) if status.success() => {}
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(error);
            }
            _ => {
                let _ = fs::remove_file(&temporary);
                return Err(VaultError::EncryptionFailed);
            }
        }
        if let Err(error) = set_private_permissions(&temporary) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        fs::rename(&temporary, &path).map_err(|_| {
            let _ = fs::remove_file(&temporary);
            VaultError::StorageFailed
        })
    }

    pub fn load_artwork(&self, tmdb_id: u64) -> Result<Option<EncryptedArtwork>, VaultError> {
        const MAX_CIPHERTEXT_BYTES: u64 = 9 * 1024 * 1024;
        const MAX_ARTWORK_BYTES: usize = 8 * 1024 * 1024;
        let path = self.artwork_path(tmdb_id);
        let Ok(metadata) = fs::metadata(&path) else {
            return Ok(None);
        };
        if metadata.len() > MAX_CIPHERTEXT_BYTES {
            return Err(VaultError::InvalidConfig);
        }
        let output = run_with_input(
            "age",
            &["--decrypt", "--identity", "-", path_arg(&path)?],
            self.identity.as_bytes(),
        )?;
        if !output.status.success() {
            return Err(VaultError::DecryptionFailed);
        }
        let plaintext = Zeroizing::new(output.stdout);
        let Some(separator) = plaintext.iter().position(|byte| *byte == b'\n') else {
            return Err(VaultError::InvalidConfig);
        };
        let content_type =
            std::str::from_utf8(&plaintext[..separator]).map_err(|_| VaultError::InvalidConfig)?;
        let bytes = &plaintext[separator + 1..];
        if !matches!(content_type, "image/jpeg" | "image/png" | "image/webp")
            || bytes.is_empty()
            || bytes.len() > MAX_ARTWORK_BYTES
        {
            return Err(VaultError::InvalidConfig);
        }
        Ok(Some(EncryptedArtwork {
            content_type: content_type.to_owned(),
            bytes: bytes.to_vec(),
        }))
    }

    pub fn from_identity(identity: String) -> Result<Self, VaultError> {
        Self::from_identity_in_data_dir(identity, Self::data_dir())
    }

    fn from_identity_in_data_dir(identity: String, data_dir: PathBuf) -> Result<Self, VaultError> {
        if !identity
            .lines()
            .any(|line| line.trim().starts_with("AGE-SECRET-KEY-PQ-1"))
        {
            return Err(VaultError::InvalidIdentity);
        }
        let output = run_with_input("age-keygen", &["-y"], identity.as_bytes())?;
        if !output.status.success() {
            return Err(VaultError::InvalidIdentity);
        }
        let recipient = String::from_utf8(output.stdout)
            .map_err(|_| VaultError::InvalidIdentity)?
            .lines()
            .find(|line| line.starts_with("age1pq1"))
            .ok_or(VaultError::InvalidIdentity)?
            .to_owned();
        Ok(Self {
            identity,
            recipient,
            config_path: data_dir.join("config.age"),
            data_dir,
        })
    }

    pub fn generate_identity() -> Result<(String, String), VaultError> {
        let output = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .map_err(|_| VaultError::AgeUnavailable)?;
        if !output.status.success() {
            return Err(VaultError::InvalidIdentity);
        }
        let identity = String::from_utf8(output.stdout).map_err(|_| VaultError::InvalidIdentity)?;
        let vault = Self::from_identity(identity.clone())?;
        Ok((identity, vault.recipient.clone()))
    }

    pub fn config_exists(&self) -> bool {
        self.config_path.is_file()
    }

    pub fn ciphertext_format_recognized() -> bool {
        const AGE_CIPHERTEXT_MAGIC: &[u8] = b"age-encryption.org/v1\n";
        let Ok(mut file) = fs::File::open(Self::config_path()) else {
            return false;
        };
        let mut prefix = [0; AGE_CIPHERTEXT_MAGIC.len()];
        file.read_exact(&mut prefix).is_ok() && prefix == AGE_CIPHERTEXT_MAGIC
    }

    pub fn load(&self) -> Result<Option<PersistedConfig>, VaultError> {
        if !self.config_exists() {
            return Ok(None);
        }
        let output = run_with_input(
            "age",
            &["--decrypt", "--identity", "-", path_arg(&self.config_path)?],
            self.identity.as_bytes(),
        )?;
        if !output.status.success() {
            return Err(VaultError::DecryptionFailed);
        }
        let plaintext = Zeroizing::new(output.stdout);
        serde_json::from_slice(plaintext.as_slice())
            .map(Some)
            .map_err(|_| VaultError::InvalidConfig)
    }

    pub fn save(&self, config: &PersistedConfig) -> Result<(), VaultError> {
        let plaintext =
            Zeroizing::new(serde_json::to_vec(config).map_err(|_| VaultError::InvalidConfig)?);
        let directory = self.config_path.parent().ok_or(VaultError::StorageFailed)?;
        fs::create_dir_all(directory).map_err(|_| VaultError::StorageFailed)?;
        set_private_directory_permissions(directory)?;
        let temporary = temporary_path(&self.config_path);
        let output = Command::new("age")
            .args(["--recipient", &self.recipient, "--output"])
            .arg(&temporary)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| VaultError::AgeUnavailable)
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin
                        .write_all(plaintext.as_slice())
                        .map_err(|_| VaultError::EncryptionFailed)?;
                }
                child.wait().map_err(|_| VaultError::EncryptionFailed)
            });
        match output {
            Ok(status) if status.success() => {}
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(error);
            }
            _ => {
                let _ = fs::remove_file(&temporary);
                return Err(VaultError::EncryptionFailed);
            }
        }
        if let Err(error) = set_private_permissions(&temporary) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        if fs::File::open(&temporary)
            .and_then(|file| file.sync_all())
            .is_err()
        {
            let _ = fs::remove_file(&temporary);
            return Err(VaultError::StorageFailed);
        }
        fs::rename(&temporary, &self.config_path).map_err(|_| {
            let _ = fs::remove_file(&temporary);
            VaultError::StorageFailed
        })
    }

    pub fn clear(&self) -> Result<(), VaultError> {
        match fs::remove_file(&self.config_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(VaultError::StorageFailed),
        }
    }
}

fn run_with_input(
    program: &str,
    args: &[&str],
    input: &[u8],
) -> Result<std::process::Output, VaultError> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| VaultError::AgeUnavailable)?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input)
            .map_err(|_| VaultError::EncryptionFailed)?;
    }
    child
        .wait_with_output()
        .map_err(|_| VaultError::EncryptionFailed)
}

fn path_arg(path: &Path) -> Result<&str, VaultError> {
    path.to_str().ok_or(VaultError::StorageFailed)
}

fn temporary_path(path: &Path) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!(".{filename}.{}.{}.tmp", std::process::id(), nonce))
}

fn set_private_permissions(path: &Path) -> Result<(), VaultError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|_| VaultError::StorageFailed)?;
    }
    Ok(())
}

fn set_private_directory_permissions(path: &Path) -> Result<(), VaultError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| VaultError::StorageFailed)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artwork_is_stored_as_age_ciphertext_and_round_trips() {
        let keygen = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("age-keygen must be installed for this test");
        assert!(keygen.status.success());
        let identity = String::from_utf8(keygen.stdout).expect("identity is UTF-8");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let data_dir = env::temp_dir().join(format!(
            "rawrarr-artwork-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&data_dir).expect("create isolated test directory");
        let vault = AgeVault::from_identity_in_data_dir(identity, data_dir.clone())
            .expect("valid PQ age identity");
        let image = b"disposable test poster bytes";
        vault
            .store_artwork(12345, "image/jpeg", image)
            .expect("encrypt poster into .aged file");

        let encrypted_path = data_dir.join("metadata/movies/12345.aged");
        let encrypted = fs::read(&encrypted_path).expect("read encrypted poster");
        assert!(encrypted.starts_with(b"age-encryption.org/v1\n"));
        assert!(!encrypted.windows(image.len()).any(|window| window == image));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&encrypted_path)
                    .expect("poster metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
            assert_eq!(
                fs::metadata(data_dir.join("metadata"))
                    .expect("metadata directory")
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
        }
        let decrypted = vault
            .load_artwork(12345)
            .expect("decrypt poster")
            .expect("poster exists");
        assert_eq!(decrypted.content_type, "image/jpeg");
        assert_eq!(decrypted.bytes, image);
        fs::remove_dir_all(data_dir).expect("remove test data");
    }
}
