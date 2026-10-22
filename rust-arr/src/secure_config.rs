use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use zeroize::{Zeroize, Zeroizing};

#[path = "movie_metadata_cache.rs"]
mod movie_metadata_cache;

#[derive(Clone)]
pub struct AgeVault {
    movie_metadata_cache: std::sync::Arc<movie_metadata_cache::MetadataCache>,
    identity: String,
    recipient: String,
    shared_dir: PathBuf,
    metadata_dir: PathBuf,
    config_path: PathBuf,
    local_config_path: PathBuf,
    local_plaintext_config: bool,
    metadata_filename_key: std::sync::Arc<std::sync::OnceLock<Zeroizing<Vec<u8>>>>,
}

impl Drop for AgeVault {
    fn drop(&mut self) {
        self.identity.zeroize();
        self.recipient.zeroize();
    }
}

pub struct PersistedConfig {
    pub device_names: std::collections::HashMap<String, String>,
    pub sonarr: Option<ArrConnection>,
    pub radarr_url: String,
    pub radarr_api_key: String,
    pub additional_services: Vec<ServiceConfig>,
    pub(crate) needs_migration: bool,
    /// Reserved exclusively for opaque metadata filename derivation; never an encryption/auth key.
    pub metadata_filename_key: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    #[serde(rename = "type")]
    pub service_type: String,
    pub url: String,
    pub api_key: String,
}
impl Drop for ServiceConfig {
    fn drop(&mut self) { self.api_key.zeroize(); }
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct PersistedConfigWire {
    schema_version: Option<u32>,
    #[serde(default)]
    device_names: std::collections::HashMap<String, String>,
    services: Option<Vec<ServiceConfig>>,
    #[serde(default)]
    radarr_url: String,
    #[serde(default)]
    radarr_api_key: String,
    #[serde(default)]
    sonarr: Option<ArrConnection>,
    #[serde(default)]
    metadata_filename_key: Option<String>,
}

impl<'de> Deserialize<'de> for PersistedConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let wire = PersistedConfigWire::deserialize(deserializer)?;
        if wire.schema_version.is_some_and(|version| version > 1) {
            return Err(serde::de::Error::custom("unsupported shared config schema version"));
        }
        let canonical = wire.schema_version == Some(1)
            && wire.services.is_some()
            && wire.radarr_url.is_empty()
            && wire.radarr_api_key.is_empty()
            && wire.sonarr.is_none();
        let mut radarr_url = wire.radarr_url;
        let mut radarr_api_key = wire.radarr_api_key;
        let mut sonarr = wire.sonarr;
        let mut additional_services = Vec::new();
        if let Some(services) = wire.services {
            for service in services {
                match service.service_type.as_str() {
                    "radarr" => { radarr_url = service.url.clone(); radarr_api_key = service.api_key.clone(); }
                    "sonarr" => { sonarr = Some(ArrConnection { url: service.url.clone(), api_key: service.api_key.clone() }); }
                    _ => additional_services.push(service),
                }
            }
        }
        Ok(Self {
            device_names: wire.device_names,
            sonarr,
            radarr_url,
            radarr_api_key,
            additional_services,
            needs_migration: !canonical,
            metadata_filename_key: wire.metadata_filename_key,
        })
    }
}

impl Serialize for PersistedConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        use serde::ser::SerializeStruct;
        #[derive(Serialize)]
        struct ServiceRef<'a> {
            #[serde(rename = "type")]
            service_type: &'a str,
            url: &'a str,
            api_key: &'a str,
        }
        let mut services = Vec::new();
        if !self.radarr_url.is_empty() || !self.radarr_api_key.is_empty() {
            services.push(ServiceRef { service_type: "radarr", url: &self.radarr_url, api_key: &self.radarr_api_key });
        }
        if let Some(sonarr) = &self.sonarr {
            services.push(ServiceRef { service_type: "sonarr", url: &sonarr.url, api_key: &sonarr.api_key });
        }
        services.extend(self.additional_services.iter().map(|service| ServiceRef {
            service_type: &service.service_type, url: &service.url, api_key: &service.api_key,
        }));
        let mut state = serializer.serialize_struct("PersistedConfig", 4)?;
        state.serialize_field("schema_version", &1u32)?;
        state.serialize_field("device_names", &self.device_names)?;
        state.serialize_field("services", &services)?;
        state.serialize_field("metadata_filename_key", &self.metadata_filename_key)?;
        state.end()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ArrConnection {
    pub url: String,
    pub api_key: String,
}
impl Drop for ArrConnection {
    fn drop(&mut self) {
        self.api_key.zeroize();
    }
}

pub struct EncryptedArtwork {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

impl Drop for PersistedConfig {
    fn drop(&mut self) {
        self.radarr_url.zeroize();
        self.radarr_api_key.zeroize();
        self.metadata_filename_key.zeroize();
        for service in &mut self.additional_services { service.api_key.zeroize(); }
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
    pub fn shared_dir() -> PathBuf {
        env::var_os("RAWARR_SHARED_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data/shared"))
    }

    /// The metadata domain is mounted at /metadata in the container and remains
    /// a subtree of the Syncthing shared root by default.
    pub fn metadata_dir() -> PathBuf {
        metadata_dir_for(
            &Self::shared_dir(),
            env::var_os("RAWARR_METADATA_DIR").map(PathBuf::from),
        )
    }

    pub fn config_path() -> PathBuf {
        Self::shared_dir().join("config.aged")
    }

    pub fn local_config_path() -> PathBuf {
        local_data_dir().join("radarr-config.json")
    }

    pub fn enforce_local_config_policy() -> Result<(), VaultError> {
        if local_plaintext_config_enabled() {
            return Ok(());
        }
        remove_local_config(&Self::local_config_path())
    }

    pub fn load_local_config() -> Result<Option<PersistedConfig>, VaultError> {
        if !local_plaintext_config_enabled() {
            return Ok(None);
        }
        load_local_config(&Self::local_config_path())
    }

    pub fn public_recipient_path() -> PathBuf {
        Self::shared_dir().join("age-recipient")
    }

    fn artwork_path(&self, tmdb_id: u64) -> Result<PathBuf, VaultError> {
        let opaque = self.opaque_metadata_filename("artwork", "tmdb", tmdb_id, ".aged")?;
        let opaque_path = self.metadata_dir
            .join("movies")
            .join("artwork")
            .join(&opaque);
        Ok(if opaque_path.is_file() { opaque_path } else {
            self.metadata_dir.join("movies").join("artwork").join(format!("{tmdb_id}.aged"))
        })
    }

    fn movie_metadata_path(&self, tmdb_id: u64) -> Result<PathBuf, VaultError> {
        let opaque = self.opaque_metadata_filename("text", "tmdb", tmdb_id, ".json.aged")?;
        let opaque_path = self.metadata_dir
            .join("movies")
            .join("text")
            .join(&opaque);
        Ok(if opaque_path.is_file() { opaque_path } else {
            self.metadata_dir.join("movies").join("text").join(format!("{tmdb_id}.json.aged"))
        })
    }

    fn tv_artwork_path(&self, tvdb_id: u64) -> Result<PathBuf, VaultError> {
        let opaque = self.opaque_metadata_filename("artwork", "tvdb", tvdb_id, ".aged")?;
        Ok(self.metadata_dir.join("tv").join("artwork").join(opaque))
    }

    fn tv_metadata_path(&self, tvdb_id: u64) -> Result<PathBuf, VaultError> {
        let opaque = self.opaque_metadata_filename("text", "tvdb", tvdb_id, ".json.aged")?;
        Ok(self.metadata_dir.join("tv").join("text").join(opaque))
    }

    fn opaque_metadata_filename(&self, kind: &str, provider_id: &str, id: u64, suffix: &str) -> Result<String, VaultError> {
        if id == 0 { return Err(VaultError::InvalidConfig); }
        let key = self.filename_key()?;
        let mut mac = Hmac::<Sha256>::new_from_slice(&key).expect("HMAC accepts any key length");
        mac.update(b"rawrarr-metadata-filename-v1\0");
        mac.update(kind.as_bytes());
        mac.update(b"\0");
        mac.update(provider_id.as_bytes());
        mac.update(b"\0");
        mac.update(id.to_string().as_bytes());
        Ok(format!("{}{}", encode_hex(&mac.finalize().into_bytes()), suffix))
    }

    fn filename_key(&self) -> Result<Zeroizing<Vec<u8>>, VaultError> {
        if let Some(key) = self.metadata_filename_key.get() {
            return Ok(Zeroizing::new(key.to_vec()));
        }
        let mut config = self.read_config()?.ok_or(VaultError::InvalidConfig)?;
        ensure_filename_key(&mut config)?;
        let encoded = config.metadata_filename_key.as_deref().ok_or(VaultError::InvalidConfig)?;
        let bytes = decode_hex_32(encoded).ok_or(VaultError::InvalidConfig)?;
        let key = Zeroizing::new(bytes.to_vec());
        let _ = self.metadata_filename_key.set(key.clone());
        Ok(key)
    }

    /// Copy legacy numeric ciphertext to opaque HMAC paths, verify identical bytes,
    /// then remove the legacy name. Conflicting opaque files leave the legacy copy intact.
    pub fn migrate_legacy_movie_filenames(&self) -> Result<usize, VaultError> {
        let mut migrated = 0;
        for kind in ["text", "artwork"] {
            let directory = self.metadata_dir.join("movies").join(kind);
            let entries = match fs::read_dir(&directory) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return Err(VaultError::StorageFailed),
            };
            for entry in entries {
                let entry = entry.map_err(|_| VaultError::StorageFailed)?;
                let source = entry.path();
                if !source.is_file() { continue; }
                let Some(name) = source.file_name().and_then(|name| name.to_str()) else { continue; };
                let id_text = if kind == "text" { name.strip_suffix(".json.aged") } else { name.strip_suffix(".aged") };
                let Some(id) = id_text.and_then(|value| value.parse::<u64>().ok()).filter(|id| *id > 0) else { continue; };
                let destination = self.opaque_path_for(kind, id)?;
                if destination == source { continue; }
                if destination.exists() {
                    if fs::read(&destination).map_err(|_| VaultError::StorageFailed)? == fs::read(&source).map_err(|_| VaultError::StorageFailed)? {
                        fs::remove_file(&source).map_err(|_| VaultError::StorageFailed)?;
                        migrated += 1;
                    }
                    continue;
                }
                let temporary = temporary_path(&destination);
                fs::copy(&source, &temporary).map_err(|_| VaultError::StorageFailed)?;
                if fs::read(&temporary).map_err(|_| VaultError::StorageFailed)? != fs::read(&source).map_err(|_| VaultError::StorageFailed)? {
                    let _ = fs::remove_file(&temporary);
                    return Err(VaultError::StorageFailed);
                }
                fs::rename(&temporary, &destination).map_err(|_| {
                    let _ = fs::remove_file(&temporary);
                    VaultError::StorageFailed
                })?;
                fs::remove_file(&source).map_err(|_| VaultError::StorageFailed)?;
                migrated += 1;
            }
        }
        Ok(migrated)
    }

    fn opaque_path_for(&self, kind: &str, tmdb_id: u64) -> Result<PathBuf, VaultError> {
        if tmdb_id == 0 { return Err(VaultError::InvalidConfig); }
        let suffix = if kind == "text" { ".json.aged" } else { ".aged" };
        let filename = self.opaque_metadata_filename(kind, "tmdb", tmdb_id, suffix)?;
        Ok(self.metadata_dir.join("movies").join(kind).join(filename))
    }

    pub fn store_movie_metadata(&self, tmdb_id: u64, value: &[u8]) -> Result<(), VaultError> {
        let path = self.opaque_path_for("text", tmdb_id)?;
        Self::store_movie_metadata_with_recipient(
            &self.recipient,
            tmdb_id,
            value,
            &path,
        )
    }

    fn store_movie_metadata_with_recipient(
        recipient: &str,
        tmdb_id: u64,
        value: &[u8],
        path: &Path,
    ) -> Result<(), VaultError> {
        const MAX_METADATA_BYTES: usize = 7 * 1024 * 1024;
        if tmdb_id == 0
            || value.is_empty()
            || value.len() > MAX_METADATA_BYTES
            || serde_json::from_slice::<serde_json::Value>(value).is_err()
        {
            return Err(VaultError::InvalidConfig);
        }
        encrypt_shared_file(
            recipient,
            path,
            value,
        )
    }

    fn read_shared_recipient() -> Result<String, VaultError> {
        let value = fs::read_to_string(Self::public_recipient_path())
            .map_err(|_| VaultError::StorageFailed)?;
        let recipient = value.trim();
        if !is_post_quantum_recipient(recipient) {
            return Err(VaultError::InvalidIdentity);
        }
        Ok(recipient.to_owned())
    }

    pub fn movie_metadata_exists(&self, tmdb_id: u64) -> bool {
        self.movie_metadata_path(tmdb_id).is_ok_and(|path| path.is_file())
    }

    pub fn artwork_exists(&self, tmdb_id: u64) -> bool {
        self.artwork_path(tmdb_id).is_ok_and(|path| path.is_file())
    }

    pub fn tv_metadata_exists(&self, tvdb_id: u64) -> bool {
        self.tv_metadata_path(tvdb_id).is_ok_and(|path| path.is_file())
    }

    pub fn tv_artwork_exists(&self, tvdb_id: u64) -> bool {
        self.tv_artwork_path(tvdb_id).is_ok_and(|path| path.is_file())
    }

    pub fn store_tv_metadata(&self, tvdb_id: u64, value: &[u8]) -> Result<(), VaultError> {
        const MAX_METADATA_BYTES: usize = 7 * 1024 * 1024;
        if tvdb_id == 0 || value.is_empty() || value.len() > MAX_METADATA_BYTES
            || serde_json::from_slice::<serde_json::Value>(value).is_err() {
            return Err(VaultError::InvalidConfig);
        }
        let path = self.tv_metadata_path(tvdb_id)?;
        encrypt_shared_file(&self.recipient, &path, value)
    }

    pub fn load_tv_metadata(
        &self,
        tvdb_id: u64,
    ) -> Result<Option<serde_json::Value>, VaultError> {
        const MAX_CIPHERTEXT_BYTES: u64 = 7 * 1024 * 1024 + 4096;
        let path = self.tv_metadata_path(tvdb_id)?;
        let Ok(metadata) = fs::metadata(&path) else {
            return Ok(None);
        };
        if metadata.len() > MAX_CIPHERTEXT_BYTES {
            return Err(VaultError::InvalidConfig);
        }
        self.movie_metadata_cache.load(&path, || {
            let output = run_with_input(
                "age",
                &["--decrypt", "--identity", "-", path_arg(&path)?],
                self.identity.as_bytes(),
            )?;
            if !output.status.success() {
                return Err(VaultError::DecryptionFailed);
            }
            let plaintext = Zeroizing::new(output.stdout);
            serde_json::from_slice(plaintext.as_slice())
                .map(Some)
                .map_err(|_| VaultError::InvalidConfig)
        })
    }

    pub fn store_tv_artwork(&self, tvdb_id: u64, content_type: &str, bytes: &[u8]) -> Result<(), VaultError> {
        let path = self.tv_artwork_path(tvdb_id)?;
        store_artwork_with_recipient(&self.recipient, tvdb_id, content_type, bytes, &path)
    }

    pub fn store_artwork(
        &self,
        tmdb_id: u64,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), VaultError> {
        let path = self.opaque_path_for("artwork", tmdb_id)?;
        store_artwork_with_recipient(
            &self.recipient,
            tmdb_id,
            content_type,
            bytes,
            &path,
        )
    }

    /// Store one JSON media sidecar as age ciphertext under the shared Rawrarr folder.
    /// The deterministic filename is derived from device, library, and relative media path;
    /// those values remain inside the encrypted JSON payload.
    pub fn store_library_sidecar(
        &self,
        device_id: &str,
        library_name: &str,
        relative_path: &str,
        plaintext_json: &[u8],
    ) -> Result<(), VaultError> {
        Self::store_library_sidecar_with_recipient(
            &self.metadata_dir,
            &self.recipient,
            device_id,
            library_name,
            relative_path,
            plaintext_json,
        )
    }

    /// Store a sidecar using only the public age recipient, for storage-only nodes.
    /// `metadata_dir` is the node's /metadata domain, backed by /shared/metadata.
    pub fn store_library_sidecar_with_recipient(
        metadata_dir: &Path,
        recipient: &str,
        device_id: &str,
        library_name: &str,
        relative_path: &str,
        plaintext_json: &[u8],
    ) -> Result<(), VaultError> {
        const MAX_SIDECAR_BYTES: usize = 1024 * 1024;
        if plaintext_json.is_empty()
            || plaintext_json.len() > MAX_SIDECAR_BYTES
            || serde_json::from_slice::<serde_json::Value>(plaintext_json).is_err()
        {
            return Err(VaultError::InvalidConfig);
        }
        let path = Self::library_sidecar_path(
            metadata_dir,
            recipient,
            device_id,
            library_name,
            relative_path,
        )?;
        encrypt_shared_file(recipient, &path, plaintext_json)
    }

    /// Check whether the opaque shared sidecar path already has a regular file.
    /// Storage scans use this to fill gaps without rewriting existing metadata.
    pub fn library_sidecar_exists_with_recipient(
        metadata_dir: &Path,
        recipient: &str,
        device_id: &str,
        library_name: &str,
        relative_path: &str,
    ) -> Result<bool, VaultError> {
        let path = Self::library_sidecar_path(
            metadata_dir,
            recipient,
            device_id,
            library_name,
            relative_path,
        )?;
        Ok(fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_file()))
    }

    fn library_sidecar_path(
        metadata_dir: &Path,
        recipient: &str,
        device_id: &str,
        library_name: &str,
        relative_path: &str,
    ) -> Result<PathBuf, VaultError> {
        let relative = Path::new(relative_path);
        if !is_post_quantum_recipient(recipient)
            || device_id.trim().is_empty()
            || library_name.trim().is_empty()
            || relative_path.is_empty()
            || relative.is_absolute()
            || relative
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(VaultError::InvalidConfig);
        }
        let mut digest = Sha256::new();
        digest.update(device_id.as_bytes());
        digest.update([0]);
        digest.update(library_name.as_bytes());
        digest.update([0]);
        digest.update(relative_path.as_bytes());
        let filename = format!("{}.json.aged", encode_hex(&digest.finalize()));
        Ok(metadata_dir.join("library").join("text").join(filename))
    }

    pub fn persist_public_recipient(&self) -> Result<(), VaultError> {
        let path = Self::public_recipient_path_for(&self.shared_dir);
        if path.is_file() {
            let existing = fs::read_to_string(&path).map_err(|_| VaultError::StorageFailed)?;
            if existing.trim() == self.recipient {
                return Ok(());
            }
            return Err(VaultError::InvalidIdentity);
        }
        encrypt_text_file(&path, format!("{}\n", self.recipient).as_bytes())
    }

    pub fn recipient(&self) -> &str {
        &self.recipient
    }

    fn public_recipient_path_for(shared_dir: &Path) -> PathBuf {
        shared_dir.join("age-recipient")
    }

    pub fn load_artwork(&self, tmdb_id: u64) -> Result<Option<EncryptedArtwork>, VaultError> {
        self.load_artwork_at_id(tmdb_id, false, None)
    }

    pub fn load_artwork_cached(&self, tmdb_id: u64, cache: &crate::artwork_cache::ArtworkCache) -> Result<Option<EncryptedArtwork>, VaultError> {
        self.load_artwork_at_id(tmdb_id, false, Some(cache))
    }

    pub fn load_tv_artwork(&self, tvdb_id: u64) -> Result<Option<EncryptedArtwork>, VaultError> {
        self.load_artwork_at_id(tvdb_id, true, None)
    }

    pub fn load_tv_artwork_cached(&self, tvdb_id: u64, cache: &crate::artwork_cache::ArtworkCache) -> Result<Option<EncryptedArtwork>, VaultError> {
        self.load_artwork_at_id(tvdb_id, true, Some(cache))
    }

    fn load_artwork_at_id(&self, id: u64, is_tv: bool, cache: Option<&crate::artwork_cache::ArtworkCache>) -> Result<Option<EncryptedArtwork>, VaultError> {
        const MAX_CIPHERTEXT_BYTES: u64 = 9 * 1024 * 1024;
        const MAX_ARTWORK_BYTES: usize = 8 * 1024 * 1024;
        let path = if is_tv { self.tv_artwork_path(id)? } else { self.artwork_path(id)? };
        let Ok(metadata) = fs::metadata(&path) else {
            return Ok(None);
        };
        if metadata.len() > MAX_CIPHERTEXT_BYTES {
            return Err(VaultError::InvalidConfig);
        }
        let ciphertext = if let Some(cache) = cache {
            cache.read(&path).map_err(|_| VaultError::StorageFailed)?
        } else {
            fs::read(&path).map_err(|_| VaultError::StorageFailed)?
        };
        let output = run_with_input_data(
            "age",
            &["--decrypt", "--identity"],
            self.identity.as_bytes(),
            &ciphertext,
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

    pub fn load_movie_metadata(
        &self,
        tmdb_id: u64,
    ) -> Result<Option<serde_json::Value>, VaultError> {
        const MAX_CIPHERTEXT_BYTES: u64 = 7 * 1024 * 1024 + 4096;
        let path = self.movie_metadata_path(tmdb_id)?;
        let Ok(metadata) = fs::metadata(&path) else {
            return Ok(None);
        };
        if metadata.len() > MAX_CIPHERTEXT_BYTES {
            return Err(VaultError::InvalidConfig);
        }
        self.movie_metadata_cache.load(&path, || {
            let output = run_with_input(
                "age",
                &["--decrypt", "--identity", "-", path_arg(&path)?],
                self.identity.as_bytes(),
            )?;
            if !output.status.success() {
                return Err(VaultError::DecryptionFailed);
            }
            let plaintext = Zeroizing::new(output.stdout);
            serde_json::from_slice(plaintext.as_slice())
                .map(Some)
                .map_err(|_| VaultError::InvalidConfig)
        })
    }

    pub fn from_identity(identity: String) -> Result<Self, VaultError> {
        Self::from_identity_in_shared_dir(identity, Self::shared_dir())
    }

    pub fn persist_identity_locally(&self) -> Result<(), VaultError> {
        let data_dir = env::var_os("RAWARR_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data"));
        fs::create_dir_all(&data_dir).map_err(|_| VaultError::StorageFailed)?;
        set_private_directory_permissions(&data_dir)?;
        let path = data_dir.join("age-identity");
        let temporary = temporary_path(&path);
        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(|_| VaultError::StorageFailed)?;
            set_private_file_permissions(&file)?;
            file.write_all(self.identity.as_bytes())
                .map_err(|_| VaultError::StorageFailed)?;
            file.sync_all().map_err(|_| VaultError::StorageFailed)?;
            fs::rename(&temporary, &path).map_err(|_| VaultError::StorageFailed)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    fn from_identity_in_shared_dir(
        identity: String,
        shared_dir: PathBuf,
    ) -> Result<Self, VaultError> {
        Self::from_identity_in_dirs(
            identity,
            shared_dir,
            Self::local_config_path(),
            local_plaintext_config_enabled(),
        )
    }

    fn from_identity_in_dirs(
        identity: String,
        shared_dir: PathBuf,
        local_config_path: PathBuf,
        local_plaintext_config: bool,
    ) -> Result<Self, VaultError> {
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
            movie_metadata_cache: Default::default(),
            identity,
            recipient,
            metadata_dir: metadata_dir_for(
                &shared_dir,
                env::var_os("RAWARR_METADATA_DIR").map(PathBuf::from),
            ),
            config_path: shared_dir.join("config.aged"),
            local_config_path,
            local_plaintext_config,
            metadata_filename_key: std::sync::Arc::new(std::sync::OnceLock::new()),
            shared_dir,
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
        let _guard = CONFIG_WRITE_LOCK
            .lock()
            .map_err(|_| VaultError::StorageFailed)?;
        let Some(mut config) = self.read_config()? else {
            return Ok(None);
        };
        if config.needs_migration || ensure_filename_key(&mut config)? {
            config.needs_migration = false;
            ensure_filename_key(&mut config)?;
            self.write_config(&config)?;
        }
        if self.local_plaintext_config && !config.radarr_url.is_empty() {
            persist_local_config(&self.local_config_path, &config)?;
        }
        Ok(Some(config))
    }

    fn read_config(&self) -> Result<Option<PersistedConfig>, VaultError> {
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
        let config: PersistedConfig =
            serde_json::from_slice(plaintext.as_slice()).map_err(|_| VaultError::InvalidConfig)?;
        Ok(Some(config))
    }

    pub fn save(&self, config: &PersistedConfig) -> Result<(), VaultError> {
        let _guard = CONFIG_WRITE_LOCK
            .lock()
            .map_err(|_| VaultError::StorageFailed)?;
        let existing = self.read_config()?;
        let mut updated = PersistedConfig {
            device_names: existing.as_ref().map(|stored| stored.device_names.clone()).unwrap_or_default(),
            sonarr: existing.as_ref().and_then(|stored| stored.sonarr.clone()),
            radarr_url: config.radarr_url.clone(),
            radarr_api_key: config.radarr_api_key.clone(),
            metadata_filename_key: existing
                .as_ref()
                .and_then(|stored| stored.metadata_filename_key.clone())
                .or_else(|| config.metadata_filename_key.clone()),
            additional_services: existing.as_ref().map(|stored| stored.additional_services.clone()).unwrap_or_default(),
            needs_migration: false,
        };
        ensure_filename_key(&mut updated)?;
        self.write_config(&updated)
    }

    fn write_config(&self, config: &PersistedConfig) -> Result<(), VaultError> {
        let plaintext =
            Zeroizing::new(serde_json::to_vec(config).map_err(|_| VaultError::InvalidConfig)?);
        let directory = self.config_path.parent().ok_or(VaultError::StorageFailed)?;
        fs::create_dir_all(directory).map_err(|_| VaultError::StorageFailed)?;
        set_shared_directory_permissions(directory)?;
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
        if let Err(error) = set_shared_ciphertext_permissions(&temporary) {
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
        })?;
        if self.local_plaintext_config {
            persist_local_config(&self.local_config_path, config)?;
        }
        Ok(())
    }

    /// Update only Sonarr while holding the same lock used by Radarr saves.
    pub fn save_sonarr(&self, connection: Option<ArrConnection>) -> Result<(), VaultError> {
        let _guard = CONFIG_WRITE_LOCK
            .lock()
            .map_err(|_| VaultError::StorageFailed)?;
        let mut config = self.read_config()?.unwrap_or(PersistedConfig {
            device_names: std::collections::HashMap::new(),
            radarr_url: String::new(),
            radarr_api_key: String::new(),
            sonarr: None,
            metadata_filename_key: None,
            additional_services: Vec::new(),
            needs_migration: false,
        });
        config.sonarr = connection;
        ensure_filename_key(&mut config)?;
        self.write_config(&config)
    }

    pub fn clear(&self) -> Result<(), VaultError> {
        let _guard = CONFIG_WRITE_LOCK
            .lock()
            .map_err(|_| VaultError::StorageFailed)?;
        if let Some(mut config) = self.read_config()? {
            ensure_filename_key(&mut config)?;
            config.radarr_url.zeroize();
            config.radarr_url.clear();
            config.radarr_api_key.zeroize();
            config.radarr_api_key.clear();
            self.write_config(&config)?;
        }
        remove_local_config(&self.local_config_path)
    }
}

// Serialize initialization/settings changes within the node. Across nodes, synchronize
// the provisioned config before use; independent initializations cannot share a key.
static CONFIG_WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn ensure_filename_key(config: &mut PersistedConfig) -> Result<bool, VaultError> {
    if let Some(key) = &config.metadata_filename_key {
        if key.len() != 64
            || !key
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(VaultError::InvalidConfig);
        }
        return Ok(false);
    }
    let mut bytes = Zeroizing::new([0u8; 32]);
    getrandom::fill(bytes.as_mut()).map_err(|_| VaultError::StorageFailed)?;
    config.metadata_filename_key = Some(bytes.iter().map(|b| format!("{b:02x}")).collect());
    Ok(true)
}

fn local_plaintext_config_enabled() -> bool {
    env::var("RAWARR_LOCAL_PLAINTEXT_CONFIG")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn local_data_dir() -> PathBuf {
    env::var_os("RAWARR_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
}

fn persist_local_config(path: &Path, config: &PersistedConfig) -> Result<(), VaultError> {
    let directory = path.parent().ok_or(VaultError::StorageFailed)?;
    fs::create_dir_all(directory).map_err(|_| VaultError::StorageFailed)?;
    set_local_directory_permissions(directory)?;
    // This optional local mirror is Radarr-only; never copy the shared filename key here.
    #[derive(Serialize)]
    struct LocalRadarrConfig<'a> {
        radarr_url: &'a str,
        radarr_api_key: &'a str,
    }
    let local = LocalRadarrConfig {
        radarr_url: &config.radarr_url,
        radarr_api_key: &config.radarr_api_key,
    };
    let plaintext =
        Zeroizing::new(serde_json::to_vec(&local).map_err(|_| VaultError::InvalidConfig)?);
    let temporary = temporary_path(path);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| VaultError::StorageFailed)?;
    set_local_file_permissions(&file)?;
    let result = file
        .write_all(plaintext.as_slice())
        .and_then(|()| file.sync_all());
    drop(file);
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
        return Err(VaultError::StorageFailed);
    }
    fs::rename(&temporary, path).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        VaultError::StorageFailed
    })
}

fn load_local_config(path: &Path) -> Result<Option<PersistedConfig>, VaultError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(VaultError::StorageFailed),
    };
    if bytes.len() > 64 * 1024 {
        return Err(VaultError::InvalidConfig);
    }
    let plaintext = Zeroizing::new(bytes);
    serde_json::from_slice(plaintext.as_slice())
        .map(Some)
        .map_err(|_| VaultError::InvalidConfig)
}

fn remove_local_config(path: &Path) -> Result<(), VaultError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(VaultError::StorageFailed),
    }
}

fn set_local_file_permissions(file: &fs::File) -> Result<(), VaultError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|_| VaultError::StorageFailed)?;
    }
    Ok(())
}

fn set_local_directory_permissions(path: &Path) -> Result<(), VaultError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| VaultError::StorageFailed)?;
    }
    Ok(())
}

fn set_private_file_permissions(file: &fs::File) -> Result<(), VaultError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
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

fn metadata_dir_for(shared_dir: &Path, override_dir: Option<PathBuf>) -> PathBuf {
    override_dir.unwrap_or_else(|| shared_dir.join("metadata"))
}

pub fn is_post_quantum_recipient(recipient: &str) -> bool {
    recipient.starts_with("age1pq1")
        && (20..=4096).contains(&recipient.len())
        && recipient
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn encrypt_text_file(path: &Path, plaintext: &[u8]) -> Result<(), VaultError> {
    let directory = path.parent().ok_or(VaultError::StorageFailed)?;
    fs::create_dir_all(directory).map_err(|_| VaultError::StorageFailed)?;
    set_shared_directory_permissions(directory)?;
    let temporary = temporary_path(path);
    fs::write(&temporary, plaintext).map_err(|_| VaultError::StorageFailed)?;
    set_shared_ciphertext_permissions(&temporary)?;
    fs::rename(&temporary, path).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        VaultError::StorageFailed
    })
}

fn encrypt_shared_file(recipient: &str, path: &Path, plaintext: &[u8]) -> Result<(), VaultError> {
    if !is_post_quantum_recipient(recipient) {
        return Err(VaultError::InvalidIdentity);
    }
    let directory = path.parent().ok_or(VaultError::StorageFailed)?;
    fs::create_dir_all(directory).map_err(|_| VaultError::StorageFailed)?;
    if let Some(metadata_directory) = directory.parent() {
        set_shared_directory_permissions(metadata_directory)?;
    }
    set_shared_directory_permissions(directory)?;
    let temporary = temporary_path(path);
    let result = Command::new("age")
        .args(["--recipient", recipient, "--output"])
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
                    .write_all(plaintext)
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
    if let Err(error) = set_shared_ciphertext_permissions(&temporary) {
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
    fs::rename(&temporary, path).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        VaultError::StorageFailed
    })?;
    Ok(())
}

fn store_artwork_with_recipient(
    recipient: &str,
    tmdb_id: u64,
    content_type: &str,
    bytes: &[u8],
    path: &Path,
) -> Result<(), VaultError> {
    const MAX_ARTWORK_BYTES: usize = 8 * 1024 * 1024;
    if tmdb_id == 0
        || bytes.is_empty()
        || bytes.len() > MAX_ARTWORK_BYTES
        || !matches!(content_type, "image/jpeg" | "image/png" | "image/webp")
    {
        return Err(VaultError::InvalidConfig);
    }
    let mut plaintext = Zeroizing::new(Vec::with_capacity(content_type.len() + 1 + bytes.len()));
    plaintext.extend_from_slice(content_type.as_bytes());
    plaintext.push(b'\n');
    plaintext.extend_from_slice(bytes);
    encrypt_shared_file(
        recipient,
        path,
        &plaintext,
    )
}

fn decode_hex_32(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 { return None; }
    let mut output = [0u8; 32];
    for (index, byte) in output.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(output)
}

fn run_with_input_data(
    program: &str,
    args: &[&str],
    identity: &[u8],
    data: &[u8],
) -> Result<std::process::Output, VaultError> {
    let identity_path = std::env::temp_dir().join(format!("rawrarr-age-identity-{}-{}", std::process::id(), uuid::Uuid::new_v4()));
    use std::os::unix::fs::OpenOptionsExt;
    let operation = (|| {
        let mut identity_file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&identity_path).map_err(|_| VaultError::StorageFailed)?;
        identity_file.write_all(identity).map_err(|_| VaultError::StorageFailed)?;
        drop(identity_file);
        let identity_arg = identity_path.to_str().ok_or(VaultError::StorageFailed)?;
        let mut full_args = args.to_vec();
        full_args.extend_from_slice(&[identity_arg, "-"]);
        let child = Command::new(program)
            .args(full_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| VaultError::AgeUnavailable)?;
        wait_with_input(child, data, false)
    })();
    let _ = fs::remove_file(identity_path);
    operation
}

fn run_with_input(
    program: &str,
    args: &[&str],
    input: &[u8],
) -> Result<std::process::Output, VaultError> {
    let child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| VaultError::AgeUnavailable)?;
    wait_with_input(child, input, true)
}

/// Decrypt a bounded in-memory age payload using an identity supplied by the node-local vault.
pub(crate) fn decrypt_age_input(identity: &[u8], ciphertext: &[u8]) -> Result<std::process::Output, VaultError> {
    run_with_input_data("age", &["--decrypt", "--identity"], identity, ciphertext)
}

fn wait_with_input(
    mut child: std::process::Child,
    input: &[u8],
    encrypting: bool,
) -> Result<std::process::Output, VaultError> {
    let error = || {
        if encrypting {
            VaultError::EncryptionFailed
        } else {
            VaultError::DecryptionFailed
        }
    };
    let mut stdin = child.stdin.take().ok_or_else(error)?;
    let (output, write_result) = std::thread::scope(|scope| {
        let writer = scope.spawn(move || stdin.write_all(input));
        let output = child.wait_with_output();
        let write_result = writer.join();
        (output, write_result)
    });
    let output = output.map_err(|_| error())?;
    write_result.map_err(|_| error())?.map_err(|_| error())?;
    Ok(output)
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

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

// Syncthing must be able to read replicated files. Shared files contain age
// ciphertext only; plaintext is kept in memory while encrypting/decrypting.
fn set_shared_ciphertext_permissions(path: &Path) -> Result<(), VaultError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o644))
            .map_err(|_| VaultError::StorageFailed)?;
    }
    Ok(())
}

fn set_shared_directory_permissions(path: &Path) -> Result<(), VaultError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .map_err(|_| VaultError::StorageFailed)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_domain_defaults_to_shared_subtree_and_accepts_container_alias() {
        let shared = PathBuf::from("/shared");
        assert_eq!(
            metadata_dir_for(&shared, None),
            PathBuf::from("/shared/metadata")
        );
        assert_eq!(
            metadata_dir_for(&shared, Some(PathBuf::from("/metadata"))),
            PathBuf::from("/metadata")
        );
    }

    #[test]
    fn encrypted_config_and_artwork_are_shareable_age_ciphertexts() {
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
        let shared_dir = env::temp_dir().join(format!(
            "rawrarr-artwork-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&shared_dir).expect("create isolated shared directory");
        let data_dir = env::temp_dir().join(format!(
            "rawrarr-local-config-test-{}-{unique}",
            std::process::id()
        ));
        let local_config_path = data_dir.join("radarr-config.json");
        let vault = AgeVault::from_identity_in_dirs(
            identity,
            shared_dir.clone(),
            local_config_path.clone(),
            true,
        )
        .expect("valid PQ age identity");
        vault
            .persist_public_recipient()
            .expect("persist public age recipient for storage nodes");
        assert_eq!(
            fs::read_to_string(shared_dir.join("age-recipient"))
                .unwrap()
                .trim(),
            vault.recipient()
        );
        let config = PersistedConfig {
            device_names: std::collections::HashMap::new(),
            sonarr: None,
            radarr_url: "http://radarr.example:7878".into(),
            radarr_api_key: "disposable encrypted config fixture".into(),
            metadata_filename_key: None,
            additional_services: Vec::new(),
            needs_migration: false,
        };
        vault.save(&config).expect("save age-encrypted config");
        let config_path = shared_dir.join("config.aged");
        let encrypted_config = fs::read(&config_path).expect("read encrypted config");
        assert!(encrypted_config.starts_with(b"age-encryption.org/v1\n"));
        assert!(shared_dir.join("config.aged").is_file());
        assert!(
            !encrypted_config
                .windows(config.radarr_api_key.len())
                .any(|window| window == config.radarr_api_key.as_bytes())
        );
        let local_config = fs::read(&local_config_path).expect("read node-local config mirror");
        assert_eq!(
            serde_json::from_slice::<PersistedConfig>(&local_config)
                .expect("parse local config mirror")
                .radarr_api_key,
            config.radarr_api_key
        );
        assert!(!shared_dir.join("radarr-config.json").exists());
        assert_eq!(
            vault.load().expect("decrypt config").unwrap().radarr_url,
            config.radarr_url
        );
        assert!(vault.config_exists());

        let wrong_identity_output = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("generate unrelated PQ age identity");
        assert!(wrong_identity_output.status.success());
        let wrong_identity =
            String::from_utf8(wrong_identity_output.stdout).expect("wrong identity is UTF-8");
        let wrong_vault = AgeVault::from_identity_in_shared_dir(wrong_identity, shared_dir.clone())
            .expect("second valid PQ age identity");
        assert!(matches!(
            wrong_vault.persist_public_recipient(),
            Err(VaultError::InvalidIdentity)
        ));
        assert!(matches!(
            wrong_vault.load(),
            Err(VaultError::DecryptionFailed)
        ));

        // Exceed normal pipe capacity so the writer and age stdout reader must run
        // concurrently; small fixtures would not catch a subprocess pipe deadlock.
        let image: Vec<u8> = (0..512 * 1024)
            .map(|index| ((index * 31 + index / 251) % 256) as u8)
            .collect();
        vault
            .store_artwork(12345, "image/jpeg", &image)
            .expect("encrypt poster into .aged file");

        let encrypted_path = vault.artwork_path(12345).expect("derive opaque poster filename");
        let encrypted = fs::read(&encrypted_path).expect("read encrypted poster");
        assert!(encrypted.starts_with(b"age-encryption.org/v1\n"));
        assert!(!encrypted.windows(image.len()).any(|window| window == image.as_slice()));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&config_path).unwrap().permissions().mode() & 0o777,
                0o644
            );
            assert_eq!(
                fs::metadata(shared_dir.join("metadata"))
                    .expect("metadata directory")
                    .permissions()
                    .mode()
                    & 0o777,
                0o755
            );
            assert_eq!(
                fs::metadata(&local_config_path)
                    .expect("node-local config mirror")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
            assert_eq!(
                fs::metadata(&data_dir)
                    .expect("node-local data directory")
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
        let movie_metadata = br#"{"schemaVersion":1,"tmdbId":12345,"title":"Fixture Movie"}"#;
        vault
            .store_movie_metadata(12345, movie_metadata)
            .expect("encrypt movie metadata into shared folder");
        let metadata_path = vault.movie_metadata_path(12345).expect("derive opaque metadata filename");
        let encrypted_metadata = fs::read(&metadata_path).expect("read encrypted movie metadata");
        assert!(encrypted_metadata.starts_with(b"age-encryption.org/v1\n"));
        assert!(
            !encrypted_metadata
                .windows(b"Fixture Movie".len())
                .any(|window| window == b"Fixture Movie")
        );
        assert!(vault.movie_metadata_exists(12345));
        assert_eq!(
            vault
                .load_movie_metadata(12345)
                .expect("decrypt movie metadata")
                .unwrap()["title"],
            "Fixture Movie"
        );

        let sidecar_json = br#"{"schemaVersion":1,"deviceId":"storage-test","file":{"path":"Movies/Test Film (2024)/film.mkv","sizeBytes":42}}"#;
        vault
            .store_library_sidecar(
                "storage-test",
                "Test library",
                "Movies/Test Film (2024)/film.mkv",
                sidecar_json,
            )
            .expect("encrypt media JSON sidecar in shared folder");
        let sidecar_directory = shared_dir.join("metadata/library/text");
        let sidecar_path = fs::read_dir(&sidecar_directory)
            .expect("sidecar directory exists")
            .next()
            .expect("sidecar file exists")
            .expect("read sidecar entry")
            .path();
        assert!(sidecar_path.extension().is_some_and(|ext| ext == "aged"));
        assert!(
            sidecar_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".json.aged")
        );
        let encrypted_sidecar = fs::read(&sidecar_path).expect("read encrypted JSON sidecar");
        assert!(encrypted_sidecar.starts_with(b"age-encryption.org/v1\n"));
        assert!(
            !encrypted_sidecar
                .windows(sidecar_json.len())
                .any(|window| window == sidecar_json)
        );
        let decrypted_sidecar = run_with_input(
            "age",
            &[
                "--decrypt",
                "--identity",
                "-",
                path_arg(&sidecar_path).unwrap(),
            ],
            vault.identity.as_bytes(),
        )
        .expect("decrypt media JSON sidecar");
        assert!(decrypted_sidecar.status.success());
        assert_eq!(decrypted_sidecar.stdout, sidecar_json);

        vault.clear().expect("clear encrypted and local config");
        assert!(config_path.exists());
        let cleared = vault.load().unwrap().unwrap();
        assert!(cleared.radarr_url.is_empty());
        assert!(cleared.radarr_api_key.is_empty());
        assert!(cleared.metadata_filename_key.is_some());
        assert!(!local_config_path.exists());
        fs::remove_dir_all(&data_dir).expect("remove isolated local data directory");
        assert!(
            vault
                .store_library_sidecar(
                    "storage-test",
                    "Test library",
                    "../outside.mkv",
                    sidecar_json
                )
                .is_err()
        );
        fs::remove_dir_all(shared_dir).expect("remove test data");
    }
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_filename_secret.rs"]
mod filename_secret_tests;

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_hmac_metadata_filenames.rs"]
mod hmac_metadata_filename_tests;

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_arr_settings.rs"]
mod arr_settings_tests;

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_sonarr_metadata.rs"]
mod sonarr_metadata_tests;

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_config_schema.rs"]
mod config_schema_tests;
