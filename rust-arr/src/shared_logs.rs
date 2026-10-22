//! Structured, compressed operational events stored in the Syncthing-shared app-data folder.
//!
//! Only fixed event names and numeric scan summaries are written. Request data, URLs,
//! credentials, media paths, and user-provided strings are deliberately excluded.

use flate2::{Compression, write::GzEncoder};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const MAX_COMPRESSED_PART_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone)]
pub struct SharedLogger(Arc<Mutex<LogWriter>>);

struct LogWriter {
    directory: PathBuf,
    device_id: String,
    node_id: String,
    role: String,
    certificate: Option<serde_json::Value>,
    session_id: String,
    part: u32,
    public_key: Vec<u8>,
    sign: Arc<dyn Fn(&[u8]) -> io::Result<Vec<u8>> + Send + Sync>,
}

#[derive(Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignedLogRecord {
    version: u32,
    algorithm: String,
    public_key_hex: String,
    fingerprint: String,
    payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    certificate: Option<serde_json::Value>,
    signature_hex: String,
}

#[derive(Serialize)]
struct LogRecord<'a> {
    timestamp_unix_seconds: u64,
    device_id: &'a str,
    node_id: &'a str,
    role: &'a str,
    event: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    scanned_files: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sidecars_created: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sidecar_errors: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scan_issues: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scan_truncated: Option<bool>,
}

impl SharedLogger {
    /// Create a logger under `<shared_dir>/logs/<device_id>/`.
    pub fn open(
        shared_dir: &Path,
        device_id: &str,
        node_id: &str,
        role: &str,
        certificate: Option<serde_json::Value>,
        public_key: Vec<u8>,
        sign: impl Fn(&[u8]) -> io::Result<Vec<u8>> + Send + Sync + 'static,
    ) -> io::Result<Self> {
        if device_id.is_empty()
            || !device_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid device ID for shared logging",
            ));
        }

        let directory = shared_dir.join("logs").join(device_id);
        fs::create_dir_all(&directory)?;
        Ok(Self(Arc::new(Mutex::new(LogWriter {
            directory,
            device_id: device_id.to_owned(),
            node_id: node_id.to_owned(),
            role: role.to_owned(),
            certificate,
            session_id: Uuid::new_v4().to_string(),
            part: 0,
            public_key,
            sign: Arc::new(sign),
        }))))
    }

    pub fn application_started(&self) {
        self.record_or_warn("application.started", None);
    }

    pub fn scan_completed(
        &self,
        scanned_files: usize,
        sidecars_created: usize,
        sidecar_errors: usize,
        scan_issues: usize,
        scan_truncated: bool,
    ) {
        self.record_or_warn(
            "library.scan.completed",
            Some((
                scanned_files,
                sidecars_created,
                sidecar_errors,
                scan_issues,
                scan_truncated,
            )),
        );
    }

    fn record_or_warn(
        &self,
        event: &'static str,
        scan: Option<(usize, usize, usize, usize, bool)>,
    ) {
        if self.write_record(event, scan).is_err() {
            eprintln!("rawrarr could not append a compressed shared log record");
        }
    }

    fn write_record(
        &self,
        event: &'static str,
        scan: Option<(usize, usize, usize, usize, bool)>,
    ) -> io::Result<()> {
        let mut writer = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let record = LogRecord {
            timestamp_unix_seconds: now,
            device_id: &writer.device_id,
            node_id: &writer.node_id,
            role: &writer.role,
            event,
            scanned_files: scan.map(|values| values.0),
            sidecars_created: scan.map(|values| values.1),
            sidecar_errors: scan.map(|values| values.2),
            scan_issues: scan.map(|values| values.3),
            scan_truncated: scan.map(|values| values.4),
        };

        let payload = serde_json::to_value(&record).map_err(io::Error::other)?;
        let message = crate::device_signing::canonical_json(&payload).map_err(io::Error::other)?;
        let signature = (writer.sign)(&message)?;
        let envelope = SignedLogRecord {
            version: 1,
            algorithm: crate::device_signing::ALGORITHM.to_owned(),
            public_key_hex: encode_hex(&writer.public_key),
            fingerprint: crate::device_signing::node_fingerprint(&writer.public_key),
            payload,
            certificate: writer.certificate.clone(),
            signature_hex: encode_hex(&signature),
        };
        let compressed = compress_record(&envelope)?;
        let mut path = writer.log_path();
        if path.metadata().map(|metadata| metadata.len()).unwrap_or(0) > 0
            && path
                .metadata()
                .map(|metadata| {
                    metadata.len().saturating_add(compressed.len() as u64)
                        > MAX_COMPRESSED_PART_BYTES
                })
                .unwrap_or(false)
        {
            writer.part = writer.part.saturating_add(1);
            path = writer.log_path();
        }

        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o644);
        }
        let mut file = options.open(path)?;
        file.write_all(&compressed)?;
        file.flush()
    }
}

impl LogWriter {
    fn log_path(&self) -> PathBuf {
        self.directory.join(format!(
            "rawrarr-{}.{}.jsonl.gz",
            self.session_id, self.part
        ))
    }
}

fn compress_record(record: &SignedLogRecord) -> io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    serde_json::to_writer(&mut encoder, record).map_err(io::Error::other)?;
    encoder.write_all(b"\n")?;
    encoder.finish()
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 15) as usize] as char);
    }
    result
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() % 2 != 0 { return None; }
    value.as_bytes().chunks_exact(2).map(|pair| {
        let digit = |byte: u8| match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        };
        Some(digit(pair[0])? * 16 + digit(pair[1])?)
    }).collect()
}

/// Verifies integrity. Callers must compare the key fingerprint with an
/// independently trusted enrollment record to establish who signed it.
pub fn verify_signed_record(record: &serde_json::Value) -> bool {
    let Ok(envelope) = serde_json::from_value::<SignedLogRecord>(record.clone()) else { return false; };
    if envelope.version != 1 || envelope.algorithm != crate::device_signing::ALGORITHM { return false; }
    let Some(public_key) = decode_hex(&envelope.public_key_hex) else { return false; };
    let Some(signature) = decode_hex(&envelope.signature_hex) else { return false; };
    if crate::device_signing::node_fingerprint(&public_key) != envelope.fingerprint { return false; }
    let Ok(message) = crate::device_signing::canonical_json(&envelope.payload) else { return false; };
    crate::device_signing::verify(&public_key, &signature, &message)
}

/// Verifies both the event signature and its root-signed device certificate.
/// Supply the root fingerprint saved independently during enrollment.
pub fn verify_trusted_signed_record(record: &serde_json::Value, expected_root_fingerprint: &str) -> bool {
    if !verify_signed_record(record) { return false; }
    let Ok(envelope) = serde_json::from_value::<SignedLogRecord>(record.clone()) else { return false; };
    let Some(certificate) = envelope.certificate else { return false; };
    if !crate::device_signing::verify_certificate(&certificate, expected_root_fingerprint) { return false; }
    let Some(certificate_key) = certificate.get("signingPublicKeyHex").and_then(serde_json::Value::as_str) else { return false; };
    if certificate_key != envelope.public_key_hex { return false; }
    let Some(payload_node_id) = envelope.payload.get("node_id").and_then(serde_json::Value::as_str) else { return false; };
    let Some(payload_role) = envelope.payload.get("role").and_then(serde_json::Value::as_str) else { return false; };
    let Some(attested) = certificate.get("signedPayload") else { return false; };
    attested.get("nodeId").and_then(serde_json::Value::as_str) == Some(payload_node_id)
        && attested.get("role").and_then(serde_json::Value::as_str) == Some(payload_role)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::MultiGzDecoder;
    use std::{fs::File, io::Read};
    use ml_dsa::{MlDsa65, SigningKey, signature::{Keypair, Signer}};
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

    #[test]
    fn writes_appendable_gzip_members_with_only_allowlisted_fields() {
        let root = std::env::temp_dir().join(format!("rawrarr-logs-{}", Uuid::new_v4()));
        let key = SigningKey::<MlDsa65>::from_seed(&[19u8; 32].into());
        let public_key = key.verifying_key().encode().to_vec();
        let logger = SharedLogger::open(&root, "test-device", "device-fixture", "metadata", None, public_key, move |message| {
            Ok(key.sign(message).encode().to_vec())
        }).expect("logger opens");
        logger.application_started();
        logger.scan_completed(12, 11, 1, 0, false);

        let device_dir = root.join("logs/test-device");
        let files = fs::read_dir(&device_dir)
            .expect("log directory exists")
            .collect::<Result<Vec<_>, _>>()
            .expect("directory entries are readable");
        assert_eq!(files.len(), 1);
        let mut decoder = MultiGzDecoder::new(File::open(files[0].path()).expect("log opens"));
        let mut output = String::new();
        decoder
            .read_to_string(&mut output)
            .expect("gzip members decode");
        assert_eq!(output.lines().count(), 2);
        assert!(output.contains("application.started"));
        assert!(output.contains("library.scan.completed"));
        assert!(output.contains("\"scanned_files\":12"));
        assert!(!output.contains("media_path"));
        assert!(!output.contains("api_key"));
        for line in output.lines() {
            let record: serde_json::Value = serde_json::from_str(line).expect("signed record JSON");
            assert!(verify_signed_record(&record));
            let mut tampered = record.clone();
            tampered["payload"]["event"] = serde_json::json!("application.stopped");
            assert!(!verify_signed_record(&tampered));
        }

        fs::remove_dir_all(root).expect("test files removed");
    }

    #[test]
    fn rejects_path_like_device_ids() {
        let root = std::env::temp_dir().join(format!("rawrarr-logs-{}", Uuid::new_v4()));
        assert!(SharedLogger::open(&root, "../outside", "device", "metadata", None, Vec::new(), |_| Ok(Vec::new())).is_err());
        assert!(!root.exists());
    }

    #[test]
    fn shared_record_verification_requires_the_pinned_root_certificate() {
        let root = std::env::temp_dir().join(format!("rawrarr-trusted-logs-{}", Uuid::new_v4()));
        let root_key = SigningKey::<MlDsa65>::from_seed(&[21u8; 32].into());
        let node_key = SigningKey::<MlDsa65>::from_seed(&[22u8; 32].into());
        let root_public = root_key.verifying_key().encode().to_vec();
        let node_public = node_key.verifying_key().encode().to_vec();
        let root_fingerprint = crate::device_signing::fingerprint(&root_public);
        let node_public_hex = encode_hex(&node_public);
        let signed_payload = serde_json::json!({
            "domain":"rawrarr-device-approval-v1", "requestId":"request-fixture",
            "nodeId":"node-fixture", "role":"metadata", "displayName":"Fixture",
            "publicKeyHex":node_public_hex,
        });
        let approval_bytes = crate::device_signing::canonical_json(&signed_payload).unwrap();
        let approval_signature = root_key.sign(&approval_bytes).encode();
        let certificate = serde_json::json!({
            "algorithm":"ML-DSA-65", "rootPublicKey":BASE64.encode(root_public),
            "rootFingerprint":root_fingerprint, "nodeId":"node-fixture", "role":"metadata",
            "signingPublicKeyHex":node_public_hex,
            "signingFingerprint":crate::device_signing::node_fingerprint(&node_public),
            "signedPayload":signed_payload, "signature":BASE64.encode(approval_signature),
        });
        let signer = node_key.clone();
        let logger = SharedLogger::open(&root, "fixture-device", "node-fixture", "metadata", Some(certificate.clone()), node_public, move |message| {
            Ok(signer.sign(message).encode().to_vec())
        }).expect("trusted logger opens");
        logger.application_started();
        let path = fs::read_dir(root.join("logs/fixture-device")).unwrap().next().unwrap().unwrap().path();
        let mut decoder = MultiGzDecoder::new(File::open(path).unwrap());
        let mut output = String::new();
        decoder.read_to_string(&mut output).unwrap();
        let record: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert!(verify_trusted_signed_record(&record, &root_fingerprint));
        assert!(!verify_trusted_signed_record(&record, &"0".repeat(64)));
        fs::remove_dir_all(root).unwrap();
    }
}
