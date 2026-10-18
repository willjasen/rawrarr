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
    session_id: String,
    part: u32,
}

#[derive(Serialize)]
struct LogRecord<'a> {
    timestamp_unix_seconds: u64,
    device_id: &'a str,
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
    pub fn open(shared_dir: &Path, device_id: &str) -> io::Result<Self> {
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
            session_id: Uuid::new_v4().to_string(),
            part: 0,
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
            event,
            scanned_files: scan.map(|values| values.0),
            sidecars_created: scan.map(|values| values.1),
            sidecar_errors: scan.map(|values| values.2),
            scan_issues: scan.map(|values| values.3),
            scan_truncated: scan.map(|values| values.4),
        };

        let compressed = compress_record(&record)?;
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

fn compress_record(record: &LogRecord<'_>) -> io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    serde_json::to_writer(&mut encoder, record).map_err(io::Error::other)?;
    encoder.write_all(b"\n")?;
    encoder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::MultiGzDecoder;
    use std::{fs::File, io::Read};

    #[test]
    fn writes_appendable_gzip_members_with_only_allowlisted_fields() {
        let root = std::env::temp_dir().join(format!("rawrarr-logs-{}", Uuid::new_v4()));
        let logger = SharedLogger::open(&root, "test-device").expect("logger opens");
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

        fs::remove_dir_all(root).expect("test files removed");
    }

    #[test]
    fn rejects_path_like_device_ids() {
        let root = std::env::temp_dir().join(format!("rawrarr-logs-{}", Uuid::new_v4()));
        assert!(SharedLogger::open(&root, "../outside").is_err());
        assert!(!root.exists());
    }
}
