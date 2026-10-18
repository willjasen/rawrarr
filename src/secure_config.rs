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

    pub fn from_identity(identity: String) -> Result<Self, VaultError> {
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
            config_path: Self::config_path(),
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
