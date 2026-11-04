use hmac::{Hmac, Mac};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ml_dsa::{MlDsa65, SigningKey, signature::{Keypair, Signer}};
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
    user_recipient: String,
    group_identity: String,
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
        self.user_recipient.zeroize();
        self.group_identity.zeroize();
        self.recipient.zeroize();
    }
}

pub struct PersistedConfig {
    pub device_names: std::collections::HashMap<String, String>,
    pub sonarr: Option<ArrConnection>,
    pub radarr_url: String,
    pub radarr_api_key: String,
    pub radarr_ui_username: String,
    pub radarr_ui_password: String,
    pub additional_services: Vec<ServiceConfig>,
    pub(crate) needs_migration: bool,
    /// Reserved exclusively for opaque metadata filename derivation; never an encryption/auth key.
    pub metadata_filename_key: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreparedAppIdentityRegistry {
    schema_version: u32,
    apps: std::collections::BTreeMap<String, PreparedAppIdentity>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreparedAppIdentity {
    instance_id: String,
    identity: String,
    recipient: String,
}

impl Drop for PreparedAppIdentity {
    fn drop(&mut self) {
        self.identity.zeroize();
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    #[serde(rename = "type")]
    pub service_type: String,
    pub url: String,
    pub api_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_password: Option<String>,
}
impl Drop for ServiceConfig {
    fn drop(&mut self) {
        self.api_key.zeroize();
        self.ui_username.zeroize();
        self.ui_password.zeroize();
    }
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
        let mut radarr_ui_username = String::new();
        let mut radarr_ui_password = String::new();
        let mut sonarr = wire.sonarr;
        let mut additional_services = Vec::new();
        if let Some(services) = wire.services {
            for service in services {
                match service.service_type.as_str() {
                    "radarr" => {
                        radarr_url = service.url.clone();
                        radarr_api_key = service.api_key.clone();
                        radarr_ui_username = service.ui_username.clone().unwrap_or_default();
                        radarr_ui_password = service.ui_password.clone().unwrap_or_default();
                    }
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
            radarr_ui_username,
            radarr_ui_password,
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
            #[serde(skip_serializing_if = "Option::is_none")]
            ui_username: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            ui_password: Option<&'a str>,
        }
        let mut services = Vec::new();
        if !self.radarr_url.is_empty() || !self.radarr_api_key.is_empty() {
            services.push(ServiceRef {
                service_type: "radarr",
                url: &self.radarr_url,
                api_key: &self.radarr_api_key,
                ui_username: (!self.radarr_ui_username.is_empty()).then_some(&self.radarr_ui_username),
                ui_password: (!self.radarr_ui_password.is_empty()).then_some(&self.radarr_ui_password),
            });
        }
        if let Some(sonarr) = &self.sonarr {
            services.push(ServiceRef {
                service_type: "sonarr",
                url: &sonarr.url,
                api_key: &sonarr.api_key,
                ui_username: None,
                ui_password: None,
            });
        }
        services.extend(self.additional_services.iter().map(|service| ServiceRef {
            service_type: &service.service_type,
            url: &service.url,
            api_key: &service.api_key,
            ui_username: service.ui_username.as_deref(),
            ui_password: service.ui_password.as_deref(),
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

const MAX_RADARR_BACKUP_BYTES: usize = 512 * 1024 * 1024;

impl Drop for PersistedConfig {
    fn drop(&mut self) {
        self.radarr_url.zeroize();
        self.radarr_api_key.zeroize();
        self.radarr_ui_username.zeroize();
        self.radarr_ui_password.zeroize();
        self.metadata_filename_key.zeroize();
        for service in &mut self.additional_services { service.api_key.zeroize(); }
    }
}

#[derive(Debug)]
pub enum VaultError {
    InvalidIdentity,
    GroupIdentityUnavailable,
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
            Self::GroupIdentityUnavailable => "Could not unlock the shared group identity",
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
    const PREPARED_APP_IDENTITIES_FILE: &'static str = "app-identities.aged";
    const MAX_PREPARED_APP_IDENTITIES_BYTES: u64 = 64 * 1024;

    fn prepared_app_identities_path(&self) -> PathBuf {
        self.shared_dir.join(Self::PREPARED_APP_IDENTITIES_FILE)
    }

    fn read_prepared_app_identities(
        &self,
    ) -> Result<Option<PreparedAppIdentityRegistry>, VaultError> {
        self.read_prepared_app_identities_at(&self.prepared_app_identities_path())
    }

    fn read_prepared_app_identities_at(
        &self,
        path: &Path,
    ) -> Result<Option<PreparedAppIdentityRegistry>, VaultError> {
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(VaultError::StorageFailed),
        };
        if !metadata.file_type().is_file()
            || metadata.len() > Self::MAX_PREPARED_APP_IDENTITIES_BYTES
        {
            return Err(VaultError::InvalidConfig);
        }
        let ciphertext = fs::read(&path).map_err(|_| VaultError::StorageFailed)?;
        if !ciphertext.starts_with(b"age-encryption.org/v1\n") {
            return Err(VaultError::InvalidConfig);
        }
        let output = run_with_input_data(
            "age",
            &["--decrypt", "--identity"],
            self.group_identity.as_bytes(),
            &ciphertext,
        )?;
        if !output.status.success() {
            return Err(VaultError::DecryptionFailed);
        }
        let plaintext = Zeroizing::new(output.stdout);
        if plaintext.len() as u64 > Self::MAX_PREPARED_APP_IDENTITIES_BYTES {
            return Err(VaultError::InvalidConfig);
        }
        let registry: PreparedAppIdentityRegistry =
            serde_json::from_slice(plaintext.as_slice()).map_err(|_| VaultError::InvalidConfig)?;
        Self::validate_prepared_app_identities(&registry)?;
        Ok(Some(registry))
    }

    fn validate_prepared_app_identities(
        registry: &PreparedAppIdentityRegistry,
    ) -> Result<(), VaultError> {
        if registry.schema_version != 1 || registry.apps.len() > 2 {
            return Err(VaultError::InvalidConfig);
        }
        let mut instance_ids = std::collections::HashSet::new();
        for (service_type, app) in &registry.apps {
            if !matches!(service_type.as_str(), "radarr" | "sonarr")
                || uuid::Uuid::parse_str(&app.instance_id)
                    .map_or(true, |id| id.to_string() != app.instance_id)
                || !instance_ids.insert(app.instance_id.as_str())
                || !is_post_quantum_recipient(&app.recipient)
                || !has_single_post_quantum_identity(&app.identity)
                || identity_to_recipient(&app.identity)
                    .map_or(true, |recipient| recipient != app.recipient)
            {
                return Err(VaultError::InvalidConfig);
            }
        }
        Ok(())
    }

    fn prepare_app_identities(&self, config: &PersistedConfig) -> Result<(), VaultError> {
        let mut registry = self
            .read_prepared_app_identities()?
            .unwrap_or_else(|| PreparedAppIdentityRegistry {
                schema_version: 1,
                apps: std::collections::BTreeMap::new(),
            });
        let configured = [
            (!config.radarr_url.is_empty() && !config.radarr_api_key.is_empty()).then_some("radarr"),
            config.sonarr.as_ref().map(|_| "sonarr"),
        ];
        let mut changed = false;
        for service_type in configured.into_iter().flatten() {
            if registry.apps.contains_key(service_type) {
                continue;
            }
            let (identity, recipient) = Self::generate_identity()?;
            registry.apps.insert(
                service_type.to_owned(),
                PreparedAppIdentity {
                    instance_id: uuid::Uuid::new_v4().to_string(),
                    identity,
                    recipient,
                },
            );
            changed = true;
        }
        if changed {
            let plaintext = Zeroizing::new(
                serde_json::to_vec(&registry).map_err(|_| VaultError::InvalidConfig)?,
            );
            let path = self.prepared_app_identities_path();
            let staging = temporary_path(&path);
            encrypt_shared_file(&self.recipient, &staging, plaintext.as_slice())?;
            let verification = (|| {
                let verified = self
                    .read_prepared_app_identities_at(&staging)?
                    .ok_or(VaultError::InvalidConfig)?;
                let verified_plaintext = Zeroizing::new(
                    serde_json::to_vec(&verified).map_err(|_| VaultError::InvalidConfig)?,
                );
                if verified_plaintext.as_slice() != plaintext.as_slice() {
                    return Err(VaultError::InvalidConfig);
                }
                Ok(())
            })();
            if verification.is_err() {
                let _ = fs::remove_file(&staging);
            }
            verification?;
            fs::rename(&staging, &path).map_err(|_| {
                let _ = fs::remove_file(&staging);
                VaultError::StorageFailed
            })?;
        }
        Ok(())
    }

    pub fn shared_dir() -> PathBuf {
        env::var_os("RAWARR_SHARED_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data/shared"))
    }

    pub fn shared_in_dir() -> PathBuf {
        shared_in_dir_for(&Self::shared_dir())
    }

    pub fn shared_out_dir() -> PathBuf {
        shared_out_dir_for(&Self::shared_dir())
    }

    /// The metadata domain is mounted at /metadata in the container and remains
    /// a subtree of the Syncthing shared root by default.
    pub fn metadata_dir() -> PathBuf {
        metadata_dir_for(
            &Self::shared_dir(),
            env::var_os("RAWARR_METADATA_DIR").map(PathBuf::from),
        )
    }

    pub fn initialize_metadata_storage() -> Result<(), VaultError> {
        migrate_legacy_metadata(&Self::shared_dir())
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

    pub fn group_recipient_path() -> PathBuf {
        Self::shared_dir().join("group-age-recipient")
    }

    pub fn group_user_envelope_path() -> PathBuf {
        Self::shared_dir().join("group-age-user-envelope.aged")
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
            let output = self.decrypt_shared_file(&path)?;
            let plaintext = Zeroizing::new(output);
            serde_json::from_slice(plaintext.as_slice())
                .map(Some)
                .map_err(|_| VaultError::InvalidConfig)
        })
    }

    fn decrypt_shared_file(&self, path: &Path) -> Result<Vec<u8>, VaultError> {
        let ciphertext = fs::read(path).map_err(|_| VaultError::StorageFailed)?;
        self.decrypt_shared_bytes(&ciphertext)
    }

    fn decrypt_shared_bytes(&self, ciphertext: &[u8]) -> Result<Vec<u8>, VaultError> {
        let group = run_with_input_data("age", &["--decrypt", "--identity"], self.group_identity.as_bytes(), ciphertext)?;
        if group.status.success() {
            return Ok(group.stdout);
        }
        let legacy = run_with_input_data("age", &["--decrypt", "--identity"], self.identity.as_bytes(), ciphertext)?;
        if legacy.status.success() {
            return Ok(legacy.stdout);
        }
        Err(VaultError::DecryptionFailed)
    }

    pub fn store_tv_artwork(&self, tvdb_id: u64, content_type: &str, bytes: &[u8]) -> Result<(), VaultError> {
        let path = self.tv_artwork_path(tvdb_id)?;
        store_artwork_with_recipient(&self.recipient, tvdb_id, content_type, bytes, &path)
    }

    pub fn radarr_backup_path(&self) -> PathBuf {
        self.metadata_dir
            .join("radarr")
            .join("recovery.zip.aged")
    }

    pub fn store_radarr_backup(&self, bytes: &[u8]) -> Result<(), VaultError> {
        if bytes.len() < 4
            || bytes.len() > MAX_RADARR_BACKUP_BYTES
            || !bytes.starts_with(b"PK\x03\x04")
        {
            return Err(VaultError::InvalidConfig);
        }
        encrypt_shared_file(&self.recipient, &self.radarr_backup_path(), bytes)
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
    /// `metadata_dir` is the node's /metadata domain, backed by /shared/in/metadata.
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
            if identity_to_recipient(&self.identity)? == existing.trim() {
                encrypt_text_file(&path, format!("{}\n", self.recipient).as_bytes())?;
                return Ok(());
            }
            return Err(VaultError::InvalidIdentity);
        }
        encrypt_text_file(&path, format!("{}\n", self.recipient).as_bytes())?;
        let group_path = self.shared_dir.join("group-age-recipient");
        if !group_path.is_file() {
            encrypt_text_file(&group_path, format!("{}\n", self.recipient).as_bytes())?;
        }
        Ok(())
    }

    pub fn recipient(&self) -> &str {
        &self.recipient
    }

    pub fn user_recipient(&self) -> &str {
        &self.user_recipient
    }

    pub fn remembered_user_recipient() -> Result<Option<String>, VaultError> {
        remembered_user_recipient_at(&local_data_dir().join("age-identity"))
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
        let plaintext = Zeroizing::new(self.decrypt_shared_bytes(&ciphertext)?);
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
            let plaintext = Zeroizing::new(self.decrypt_shared_file(&path)?);
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
        let data_dir = env::var_os("RAWARR_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| shared_dir.join(".node-state"));
        Self::from_identity_in_data_dir(identity, shared_dir, data_dir, local_config_path, local_plaintext_config)
    }

    fn from_identity_in_data_dir(
        identity: String,
        shared_dir: PathBuf,
        data_dir: PathBuf,
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
        let user_recipient = String::from_utf8(output.stdout)
            .map_err(|_| VaultError::InvalidIdentity)?
            .lines()
            .find(|line| line.starts_with("age1pq1"))
            .ok_or(VaultError::InvalidIdentity)?
            .to_owned();
        fs::create_dir_all(&data_dir).map_err(|_| VaultError::StorageFailed)?;
        set_private_directory_permissions(&data_dir)?;
        let group_path = data_dir.join("group-age-identity");
        if !shared_dir.join("group-age-recipient").exists()
            && !group_path.is_file()
        {
            let legacy_path = shared_dir.join("age-recipient");
            if legacy_path.is_file()
                && fs::read_to_string(&legacy_path)
                    .map_err(|_| VaultError::StorageFailed)?
                    .trim() != user_recipient
            {
                // `age-recipient` predates the distinct group identity and may
                // name the key that encrypted the legacy config. Do not treat
                // that public marker as proof that this user identity is wrong:
                // accept the migration only when this identity can actually
                // open a valid config. This check happens before creating any
                // new group-key files, so a failed attempt leaves shared state
                // untouched.
                let config_path = shared_dir.join("config.aged");
                if !config_path.is_file() {
                    return Err(VaultError::InvalidIdentity);
                }
                let output = run_with_input(
                    "age",
                    &["--decrypt", "--identity", "-", path_arg(&config_path)?],
                    identity.as_bytes(),
                )?;
                if !output.status.success() {
                    return Err(VaultError::DecryptionFailed);
                }
                let plaintext = Zeroizing::new(output.stdout);
                if serde_json::from_slice::<PersistedConfig>(plaintext.as_slice()).is_err() {
                    return Err(VaultError::InvalidConfig);
                }
            }
        }
        let (group_identity, recipient) = if group_path.is_file() {
            let group_identity =
                fs::read_to_string(&group_path).map_err(|_| VaultError::StorageFailed)?;
            let group_recipient = identity_to_recipient(&group_identity)
                .map_err(|_| VaultError::GroupIdentityUnavailable)?;
            let published_path = shared_dir.join("group-age-recipient");
            if published_path.is_file() {
                let published =
                    fs::read_to_string(published_path).map_err(|_| VaultError::StorageFailed)?;
                if published.trim() != group_recipient {
                    return Err(VaultError::GroupIdentityUnavailable);
                }
            }
            (group_identity, group_recipient)
        } else if shared_dir.join("group-age-recipient").is_file() {
            let published_path = shared_dir.join("group-age-recipient");
            // A published recipient is not enough to recover the group private
            // identity. Require the user-wrapped envelope; never replace an
            // existing group recipient or encrypted config with a new key.
            let envelope_path = shared_dir.join("group-age-user-envelope.aged");
            let envelope = fs::read(envelope_path).map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    VaultError::GroupIdentityUnavailable
                } else {
                    VaultError::StorageFailed
                }
            })?;
            let output = decrypt_age_input(identity.as_bytes(), &envelope).map_err(|error| {
                if matches!(error, VaultError::DecryptionFailed) {
                    VaultError::GroupIdentityUnavailable
                } else {
                    error
                }
            })?;
            if !output.status.success() {
                return Err(VaultError::GroupIdentityUnavailable);
            }
            let group_identity = String::from_utf8(output.stdout)
                .map_err(|_| VaultError::GroupIdentityUnavailable)?;
            let group_recipient = identity_to_recipient(&group_identity)
                .map_err(|_| VaultError::GroupIdentityUnavailable)?;
            let published =
                fs::read_to_string(published_path).map_err(|_| VaultError::StorageFailed)?;
            if published.trim() != group_recipient {
                return Err(VaultError::GroupIdentityUnavailable);
            }
            persist_private_identity(&group_path, &group_identity)?;
            (group_identity, group_recipient)
        } else {
            // Pre-group-key shared folders can be recovered only when the
            // supplied user identity actually decrypts a valid legacy config.
            // Generate a group key only after that proof, then publish its
            // recipient and user envelope for other devices to install.
            let config_path = shared_dir.join("config.aged");
            if config_path.is_file() {
                let legacy = run_with_input(
                    "age",
                    &["--decrypt", "--identity", "-", path_arg(&config_path)?],
                    identity.as_bytes(),
                )?;
                if !legacy.status.success() {
                    return Err(VaultError::DecryptionFailed);
                }
                let plaintext = Zeroizing::new(legacy.stdout);
                if serde_json::from_slice::<PersistedConfig>(plaintext.as_slice()).is_err() {
                    return Err(VaultError::InvalidConfig);
                }
            }
            let output = Command::new("age-keygen")
                .arg("-pq")
                .output()
                .map_err(|_| VaultError::AgeUnavailable)?;
            if !output.status.success() {
                return Err(VaultError::InvalidIdentity);
            }
            let group_identity = String::from_utf8(output.stdout).map_err(|_| VaultError::InvalidIdentity)?;
            let group_recipient = identity_to_recipient(&group_identity)?;
            persist_private_identity(&group_path, &group_identity)?;
            encrypt_text_file(
                &shared_dir.join("group-age-recipient"),
                format!("{group_recipient}\n").as_bytes(),
            )?;
            (group_identity, group_recipient)
        };
        let user_envelope_path = shared_dir.join("group-age-user-envelope.aged");
        if user_envelope_path.exists() {
            let ciphertext =
                fs::read(&user_envelope_path).map_err(|_| VaultError::StorageFailed)?;
            let opened = decrypt_age_input(identity.as_bytes(), &ciphertext).map_err(|error| {
                if matches!(error, VaultError::DecryptionFailed) {
                    VaultError::GroupIdentityUnavailable
                } else {
                    error
                }
            })?;
            let opened_successfully = opened.status.success();
            let plaintext = Zeroizing::new(opened.stdout);
            if !opened_successfully
                || identity_to_recipient(
                    std::str::from_utf8(&plaintext)
                        .map_err(|_| VaultError::GroupIdentityUnavailable)?,
                )
                .map_err(|_| VaultError::GroupIdentityUnavailable)?
                    != recipient
            {
                return Err(VaultError::GroupIdentityUnavailable);
            }
        } else {
            encrypt_shared_file(&user_recipient, &user_envelope_path, group_identity.as_bytes())?;
        }
        let vault = Self {
            movie_metadata_cache: Default::default(),
            identity,
            user_recipient,
            group_identity,
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
        };
        migrate_legacy_metadata(&vault.shared_dir)?;
        // The first approved node creates the shared config as soon as it
        // installs the user's identity. `write_config` always encrypts with
        // the group recipient, never the user's or device's recipient.
        let peer_repair_configured = env::var("RAWARR_CONFIG_PEERS").ok()
            .and_then(|value| serde_json::from_str::<std::collections::HashMap<String, String>>(&value).ok())
            .is_some_and(|peers| peers.iter().any(|(id, value)| {
                !id.is_empty() && id.len() <= 128 && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
                    && url::Url::parse(value).is_ok_and(|parsed| parsed.scheme() == "https"
                        && parsed.host_str().is_some_and(|host| host.ends_with(".ts.net"))
                        && parsed.username().is_empty() && parsed.password().is_none()
                        && parsed.query().is_none() && parsed.fragment().is_none())
            }));
        if !vault.config_path.is_file() && !peer_repair_configured {
            vault.save(&PersistedConfig {
                device_names: std::collections::HashMap::new(),
                sonarr: None,
                radarr_url: String::new(),
                radarr_api_key: String::new(),
                radarr_ui_username: String::new(),
                radarr_ui_password: String::new(),
                additional_services: Vec::new(),
                needs_migration: false,
                metadata_filename_key: None,
            })?;
        }
        Ok(vault)
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
        let recipient = identity_to_recipient(&identity)?;
        Ok((identity, recipient))
    }

    /// Ensure this subscriber node has a persistent age identity separate from the user's vault.
    /// Existing malformed files are deliberately left untouched for recovery.
    pub fn ensure_node_identity() -> Result<String, VaultError> {
        let data_dir = env::var_os("RAWARR_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data"));
        ensure_node_identity_at(&data_dir.join("node-age-identity"))
    }

    /// Ensure this subscriber has a persistent signing key separate from the user's signing authority.
    /// Existing malformed files are deliberately left untouched for recovery.
    pub fn ensure_node_signing_key() -> Result<(), VaultError> {
        let data_dir = env::var_os("RAWARR_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data"));
        ensure_node_signing_key_at(&data_dir.join("node-signing-key"))
    }

    pub fn node_signing_public_key() -> Result<Vec<u8>, VaultError> {
        let data_dir = env::var_os("RAWARR_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data"));
        let key = load_node_signing_key(&data_dir.join("node-signing-key"))?;
        Ok(key.verifying_key().encode().to_vec())
    }

    pub fn sign_node_message(message: &[u8]) -> Result<Vec<u8>, VaultError> {
        let data_dir = env::var_os("RAWARR_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data"));
        let key = load_node_signing_key(&data_dir.join("node-signing-key"))?;
        Ok(key.sign(message).encode().to_vec())
    }

    pub fn node_identity_recipient() -> Result<String, VaultError> {
        let data_dir = env::var_os("RAWARR_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data"));
        node_identity_recipient(&data_dir.join("node-age-identity"))
    }

    /// Decrypt an age-wrapped user identity addressed to this node's device ID.
    pub fn decrypt_user_identity_envelope(encoded_ciphertext: &str) -> Result<String, VaultError> {
        Self::decrypt_user_identity_envelope_at(encoded_ciphertext, &local_data_dir())
    }

    fn decrypt_user_identity_envelope_at(
        encoded_ciphertext: &str,
        data_dir: &Path,
    ) -> Result<String, VaultError> {
        if encoded_ciphertext.len() > 24 * 1024 {
            return Err(VaultError::InvalidIdentity);
        }
        let ciphertext = BASE64.decode(encoded_ciphertext).map_err(|_| VaultError::InvalidIdentity)?;
        if ciphertext.len() > 16 * 1024 || !ciphertext.starts_with(b"age-encryption.org/v1\n") {
            return Err(VaultError::InvalidIdentity);
        }
        let device_identity = Zeroizing::new(
            fs::read_to_string(data_dir.join("node-age-identity"))
                .map_err(|_| VaultError::StorageFailed)?,
        );
        let output = decrypt_age_input(device_identity.as_bytes(), &ciphertext)?;
        if !output.status.success() {
            return Err(VaultError::InvalidIdentity);
        }
        let identity = String::from_utf8(output.stdout).map_err(|_| VaultError::InvalidIdentity)?;
        Ok(identity)
    }

    /// Install a group identity encrypted specifically for this device.
    pub fn install_group_identity_envelope(ciphertext: &[u8]) -> Result<(), VaultError> {
        Self::install_group_identity_envelope_at(
            ciphertext,
            &local_data_dir(),
            &Self::group_recipient_path(),
        )
    }

    fn install_group_identity_envelope_at(
        ciphertext: &[u8],
        data_dir: &Path,
        group_recipient_path: &Path,
    ) -> Result<(), VaultError> {
        if ciphertext.len() > 16 * 1024 || !ciphertext.starts_with(b"age-encryption.org/v1\n") {
            return Err(VaultError::InvalidIdentity);
        }
        let device_identity = Zeroizing::new(
            fs::read_to_string(data_dir.join("node-age-identity"))
                .map_err(|_| VaultError::StorageFailed)?,
        );
        let output = decrypt_age_input(device_identity.as_bytes(), ciphertext)?;
        if !output.status.success() {
            return Err(VaultError::InvalidIdentity);
        }
        let group_identity = Zeroizing::new(String::from_utf8(output.stdout).map_err(|_| VaultError::InvalidIdentity)?);
        let recipient = identity_to_recipient(&group_identity)?;
        let published = fs::read_to_string(group_recipient_path).map_err(|_| VaultError::StorageFailed)?;
        if published.trim() != recipient {
            return Err(VaultError::InvalidIdentity);
        }
        let path = data_dir.join("group-age-identity");
        if path.exists() {
            let current = Zeroizing::new(fs::read_to_string(&path).map_err(|_| VaultError::StorageFailed)?);
            if identity_to_recipient(&current)? != recipient {
                return Err(VaultError::InvalidIdentity);
            }
            return Ok(());
        }
        persist_private_identity(&path, &group_identity)
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
            self.group_identity.as_bytes(),
        )?;
        let (plaintext, legacy) = if output.status.success() {
            (Zeroizing::new(output.stdout), false)
        } else {
            let legacy = run_with_input(
                "age",
                &["--decrypt", "--identity", "-", path_arg(&self.config_path)?],
                self.identity.as_bytes(),
            )?;
            if !legacy.status.success() {
                return Err(VaultError::DecryptionFailed);
            }
            (Zeroizing::new(legacy.stdout), true)
        };
        let config: PersistedConfig =
            serde_json::from_slice(plaintext.as_slice()).map_err(|_| VaultError::InvalidConfig)?;
        if legacy {
            self.write_config(&config)?;
        }
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
            radarr_ui_username: config.radarr_ui_username.clone(),
            radarr_ui_password: config.radarr_ui_password.clone(),
            metadata_filename_key: existing
                .as_ref()
                .and_then(|stored| stored.metadata_filename_key.clone())
                .or_else(|| config.metadata_filename_key.clone()),
            additional_services: existing.as_ref().map(|stored| stored.additional_services.clone()).unwrap_or_default(),
            needs_migration: false,
        };
        ensure_filename_key(&mut updated)?;
        self.prepare_app_identities(&updated)?;
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
        self.save_service("sonarr", connection)
    }

    /// Update only Prowlarr while preserving all other service entries.
    pub fn save_prowlarr(&self, connection: Option<ArrConnection>) -> Result<(), VaultError> {
        self.save_service("prowlarr", connection)
    }

    fn save_service(
        &self,
        service_type: &str,
        connection: Option<ArrConnection>,
    ) -> Result<(), VaultError> {
        let _guard = CONFIG_WRITE_LOCK
            .lock()
            .map_err(|_| VaultError::StorageFailed)?;
        let mut config = self.read_config()?.unwrap_or(PersistedConfig {
            device_names: std::collections::HashMap::new(),
            radarr_url: String::new(),
            radarr_api_key: String::new(),
            radarr_ui_username: String::new(),
            radarr_ui_password: String::new(),
            sonarr: None,
            metadata_filename_key: None,
            additional_services: Vec::new(),
            needs_migration: false,
        });
        if service_type == "sonarr" {
            config.sonarr = connection;
        } else {
            config
                .additional_services
                .retain(|service| service.service_type != service_type);
            if let Some(connection) = connection {
                config.additional_services.push(ServiceConfig {
                    service_type: service_type.to_owned(),
                    url: connection.url.clone(),
                    api_key: connection.api_key.clone(),
                    ui_username: None,
                    ui_password: None,
                });
            }
        }
        ensure_filename_key(&mut config)?;
        self.prepare_app_identities(&config)?;
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
            config.radarr_ui_username.zeroize();
            config.radarr_ui_username.clear();
            config.radarr_ui_password.zeroize();
            config.radarr_ui_password.clear();
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

fn identity_to_recipient(identity: &str) -> Result<String, VaultError> {
    let output = run_with_input("age-keygen", &["-y"], identity.as_bytes())?;
    if !output.status.success() {
        return Err(VaultError::InvalidIdentity);
    }
    String::from_utf8(output.stdout)
        .map_err(|_| VaultError::InvalidIdentity)?
        .lines()
        .find(|line| is_post_quantum_recipient(line))
        .map(str::to_owned)
        .ok_or(VaultError::InvalidIdentity)
}

fn remembered_user_recipient_at(path: &Path) -> Result<Option<String>, VaultError> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(VaultError::StorageFailed),
    };
    if !metadata.is_file() || metadata.len() > 64 * 1024
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(VaultError::StorageFailed);
    }
    let identity = Zeroizing::new(fs::read_to_string(path).map_err(|_| VaultError::StorageFailed)?);
    identity_to_recipient(&identity).map(Some)
}

fn has_single_post_quantum_identity(identity: &str) -> bool {
    let mut identity_count = 0;
    for line in identity.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with("AGE-SECRET-KEY-PQ-1") {
            identity_count += 1;
        } else if !line.starts_with('#') {
            return false;
        }
    }
    identity_count == 1
}

fn persist_private_identity(path: &Path, identity: &str) -> Result<(), VaultError> {
    use std::os::unix::fs::OpenOptionsExt;
    let parent = path.parent().ok_or(VaultError::StorageFailed)?;
    fs::create_dir_all(parent).map_err(|_| VaultError::StorageFailed)?;
    set_private_directory_permissions(parent)?;
    let temporary = temporary_path(path);
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|_| VaultError::StorageFailed)?;
        file.write_all(identity.as_bytes()).map_err(|_| VaultError::StorageFailed)?;
        file.sync_all().map_err(|_| VaultError::StorageFailed)?;
        match fs::hard_link(&temporary, path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let existing = Zeroizing::new(fs::read_to_string(path).map_err(|_| VaultError::StorageFailed)?);
                if identity_to_recipient(&existing)? == identity_to_recipient(identity)? {
                    Ok(())
                } else {
                    Err(VaultError::InvalidIdentity)
                }
            }
            Err(_) => Err(VaultError::StorageFailed),
        }
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
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

fn ensure_node_identity_at(path: &Path) -> Result<String, VaultError> {
    if path.exists() {
        return node_identity_recipient(path);
    }
    let parent = path.parent().ok_or(VaultError::StorageFailed)?;
    fs::create_dir_all(parent).map_err(|_| VaultError::StorageFailed)?;
    set_private_directory_permissions(parent)?;

    let (identity, recipient) = AgeVault::generate_identity()?;
    let temporary = temporary_path(path);
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| VaultError::StorageFailed)?;
        set_private_file_permissions(&file)?;
        file.write_all(identity.as_bytes())
            .map_err(|_| VaultError::StorageFailed)?;
        file.sync_all().map_err(|_| VaultError::StorageFailed)?;
        match fs::hard_link(&temporary, path) {
            Ok(()) => Ok(recipient),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                node_identity_recipient(path)
            }
            Err(_) => Err(VaultError::StorageFailed),
        }
    })();
    let _ = fs::remove_file(&temporary);
    result
}

fn node_identity_recipient(path: &Path) -> Result<String, VaultError> {
    let identity = Zeroizing::new(fs::read_to_string(path).map_err(|_| VaultError::StorageFailed)?);
    identity_to_recipient(&identity)
}

const NODE_SIGNING_KEY_MAGIC: &[u8] = b"RAWARR-NODE-SIGNING-KEY\0";
const NODE_SIGNING_KEY_VERSION: u8 = 1;
const NODE_SIGNING_SEED_LEN: usize = 32;

fn ensure_node_signing_key_at(path: &Path) -> Result<(), VaultError> {
    if path.exists() {
        return load_node_signing_key(path).map(|_| ());
    }
    let parent = path.parent().ok_or(VaultError::StorageFailed)?;
    fs::create_dir_all(parent).map_err(|_| VaultError::StorageFailed)?;
    set_private_directory_permissions(parent)?;

    let mut seed = Zeroizing::new([0u8; NODE_SIGNING_SEED_LEN]);
    getrandom::fill(&mut *seed).map_err(|_| VaultError::StorageFailed)?;
    // The fixed-size seed derives an ML-DSA-65 key pair.
    let signing_key = SigningKey::<MlDsa65>::from_seed(&(*seed).into());
    drop(signing_key);

    let temporary = temporary_path(path);
    let mut contents = Zeroizing::new(Vec::with_capacity(
        NODE_SIGNING_KEY_MAGIC.len() + 1 + NODE_SIGNING_SEED_LEN,
    ));
    contents.extend_from_slice(NODE_SIGNING_KEY_MAGIC);
    contents.push(NODE_SIGNING_KEY_VERSION);
    contents.extend_from_slice(&*seed);
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| VaultError::StorageFailed)?;
        set_private_file_permissions(&file)?;
        file.write_all(&contents)
            .map_err(|_| VaultError::StorageFailed)?;
        file.sync_all().map_err(|_| VaultError::StorageFailed)?;
        match fs::hard_link(&temporary, path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                load_node_signing_key(path).map(|_| ())
            }
            Err(_) => Err(VaultError::StorageFailed),
        }
    })();
    let _ = fs::remove_file(&temporary);
    result
}

fn load_node_signing_key(path: &Path) -> Result<SigningKey<MlDsa65>, VaultError> {
    let contents = Zeroizing::new(fs::read(path).map_err(|_| VaultError::StorageFailed)?);
    let expected_len = NODE_SIGNING_KEY_MAGIC.len() + 1 + NODE_SIGNING_SEED_LEN;
    if contents.len() != expected_len
        || !contents.starts_with(NODE_SIGNING_KEY_MAGIC)
        || contents[NODE_SIGNING_KEY_MAGIC.len()] != NODE_SIGNING_KEY_VERSION
    {
        return Err(VaultError::InvalidIdentity);
    }
    let seed_start = NODE_SIGNING_KEY_MAGIC.len() + 1;
    let seed: [u8; NODE_SIGNING_SEED_LEN] = contents[seed_start..]
        .try_into()
        .map_err(|_| VaultError::InvalidIdentity)?;
    let seed = Zeroizing::new(seed);
    Ok(SigningKey::<MlDsa65>::from_seed(&(*seed).into()))
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
    override_dir.unwrap_or_else(|| shared_dir.join("in").join("metadata"))
}

fn shared_in_dir_for(shared_dir: &Path) -> PathBuf {
    shared_dir.join("in")
}

fn shared_out_dir_for(shared_dir: &Path) -> PathBuf {
    shared_dir.join("out")
}

fn migrate_legacy_metadata(shared_dir: &Path) -> Result<(), VaultError> {
    let legacy = shared_dir.join("metadata");
    let incoming = shared_in_dir_for(shared_dir);
    // Mounted shared volumes hide the image's directories. Create the alias
    // target even when there is no legacy tree to copy.
    for directory in [&incoming, &incoming.join("metadata")] {
        match fs::symlink_metadata(directory) {
            Ok(metadata) if !metadata.is_dir() => return Err(VaultError::StorageFailed),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir_all(directory).map_err(|_| VaultError::StorageFailed)?;
            }
            Err(_) => return Err(VaultError::StorageFailed),
        }
        set_shared_directory_permissions(directory)?;
    }
    if !legacy.exists() {
        return Ok(());
    }
    let legacy_type = fs::symlink_metadata(&legacy).map_err(|_| VaultError::StorageFailed)?;
    if !legacy_type.is_dir() {
        return Err(VaultError::StorageFailed);
    }
    copy_missing_shared_tree(&legacy, &incoming.join("metadata"))
}

fn copy_missing_shared_tree(source: &Path, target: &Path) -> Result<(), VaultError> {
    if !fs::symlink_metadata(source)
        .map_err(|_| VaultError::StorageFailed)?
        .is_dir()
    {
        return Err(VaultError::StorageFailed);
    }
    if let Ok(target_type) = fs::symlink_metadata(target) {
        if target_type.file_type().is_symlink() || !target_type.is_dir() {
            return Err(VaultError::StorageFailed);
        }
    }
    fs::create_dir_all(target).map_err(|_| VaultError::StorageFailed)?;
    set_shared_directory_permissions(target)?;
    for entry in fs::read_dir(source).map_err(|_| VaultError::StorageFailed)? {
        let entry = entry.map_err(|_| VaultError::StorageFailed)?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let entry_type = entry.file_type().map_err(|_| VaultError::StorageFailed)?;
        if entry_type.is_symlink() {
            return Err(VaultError::StorageFailed);
        }
        if entry_type.is_dir() {
            copy_missing_shared_tree(&source_path, &target_path)?;
            continue;
        }
        if !entry_type.is_file() {
            return Err(VaultError::StorageFailed);
        }
        if let Ok(existing) = fs::symlink_metadata(&target_path) {
            if existing.file_type().is_symlink() || !existing.is_file() {
                return Err(VaultError::StorageFailed);
            }
            // The target is authoritative once the new layout is active.
            // Keep the legacy source untouched when the contents differ.
            continue;
        }
        let temporary = temporary_path(&target_path);
        let copied = (|| {
            fs::copy(&source_path, &temporary).map_err(|_| VaultError::StorageFailed)?;
            set_shared_ciphertext_permissions(&temporary)?;
            fs::File::open(&temporary)
                .and_then(|file| file.sync_all())
                .map_err(|_| VaultError::StorageFailed)?;
            match fs::hard_link(&temporary, &target_path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
                Err(_) => Err(VaultError::StorageFailed),
            }
        })();
        let _ = fs::remove_file(&temporary);
        copied?;
    }
    Ok(())
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
        // Do not try to change permissions on the filesystem root for files
        // stored directly under the shared root (for example, the group key
        // envelope). Nested metadata folders still need Syncthing-readable
        // parent permissions.
        if metadata_directory.parent().is_some() {
            set_shared_directory_permissions(metadata_directory)?;
        }
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
    fn node_identity_is_created_once_and_existing_identity_is_reused() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let data_dir = env::temp_dir().join(format!(
            "rawrarr-node-identity-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&data_dir).expect("create isolated data directory");
        let node_identity = data_dir.join("node-age-identity");
        let user_identity = data_dir.join("age-identity");
        fs::write(&user_identity, "existing user identity fixture").expect("keep user identity");

        let first_recipient =
            ensure_node_identity_at(&node_identity).expect("create node identity");
        let first_identity = fs::read(&node_identity).expect("read generated node identity");
        let second_recipient = ensure_node_identity_at(&node_identity).expect("reuse node identity");

        assert_eq!(first_recipient, second_recipient);
        assert_eq!(
            fs::read(&node_identity).expect("read reused identity"),
            first_identity
        );
        assert_eq!(
            fs::read(&user_identity).expect("read preserved user identity"),
            b"existing user identity fixture"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&node_identity).unwrap().permissions().mode() & 0o777, 0o600);
        }
        fs::remove_dir_all(data_dir).expect("remove isolated data directory");
    }

    #[test]
    fn group_identity_envelope_is_installed_only_for_its_device_recipient() {
        let keygen = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("age-keygen must be installed for this test");
        assert!(keygen.status.success());
        let group_identity = String::from_utf8(keygen.stdout).expect("group identity is UTF-8");
        let group_recipient = identity_to_recipient(&group_identity).expect("derive group recipient");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root = env::temp_dir().join(format!("rawrarr-group-envelope-test-{}-{unique}", std::process::id()));
        let data_dir = root.join("data");
        let shared_dir = root.join("shared");
        fs::create_dir_all(&data_dir).expect("create isolated device data directory");
        fs::create_dir_all(&shared_dir).expect("create isolated shared directory");
        let device_path = data_dir.join("node-age-identity");
        ensure_node_identity_at(&device_path).expect("create device identity");
        let device_identity = Zeroizing::new(fs::read_to_string(&device_path).expect("read device identity"));
        let device_recipient = identity_to_recipient(&device_identity).expect("derive device recipient");
        encrypt_text_file(&shared_dir.join("group-age-recipient"), format!("{group_recipient}\n").as_bytes())
            .expect("publish group recipient");
        let envelope_path = shared_dir.join("group-envelope.aged");
        encrypt_shared_file(&device_recipient, &envelope_path, group_identity.as_bytes())
            .expect("wrap group key for device");
        let envelope = fs::read(envelope_path).expect("read device envelope");

        AgeVault::install_group_identity_envelope_at(
            &envelope,
            &data_dir,
            &shared_dir.join("group-age-recipient"),
        )
        .expect("install group key on intended device");
        let installed = Zeroizing::new(fs::read_to_string(data_dir.join("group-age-identity")).expect("read installed group key"));
        assert_eq!(identity_to_recipient(&installed).unwrap(), group_recipient);
        assert_ne!(device_recipient, group_recipient);

        fs::remove_dir_all(root).expect("remove isolated test tree");
    }

    #[test]
    fn legacy_config_bootstrap_and_per_device_group_key_installation() {
        let first_key = Command::new("age-keygen").arg("-pq").output().expect("age-keygen");
        assert!(first_key.status.success());
        let first_identity = String::from_utf8(first_key.stdout).expect("first user identity");
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let root = env::temp_dir().join(format!("rawrarr-legacy-group-recovery-{}-{unique}", std::process::id()));
        let shared_dir = root.join("shared");
        let first_data = shared_dir.join(".node-state");
        let second_shared_dir = root.join("second-shared");
        let second_data = second_shared_dir.join(".node-state");
        fs::create_dir_all(&shared_dir).expect("shared directory");
        fs::create_dir_all(&first_data).expect("first node data");
        fs::create_dir_all(&second_shared_dir).expect("second shared directory");
        fs::create_dir_all(&second_data).expect("second node data");
        let first_recipient = identity_to_recipient(&first_identity).expect("first recipient");
        ensure_node_identity_at(&first_data.join("node-age-identity")).expect("first device identity");
        encrypt_shared_file(&first_recipient, &shared_dir.join("config.aged"), serde_json::to_string(&PersistedConfig {
            device_names: Default::default(), sonarr: None, radarr_url: String::new(), radarr_api_key: String::new(),
            radarr_ui_username: String::new(), radarr_ui_password: String::new(), additional_services: Vec::new(),
            needs_migration: false, metadata_filename_key: None,
        }).unwrap().as_bytes()).expect("legacy user-encrypted config");

        let first = AgeVault::from_identity_in_data_dir(
            first_identity.clone(), shared_dir.clone(), first_data.clone(), root.join("first-config"), false)
            .expect("original identity bootstraps group key from legacy config");
        let group_recipient = first.recipient().to_owned();
        assert_ne!(group_recipient, first_recipient);
        assert!(shared_dir.join("group-age-user-envelope.aged").is_file());
        assert_eq!(first.load().expect("migrate legacy config").unwrap().radarr_url, "");

        let first_device_identity = fs::read_to_string(first_data.join("node-age-identity")).expect("first device identity");
        let first_device = identity_to_recipient(&first_device_identity).expect("first device recipient");
        let first_device_envelope = root.join("first-device-envelope.aged");
        encrypt_shared_file(&first_device, &first_device_envelope, &fs::read(first_data.join("group-age-identity")).unwrap())
            .expect("wrap group key to first fossil device");
        let first_opened = run_with_input("age", &["--decrypt", "--identity", "-", path_arg(&first_device_envelope).unwrap()],
            first_device_identity.as_bytes()).expect("open first node envelope");
        assert!(first_opened.status.success());
        assert_eq!(identity_to_recipient(std::str::from_utf8(&first_opened.stdout).unwrap()).unwrap(), group_recipient);

        // A second fossil installs its own device-wrapped copy of the same
        // shared group identity.
        ensure_node_identity_at(&second_data.join("node-age-identity")).expect("second device identity");
        let second_device = node_identity_recipient(&second_data.join("node-age-identity")).expect("second device recipient");
        let second_group_identity_path = second_data.join("group-age-identity");
        let wrapped_for_second_device = root.join("second-device-envelope.aged");
        encrypt_shared_file(&second_device, &wrapped_for_second_device, &fs::read(first_data.join("group-age-identity")).unwrap())
            .expect("encrypt group identity to second device");
        let ciphertext = fs::read(&wrapped_for_second_device).unwrap();
        AgeVault::install_group_identity_envelope_at(&ciphertext, &second_data, &shared_dir.join("group-age-recipient"))
            .expect("install same group key on second device");
        assert!(second_group_identity_path.is_file());
        drop(first);
        fs::remove_dir_all(root).expect("remove test files");
    }

    #[test]
    fn missing_group_private_key_never_replaces_published_recipient() {
        let generated = Command::new("age-keygen").arg("-pq").output().expect("age-keygen");
        assert!(generated.status.success());
        let identity = String::from_utf8(generated.stdout).unwrap();
        let unrelated = Command::new("age-keygen").arg("-pq").output().expect("age-keygen");
        assert!(unrelated.status.success());
        let unrelated = String::from_utf8(unrelated.stdout).unwrap();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let root = env::temp_dir().join(format!("rawrarr-missing-group-key-{}-{unique}", std::process::id()));
        let shared = root.join("shared");
        fs::create_dir_all(&shared).unwrap();
        let published = identity_to_recipient(&identity).unwrap();
        encrypt_text_file(&shared.join("group-age-recipient"), format!("{published}\n").as_bytes()).unwrap();
        let before = fs::read(shared.join("group-age-recipient")).unwrap();
        assert!(matches!(
            AgeVault::from_identity_in_dirs(
                unrelated,
                shared.clone(),
                root.join("local-config.json"),
                false,
            ),
            Err(VaultError::GroupIdentityUnavailable)
        ));
        assert_eq!(fs::read(shared.join("group-age-recipient")).unwrap(), before);
        assert!(!shared.join("group-age-user-envelope.aged").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn browser_unlock_identity_is_age_wrapped_to_the_device_identity() {
        let keygen = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("age-keygen must be installed for this test");
        assert!(keygen.status.success());
        let user_identity = String::from_utf8(keygen.stdout).expect("user identity is UTF-8");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let data_dir = env::temp_dir().join(format!("rawrarr-user-envelope-test-{}-{unique}", std::process::id()));
        fs::create_dir_all(&data_dir).expect("create isolated device directory");
        let device_path = data_dir.join("node-age-identity");
        ensure_node_identity_at(&device_path).expect("create device age identity");
        let device_identity = Zeroizing::new(fs::read_to_string(&device_path).expect("read device identity"));
        let device_recipient = identity_to_recipient(&device_identity).expect("derive device recipient");
        let envelope_path = data_dir.join("user-identity.aged");
        encrypt_shared_file(&device_recipient, &envelope_path, user_identity.as_bytes())
            .expect("encrypt user age identity to the device");
        let ciphertext = fs::read(envelope_path).expect("read device-encrypted user identity");
        let encoded = BASE64.encode(ciphertext);

        let opened = AgeVault::decrypt_user_identity_envelope_at(&encoded, &data_dir)
            .expect("device opens its encrypted request");
        assert_eq!(opened, user_identity);

        fs::remove_dir_all(data_dir).expect("remove isolated device directory");
    }

    #[test]
    fn invalid_existing_node_identity_is_not_replaced() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let data_dir = env::temp_dir().join(format!(
            "rawrarr-invalid-node-identity-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&data_dir).expect("create isolated data directory");
        let path = data_dir.join("node-age-identity");
        fs::write(&path, "not an age identity").expect("write invalid fixture");

        assert!(ensure_node_identity_at(&path).is_err());
        assert_eq!(
            fs::read(&path).expect("read preserved invalid identity"),
            b"not an age identity"
        );
        fs::remove_dir_all(data_dir).expect("remove isolated data directory");
    }

    #[test]
    fn node_signing_key_is_created_once_and_reused_separately_from_user_identity() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let data_dir = env::temp_dir().join(format!(
            "rawrarr-node-signing-key-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&data_dir).expect("create isolated data directory");
        let node_key = data_dir.join("node-signing-key");
        let user_identity = data_dir.join("age-identity");
        fs::write(&user_identity, "user identity fixture").expect("write user identity fixture");

        ensure_node_signing_key_at(&node_key).expect("create node signing key");
        let first_key = fs::read(&node_key).expect("read generated node signing key");
        let loaded_key = load_node_signing_key(&node_key).expect("load generated node signing key");
        use ml_dsa::signature::Keypair;
        let first_public_key = loaded_key.verifying_key().encode();

        ensure_node_signing_key_at(&node_key).expect("reuse node signing key");
        let reused_key = load_node_signing_key(&node_key).expect("load reused node signing key");
        assert_eq!(
            reused_key.verifying_key().encode(),
            first_public_key,
            "the same public key must be derived after restart"
        );
        assert_eq!(fs::read(&node_key).expect("read reused key"), first_key);
        assert_eq!(
            fs::read(&user_identity).expect("read preserved user identity"),
            b"user identity fixture"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&node_key).unwrap().permissions().mode() & 0o777, 0o600);
        }
        fs::remove_dir_all(data_dir).expect("remove isolated data directory");
    }

    #[test]
    fn invalid_existing_node_signing_key_is_not_replaced() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let data_dir = env::temp_dir().join(format!(
            "rawrarr-invalid-node-signing-key-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&data_dir).expect("create isolated data directory");
        let path = data_dir.join("node-signing-key");
        fs::write(&path, b"invalid signing key fixture").expect("write invalid fixture");

        assert!(ensure_node_signing_key_at(&path).is_err());
        assert_eq!(
            fs::read(&path).expect("read preserved invalid key"),
            b"invalid signing key fixture"
        );
        fs::remove_dir_all(data_dir).expect("remove isolated data directory");
    }

    #[test]
    fn metadata_domain_defaults_to_shared_subtree_and_accepts_container_alias() {
        let shared = PathBuf::from("/shared");
        assert_eq!(
            metadata_dir_for(&shared, None),
            PathBuf::from("/shared/in/metadata")
        );
        assert_eq!(
            metadata_dir_for(&shared, Some(PathBuf::from("/metadata"))),
            PathBuf::from("/metadata")
        );
    }

    #[test]
    fn shared_tree_exposes_in_and_out_subtrees() {
        let shared = PathBuf::from("/shared");
        assert_eq!(shared.join("in"), shared_in_dir_for(&shared));
        assert_eq!(shared.join("out"), shared_out_dir_for(&shared));
    }

    #[test]
    fn legacy_metadata_migration_copies_without_removing_or_overwriting() {
        let root = env::temp_dir().join(format!(
            "rawrarr-shared-in-migration-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        let legacy = root.join("metadata");
        let target = root.join("in/metadata");
        fs::create_dir_all(legacy.join("movies/text")).unwrap();
        fs::write(legacy.join("movies/text/movie.aged"), b"ciphertext").unwrap();

        migrate_legacy_metadata(&root).unwrap();
        assert_eq!(fs::read(target.join("movies/text/movie.aged")).unwrap(), b"ciphertext");
        assert_eq!(fs::read(legacy.join("movies/text/movie.aged")).unwrap(), b"ciphertext");
        migrate_legacy_metadata(&root).unwrap();

        fs::write(target.join("movies/text/movie.aged"), b"different ciphertext").unwrap();
        migrate_legacy_metadata(&root).unwrap();
        assert_eq!(fs::read(legacy.join("movies/text/movie.aged")).unwrap(), b"ciphertext");
        assert_eq!(fs::read(target.join("movies/text/movie.aged")).unwrap(), b"different ciphertext");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prepared_app_identities_are_group_encrypted_without_changing_config_or_metadata() {
        let keygen = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("age-keygen must be installed for this test");
        assert!(keygen.status.success());
        let user_identity = String::from_utf8(keygen.stdout).expect("identity is UTF-8");
        let unique = uuid::Uuid::new_v4();
        let root = env::temp_dir().join(format!("rawrarr-app-identity-test-{unique}"));
        let shared_dir = root.join("shared");
        fs::create_dir_all(&shared_dir).expect("create isolated shared directory");
        let vault = AgeVault::from_identity_in_dirs(
            user_identity.clone(),
            shared_dir.clone(),
            root.join("radarr-config.json"),
            false,
        )
        .expect("open isolated age vault");
        let empty_config = PersistedConfig {
            device_names: std::collections::HashMap::new(),
            sonarr: None,
            radarr_url: String::new(),
            radarr_api_key: String::new(),
            radarr_ui_username: String::new(),
            radarr_ui_password: String::new(),
            additional_services: Vec::new(),
            needs_migration: false,
            metadata_filename_key: Some("11".repeat(32)),
        };
        vault.save(&empty_config).expect("save empty config");
        vault
            .store_movie_metadata(12345, br#"{"schemaVersion":1,"title":"Existing"}"#)
            .expect("write existing group-encrypted metadata");
        let metadata_path = vault.movie_metadata_path(12345).expect("metadata path");
        let existing_metadata_ciphertext =
            fs::read(&metadata_path).expect("existing metadata ciphertext");

        let legacy_config_path = shared_dir.join("config.aged");
        let existing_radarr_config = serde_json::json!({
            "schema_version": 1,
            "device_names": {},
            "services": [{
                "type": "radarr",
                "url": "http://radarr.example:7878",
                "api_key": "disposable Radarr fixture"
            }],
            "metadata_filename_key": "11".repeat(32)
        });
        encrypt_shared_file(
            &vault.recipient,
            &legacy_config_path,
            &serde_json::to_vec(&existing_radarr_config).unwrap(),
        )
        .expect("write pre-identity v1 config");
        vault.load().expect("load existing v1 config");
        assert!(
            !vault.prepared_app_identities_path().exists(),
            "unlocking an old config must not initialize shared identities"
        );

        let radarr_config = PersistedConfig {
            device_names: std::collections::HashMap::new(),
            sonarr: None,
            radarr_url: "http://radarr.example:7878".into(),
            radarr_api_key: "disposable Radarr fixture".into(),
            radarr_ui_username: String::new(),
            radarr_ui_password: String::new(),
            additional_services: Vec::new(),
            needs_migration: false,
            metadata_filename_key: Some("11".repeat(32)),
        };
        vault.save(&radarr_config).expect("save Radarr settings");
        vault
            .save_sonarr(Some(ArrConnection {
                url: "http://sonarr.example:8989".into(),
                api_key: "disposable Sonarr fixture".into(),
            }))
            .expect("save Sonarr settings");

        let registry = vault
            .read_prepared_app_identities()
            .expect("read prepared identities")
            .expect("identity registry exists");
        let radarr = registry.apps.get("radarr").expect("Radarr identity");
        let sonarr = registry.apps.get("sonarr").expect("Sonarr identity");
        assert_ne!(radarr.instance_id, sonarr.instance_id);
        assert_ne!(radarr.recipient, sonarr.recipient);
        assert_ne!(radarr.recipient, vault.recipient);
        assert_ne!(sonarr.recipient, vault.recipient);
        let radarr_id = radarr.instance_id.clone();
        let sonarr_id = sonarr.instance_id.clone();
        let radarr_identity = Zeroizing::new(radarr.identity.clone());
        let sonarr_identity = Zeroizing::new(sonarr.identity.clone());
        let registry_ciphertext = fs::read(vault.prepared_app_identities_path())
            .expect("read encrypted app identity registry");
        assert!(registry_ciphertext.starts_with(b"age-encryption.org/v1\n"));
        assert!(!registry_ciphertext
            .windows(radarr.identity.len())
            .any(|window| window == radarr.identity.as_bytes()));
        assert!(!registry_ciphertext
            .windows(sonarr.identity.len())
            .any(|window| window == sonarr.identity.as_bytes()));
        drop(registry);

        let group_opened = run_with_input_data(
            "age",
            &["--decrypt", "--identity"],
            vault.group_identity.as_bytes(),
            &registry_ciphertext,
        )
        .expect("decrypt app identities with group identity");
        assert!(group_opened.status.success());
        let user_opened = run_with_input_data(
            "age",
            &["--decrypt", "--identity"],
            user_identity.as_bytes(),
            &registry_ciphertext,
        )
        .expect("try user identity against group-encrypted app identities");
        assert!(!user_opened.status.success());

        let radarr_opened = run_with_input_data(
            "age",
            &["--decrypt", "--identity"],
            radarr_identity.as_bytes(),
            &existing_metadata_ciphertext,
        )
        .expect("try app identity against legacy metadata");
        let sonarr_opened = run_with_input_data(
            "age",
            &["--decrypt", "--identity"],
            sonarr_identity.as_bytes(),
            &existing_metadata_ciphertext,
        )
        .expect("try second app identity against legacy metadata");
        assert!(!radarr_opened.status.success());
        assert!(!sonarr_opened.status.success());
        assert_eq!(
            fs::read(&metadata_path).expect("metadata ciphertext after preparation"),
            existing_metadata_ciphertext,
            "preparation must not rewrite existing metadata"
        );
        assert_eq!(
            vault
                .load_movie_metadata(12345)
                .expect("read unchanged group-encrypted metadata")
                .expect("existing metadata remains readable")["title"],
            "Existing"
        );

        let config_path = shared_dir.join("config.aged");
        let encrypted_config = fs::read(&config_path).expect("read v1 config ciphertext");
        let opened_config = run_with_input(
            "age",
            &["--decrypt", "--identity", "-", path_arg(&config_path).unwrap()],
            vault.group_identity.as_bytes(),
        )
        .expect("decrypt v1 config");
        assert!(opened_config.status.success());
        serde_json::from_slice::<PersistedConfig>(&opened_config.stdout)
            .expect("the existing v1 reader accepts config after preparation");
        let config_json: serde_json::Value =
            serde_json::from_slice(&opened_config.stdout).expect("parse v1 config");
        assert_eq!(config_json["schema_version"], 1);
        assert!(config_json.get("appIdentities").is_none());
        assert!(!encrypted_config.is_empty());

        vault.save(&radarr_config).expect("repeat Radarr save");
        vault
            .save_sonarr(Some(ArrConnection {
                url: "http://sonarr.example:8989".into(),
                api_key: "disposable Sonarr fixture".into(),
            }))
            .expect("repeat Sonarr save");
        vault.save_sonarr(None).expect("clear Sonarr connection");
        vault.clear().expect("clear Radarr connection");
        let retained = vault
            .read_prepared_app_identities()
            .expect("read retained app identities")
            .expect("retained registry exists");
        assert_eq!(retained.apps["radarr"].instance_id, radarr_id);
        assert_eq!(retained.apps["sonarr"].instance_id, sonarr_id);
        fs::remove_dir_all(root).expect("remove isolated test data");
    }

    #[test]
    fn malformed_prepared_app_identity_registry_is_not_replaced() {
        let keygen = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("age-keygen must be installed for this test");
        assert!(keygen.status.success());
        let identity = String::from_utf8(keygen.stdout).expect("identity is UTF-8");
        let root = env::temp_dir().join(format!(
            "rawrarr-invalid-app-identity-test-{}",
            uuid::Uuid::new_v4()
        ));
        let shared_dir = root.join("shared");
        fs::create_dir_all(&shared_dir).expect("create isolated shared directory");
        let vault = AgeVault::from_identity_in_dirs(
            identity,
            shared_dir.clone(),
            root.join("radarr-config.json"),
            false,
        )
        .expect("open isolated age vault");
        let invalid_identity_registry = PreparedAppIdentityRegistry {
            schema_version: 1,
            apps: [(
                "radarr".to_owned(),
                PreparedAppIdentity {
                    instance_id: uuid::Uuid::new_v4().to_string(),
                    identity: "not an age identity".to_owned(),
                    recipient: vault.recipient.clone(),
                },
            )]
            .into_iter()
            .collect(),
        };
        assert!(matches!(
            AgeVault::validate_prepared_app_identities(&invalid_identity_registry),
            Err(VaultError::InvalidConfig)
        ));
        let path = vault.prepared_app_identities_path();
        encrypt_shared_file(&vault.recipient, &path, br#"{"schemaVersion":99,"apps":{}}"#)
            .expect("write unsupported registry");
        let original_ciphertext = fs::read(&path).expect("read malformed registry ciphertext");
        let config = PersistedConfig {
            device_names: std::collections::HashMap::new(),
            sonarr: None,
            radarr_url: "http://radarr.example:7878".into(),
            radarr_api_key: "disposable Radarr fixture".into(),
            radarr_ui_username: String::new(),
            radarr_ui_password: String::new(),
            additional_services: Vec::new(),
            needs_migration: false,
            metadata_filename_key: None,
        };
        assert!(matches!(
            vault.save(&config),
            Err(VaultError::InvalidConfig)
        ));
        assert_eq!(
            fs::read(path).expect("read retained malformed registry"),
            original_ciphertext
        );
        fs::remove_dir_all(root).expect("remove isolated test data");
    }

    #[test]
    fn radarr_recovery_archive_is_age_encrypted_under_shared_metadata() {
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
            "rawrarr-radarr-backup-test-{}-{unique}",
            std::process::id()
        ));
        let vault = AgeVault::from_identity_in_dirs(
            identity.clone(),
            shared_dir.clone(),
            shared_dir.join("local.json"),
            false,
        )
        .expect("valid PQ age identity");
        vault
            .persist_public_recipient()
            .expect("persist shared recipient");
        let archive = b"PK\x03\x04private-radarr-config-fixture";
        vault
            .store_radarr_backup(archive)
            .expect("store encrypted backup");

        let path = shared_dir.join("in/metadata/radarr/recovery.zip.aged");
        let ciphertext = fs::read(&path).expect("read shared backup ciphertext");
        assert!(ciphertext.starts_with(b"age-encryption.org/v1\n"));
        assert!(!ciphertext
            .windows(archive.len())
            .any(|window| window == archive));
        let decrypted = run_with_input(
            "age",
            &["--decrypt", "--identity", "-", path_arg(&path).unwrap()],
            vault.group_identity.as_bytes(),
        )
        .expect("decrypt backup fixture");
        assert!(decrypted.status.success());
        assert_eq!(decrypted.stdout, archive);
        assert!(matches!(
            vault.store_radarr_backup(b"not a zip"),
            Err(VaultError::InvalidConfig)
        ));
        fs::remove_dir_all(shared_dir).expect("remove isolated test data");
    }

    #[test]
    fn legacy_recipient_mismatch_migrates_only_when_user_identity_opens_config() {
        let user_keygen = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("age-keygen must be installed for this test");
        assert!(user_keygen.status.success());
        let user_identity = String::from_utf8(user_keygen.stdout).expect("user identity is UTF-8");
        let user_recipient = identity_to_recipient(&user_identity).expect("derive user recipient");

        let marker_keygen = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("age-keygen must be installed for this test");
        assert!(marker_keygen.status.success());
        let marker_identity = String::from_utf8(marker_keygen.stdout).expect("marker identity is UTF-8");
        let stale_recipient = identity_to_recipient(&marker_identity).expect("derive legacy marker");
        assert_ne!(stale_recipient, user_recipient);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root = env::temp_dir().join(format!(
            "rawrarr-legacy-recipient-migration-test-{}-{unique}",
            std::process::id()
        ));
        let shared_dir = root.join("shared");
        let data_dir = root.join("data");
        fs::create_dir_all(&shared_dir).expect("create isolated shared directory");
        fs::create_dir_all(&data_dir).expect("create isolated data directory");
        fs::write(shared_dir.join("age-recipient"), format!("{stale_recipient}\n"))
            .expect("write legacy public recipient marker");
        let config_path = shared_dir.join("config.aged");
        encrypt_shared_file(
            &user_recipient,
            &config_path,
            br#"{"schema_version":1,"services":[]}"#,
        )
        .expect("encrypt legacy config to user identity");

        let vault = AgeVault::from_identity_in_dirs(
            user_identity.clone(),
            shared_dir.clone(),
            data_dir.join("radarr-config.json"),
            false,
        )
        .expect("accept identity proven by encrypted config");
        assert!(vault.load().expect("migrate readable config").is_some());
        assert!(shared_dir.join("group-age-recipient").is_file());
        assert!(shared_dir.join("group-age-user-envelope.aged").is_file());

        let rejected_root = root.join("rejected");
        let rejected_shared = rejected_root.join("shared");
        let rejected_data = rejected_root.join("data");
        fs::create_dir_all(&rejected_shared).expect("create rejected shared directory");
        fs::create_dir_all(&rejected_data).expect("create rejected data directory");
        fs::write(rejected_shared.join("age-recipient"), format!("{stale_recipient}\n"))
            .expect("write second legacy marker");
        let rejected_config = rejected_shared.join("config.aged");
        encrypt_shared_file(
            &user_recipient,
            &rejected_config,
            br#"{"schema_version":1,"services":[]}"#,
        )
        .expect("encrypt second legacy config");
        let wrong_keygen = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("age-keygen must be installed for this test");
        assert!(wrong_keygen.status.success());
        let wrong_identity = String::from_utf8(wrong_keygen.stdout).expect("wrong identity is UTF-8");
        assert!(matches!(
            AgeVault::from_identity_in_dirs(
                wrong_identity,
                rejected_shared.clone(),
                rejected_data.join("radarr-config.json"),
                false,
            ),
            Err(VaultError::DecryptionFailed)
        ));
        assert!(!rejected_shared.join("group-age-recipient").exists());
        assert!(!rejected_shared.join("group-age-user-envelope.aged").exists());
        assert!(!rejected_data.join("group-age-identity").exists());

        fs::remove_dir_all(root).expect("remove isolated test directories");
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
            identity.clone(),
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
        assert_ne!(identity_to_recipient(&identity).unwrap(), vault.recipient());
        let user_envelope = fs::read(shared_dir.join("group-age-user-envelope.aged"))
            .expect("read user-wrapped group identity");
        let recovered_group = run_with_input_data(
            "age",
            &["--decrypt", "--identity"],
            identity.as_bytes(),
            &user_envelope,
        )
        .expect("decrypt group identity envelope as the user");
        assert!(recovered_group.status.success());
        assert_eq!(recovered_group.stdout, vault.group_identity.as_bytes());
        let initial_config_path = shared_dir.join("config.aged");
        let initial_ciphertext = fs::read(&initial_config_path).expect("first unlock creates shared config");
        assert!(initial_ciphertext.starts_with(b"age-encryption.org/v1\n"));
        let initial_plaintext = run_with_input(
            "age",
            &["--decrypt", "--identity", "-", path_arg(&initial_config_path).unwrap()],
            vault.group_identity.as_bytes(),
        ).expect("group identity decrypts initial config");
        assert!(initial_plaintext.status.success());
        assert!(serde_json::from_slice::<serde_json::Value>(&initial_plaintext.stdout).is_ok());
        let user_decryption = run_with_input(
            "age",
            &["--decrypt", "--identity", "-", path_arg(&initial_config_path).unwrap()],
            identity.as_bytes(),
        ).expect("attempt user identity decryption");
        assert!(!user_decryption.status.success(), "config must target the group recipient");
        let config = PersistedConfig {
            device_names: std::collections::HashMap::new(),
            sonarr: None,
            radarr_url: "http://radarr.example:7878".into(),
            radarr_api_key: "disposable encrypted config fixture".into(),
            radarr_ui_username: "radarr-user-fixture".into(),
            radarr_ui_password: "disposable radarr password fixture".into(),
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
        assert!(
            !encrypted_config
                .windows(config.radarr_ui_username.len())
                .any(|window| window == config.radarr_ui_username.as_bytes())
        );
        assert!(
            !encrypted_config
                .windows(config.radarr_ui_password.len())
                .any(|window| window == config.radarr_ui_password.as_bytes())
        );
        let local_config = fs::read(&local_config_path).expect("read node-local config mirror");
        assert!(!local_config
            .windows(config.radarr_ui_username.len())
            .any(|window| window == config.radarr_ui_username.as_bytes()));
        assert!(!local_config
            .windows(config.radarr_ui_password.len())
            .any(|window| window == config.radarr_ui_password.as_bytes()));
        assert_eq!(
            serde_json::from_slice::<PersistedConfig>(&local_config)
                .expect("parse local config mirror")
                .radarr_api_key,
            config.radarr_api_key
        );
        assert!(!shared_dir.join("radarr-config.json").exists());
        let loaded = vault.load().expect("decrypt config").unwrap();
        assert_eq!(loaded.radarr_url, config.radarr_url);
        assert_eq!(loaded.radarr_ui_username, config.radarr_ui_username);
        assert_eq!(loaded.radarr_ui_password, config.radarr_ui_password);
        drop(loaded);
        let prowlarr_key = "disposable encrypted Prowlarr fixture";
        vault
            .save_prowlarr(Some(ArrConnection {
                url: "https://prowlarr.example.test".to_owned(),
                api_key: prowlarr_key.to_owned(),
            }))
            .expect("save encrypted Prowlarr settings");
        let encrypted_config = fs::read(&config_path).expect("read Prowlarr ciphertext");
        assert!(!encrypted_config
            .windows(prowlarr_key.len())
            .any(|window| window == prowlarr_key.as_bytes()));
        let loaded = vault.load().expect("load Prowlarr config").unwrap();
        let prowlarr = loaded
            .additional_services
            .iter()
            .find(|service| service.service_type == "prowlarr")
            .expect("Prowlarr config is present");
        assert_eq!(prowlarr.url, "https://prowlarr.example.test");
        assert_eq!(prowlarr.api_key, prowlarr_key);
        assert_eq!(loaded.radarr_api_key, config.radarr_api_key);
        drop(loaded);
        vault
            .save_prowlarr(None)
            .expect("clear only Prowlarr settings");
        let loaded = vault.load().expect("load config after clearing Prowlarr").unwrap();
        assert!(loaded
            .additional_services
            .iter()
            .all(|service| service.service_type != "prowlarr"));
        assert_eq!(loaded.radarr_api_key, config.radarr_api_key);
        drop(loaded);
        assert!(vault.config_exists());

        let wrong_identity_output = Command::new("age-keygen")
            .arg("-pq")
            .output()
            .expect("generate unrelated PQ age identity");
        assert!(wrong_identity_output.status.success());
        let wrong_identity =
            String::from_utf8(wrong_identity_output.stdout).expect("wrong identity is UTF-8");
        assert!(matches!(
            AgeVault::from_identity_in_shared_dir(wrong_identity, shared_dir.clone()),
            Err(VaultError::GroupIdentityUnavailable)
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
                fs::metadata(shared_dir.join("in/metadata"))
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
        let sidecar_directory = shared_dir.join("in/metadata/library/text");
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
            vault.group_identity.as_bytes(),
        )
        .expect("decrypt media JSON sidecar");
        assert!(decrypted_sidecar.status.success());
        assert_eq!(decrypted_sidecar.stdout, sidecar_json);
        let user_cannot_decrypt = run_with_input_data(
            "age",
            &["--decrypt", "--identity"],
            identity.as_bytes(),
            &encrypted_sidecar,
        )
        .expect("run age with the separate user identity");
        assert!(!user_cannot_decrypt.status.success(), "shared records must target the group recipient");

        vault.clear().expect("clear encrypted and local config");
        assert!(config_path.exists());
        let cleared = vault.load().unwrap().unwrap();
        assert!(cleared.radarr_url.is_empty());
        assert!(cleared.radarr_api_key.is_empty());
        assert!(cleared.radarr_ui_username.is_empty());
        assert!(cleared.radarr_ui_password.is_empty());
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
#[path = "../../scripts/tests/test_rawrarr_metadata_storage.rs"]
mod metadata_storage_tests;

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

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_owner_recipient.rs"]
mod owner_recipient_tests;
