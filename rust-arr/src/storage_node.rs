//! Storage-node media scanning and age-encrypted sidecar publication.

use crate::{devices, library, secure_config};
use chrono::{DateTime, Datelike, Duration as ChronoDuration, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use md5::Md5;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const DEFAULT_SCAN_TIME: &str = "04:00";
const DEFAULT_SCAN_TIMEZONE: &str = "UTC";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScanFrequency {
    Daily,
    Weekly,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScanWeekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl ScanWeekday {
    fn chrono(self) -> Weekday {
        match self {
            Self::Monday => Weekday::Mon,
            Self::Tuesday => Weekday::Tue,
            Self::Wednesday => Weekday::Wed,
            Self::Thursday => Weekday::Thu,
            Self::Friday => Weekday::Fri,
            Self::Saturday => Weekday::Sat,
            Self::Sunday => Weekday::Sun,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScanSchedule {
    pub frequency: ScanFrequency,
    pub time: String,
    pub time_zone: String,
    pub weekday: Option<ScanWeekday>,
}

impl ScanSchedule {
    pub fn default_for_timezone(time_zone: String) -> Self {
        Self {
            frequency: ScanFrequency::Daily,
            time: DEFAULT_SCAN_TIME.to_owned(),
            time_zone,
            weekday: None,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        let time_bytes = self.time.as_bytes();
        if self.time.len() != 5
            || time_bytes[2] != b':'
            || !time_bytes[..2].iter().all(u8::is_ascii_digit)
            || !time_bytes[3..].iter().all(u8::is_ascii_digit)
            || NaiveTime::parse_from_str(&self.time, "%H:%M").is_err()
        {
            return Err("time must use 24-hour HH:MM format");
        }
        if self.time_zone.parse::<Tz>().is_err() {
            return Err("timeZone must be a recognized IANA time zone");
        }
        match (self.frequency, self.weekday) {
            (ScanFrequency::Daily, None) | (ScanFrequency::Weekly, Some(_)) => Ok(()),
            (ScanFrequency::Daily, Some(_)) => Err("weekday is only valid for a weekly schedule"),
            (ScanFrequency::Weekly, None) => Err("weekday is required for a weekly schedule"),
        }
    }

    fn time_zone(&self) -> Tz {
        self.time_zone.parse().expect("validated scan time zone")
    }

    fn time(&self) -> NaiveTime {
        NaiveTime::parse_from_str(&self.time, "%H:%M").expect("validated scan time")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistedScanSchedule {
    schedule: ScanSchedule,
    last_scheduled_run_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanScheduleStatus {
    pub schedule: ScanSchedule,
    pub next_run_at: DateTime<Utc>,
    pub last_scheduled_run_at: Option<DateTime<Utc>>,
    pub scan: ScanStatus,
}

#[derive(Clone)]
pub struct ScanScheduler {
    state: Arc<Mutex<PersistedScanSchedule>>,
    path: PathBuf,
}

impl ScanScheduler {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let path =
            PathBuf::from(std::env::var_os("RAWARR_DATA_DIR").unwrap_or_else(|| "/data".into()))
                .join("storage-scan-schedule.json");
        let state = match fs::read(&path) {
            Ok(bytes) => {
                let state: PersistedScanSchedule = serde_json::from_slice(&bytes)?;
                state.schedule.validate()?;
                state
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let time_zone = std::env::var("RAWARR_STORAGE_SCAN_TIMEZONE")
                    .unwrap_or_else(|_| DEFAULT_SCAN_TIMEZONE.to_owned());
                let schedule = ScanSchedule::default_for_timezone(time_zone);
                schedule.validate()?;
                let state = PersistedScanSchedule {
                    schedule,
                    last_scheduled_run_at: None,
                };
                save_schedule(&path, &state)?;
                state
            }
            Err(error) => return Err(error.into()),
        };
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            path,
        })
    }

    pub async fn status(&self, controller: &ScanController) -> ScanScheduleStatus {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        ScanScheduleStatus {
            next_run_at: next_scheduled_run(&state.schedule, Utc::now())
                .expect("valid scan schedule always has a future run"),
            schedule: state.schedule,
            last_scheduled_run_at: state.last_scheduled_run_at,
            scan: controller.status().await,
        }
    }

    pub fn update(&self, schedule: ScanSchedule) -> Result<(), Box<dyn Error>> {
        schedule.validate()?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let updated = PersistedScanSchedule {
            schedule,
            last_scheduled_run_at: state.last_scheduled_run_at,
        };
        save_schedule(&self.path, &updated)?;
        *state = updated;
        Ok(())
    }

    fn due_run(&self, now: DateTime<Utc>) -> Result<Option<DateTime<Utc>>, Box<dyn Error>> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let Some(due) = next_scheduled_run(&state.schedule, now - ChronoDuration::minutes(1)) else {
            return Ok(None);
        };
        if due > now || state.last_scheduled_run_at.is_some_and(|last| last >= due) {
            return Ok(None);
        }
        Ok(Some(due))
    }

    fn record_run(&self, run_at: DateTime<Utc>) -> Result<(), Box<dyn Error>> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let updated = PersistedScanSchedule {
            schedule: state.schedule.clone(),
            last_scheduled_run_at: Some(run_at),
        };
        let result = save_schedule(&self.path, &updated);
        *state = updated;
        result
    }
}

fn save_schedule(path: &Path, state: &PersistedScanSchedule) -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let parent = path.parent().ok_or("storage data directory is invalid")?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".storage-scan-schedule-{}-{}.tmp",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
    file.write_all(&serde_json::to_vec(state)?)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn next_scheduled_run(schedule: &ScanSchedule, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let timezone = schedule.time_zone();
    let first_date = now.with_timezone(&timezone).date_naive();
    for offset in 0..=15 {
        let date = first_date.checked_add_days(chrono::Days::new(offset))?;
        let matches_frequency = match schedule.frequency {
            ScanFrequency::Daily => true,
            ScanFrequency::Weekly => schedule
                .weekday
                .is_some_and(|day| date.weekday() == day.chrono()),
        };
        if !matches_frequency {
            continue;
        }
        let local_datetime = date.and_time(schedule.time());
        let scheduled = match timezone.from_local_datetime(&local_datetime) {
            chrono::LocalResult::Single(value) => Some(value),
            chrono::LocalResult::Ambiguous(first, second) => Some(first.min(second)),
            chrono::LocalResult::None => None,
        };
        let Some(scheduled) = scheduled else {
            continue;
        };
        let scheduled = scheduled.with_timezone(&Utc);
        if scheduled > now {
            return Some(scheduled);
        }
    }
    None
}

/// Run the configured wall-clock schedule once per due minute. Schedule
/// settings and the last claimed run are kept on the storage node's /data.
pub fn start_scan_scheduler(
    scheduler: ScanScheduler,
    logger: Option<crate::shared_logs::SharedLogger>,
    controller: ScanController,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(15)).await;
            if controller.status().await.active {
                continue;
            }
            let due = match scheduler.due_run(Utc::now()) {
                Ok(Some(due)) => due,
                Ok(None) => continue,
                Err(error) => {
                    eprintln!("storage scan schedule could not be checked: {error}");
                    continue;
                }
            };
            match controller
                .start(ScanOptions::default(), logger.clone(), true)
                .await
            {
                Ok(()) => {
                    if let Err(error) = scheduler.record_run(due) {
                        eprintln!("storage scan schedule run time could not be saved: {error}");
                    }
                }
                Err(error) => eprintln!("scheduled storage scan could not start: {error}"),
            }
        }
    });
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScanOptions {
    pub kind: Option<library::MediaKind>,
    pub minimum_size_bytes: Option<u64>,
    pub maximum_size_bytes: Option<u64>,
    pub max_files: Option<usize>,
}

impl ScanOptions {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.minimum_size_bytes.zip(self.maximum_size_bytes)
            .is_some_and(|(minimum, maximum)| minimum > maximum)
        {
            return Err("minimumSizeBytes cannot exceed maximumSizeBytes");
        }
        if self.max_files.is_some_and(|limit| limit == 0 || limit > 10) {
            return Err("maxFiles must be between 1 and 10");
        }
        Ok(())
    }

    fn matches(&self, file: &library::MediaFile) -> bool {
        self.kind.is_none_or(|kind| kind == file.kind)
            && self.minimum_size_bytes.is_none_or(|min| file.size_bytes >= min)
            && self.maximum_size_bytes.is_none_or(|max| file.size_bytes <= max)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatus {
    pub active: bool,
    pub scheduled: bool,
    pub cancel_requested: bool,
    pub cancelled: bool,
    pub media_files_found: usize,
    pub files_completed: usize,
    pub files_total: usize,
    pub files_considered: usize,
    pub sidecars_created: usize,
    pub sidecars_skipped: usize,
    pub sidecar_errors: usize,
    pub issues: usize,
    pub truncated: bool,
    pub error: Option<String>,
}

#[derive(Clone, Default)]
pub struct ScanController {
    status: Arc<Mutex<ScanStatus>>,
}

impl ScanController {
    pub async fn status(&self) -> ScanStatus {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub async fn cancel(&self) -> Result<ScanStatus, &'static str> {
        let mut status = self.status.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if !status.active {
            return Err("no storage scan is running");
        }
        status.cancel_requested = true;
        Ok(status.clone())
    }

    pub async fn start(
        &self,
        options: ScanOptions,
        logger: Option<crate::shared_logs::SharedLogger>,
        scheduled: bool,
    ) -> Result<(), &'static str> {
        options.validate()?;
        let mut status = self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if status.active {
            return Err("a storage scan is already running");
        }
        *status = ScanStatus {
            active: true,
            scheduled,
            files_total: options.max_files.unwrap_or_default(),
            ..ScanStatus::default()
        };
        drop(status);

        let shared_status = self.status.clone();
        tokio::spawn(async move {
            let progress_status = shared_status.clone();
            let cancellation_status = shared_status.clone();
            let result = tokio::task::spawn_blocking(move || {
                scan_missing_with_progress(&options, move |completed, total| {
                    let mut status = progress_status
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    status.files_completed = completed;
                    status.files_total = total;
                }, move || {
                    cancellation_status.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).cancel_requested
                })
                .map_err(|error| std::io::Error::other(error.to_string()))
            }).await;
            let final_status = match result {
                Ok(Ok(summary)) => {
                    if let Some(logger) = logger {
                        logger.scan_completed(
                            summary.items.len(),
                            summary.sidecars_created,
                            summary.sidecar_errors,
                            summary.issues,
                            summary.truncated,
                        );
                    }
                    println!(
                        "Storage scan {}; {} media files; {} files checked; {} encrypted sidecars saved; {} already complete; {} sidecar errors; {} scan issues; truncated={}",
                        if summary.cancelled { "cancelled" } else { "complete" },
                        summary.items.len(),
                        summary.sidecars_created
                            + summary.sidecars_skipped
                            + summary.sidecar_errors,
                        summary.sidecars_created,
                        summary.sidecars_skipped,
                        summary.sidecar_errors,
                        summary.issues,
                        summary.truncated
                    );
                    if summary.sidecar_errors > 0 || summary.issues > 0 || summary.truncated {
                        eprintln!("the next scheduled storage scan will retry incomplete records");
                    }
                    ScanStatus {
                        active: false,
                        scheduled,
                        cancel_requested: summary.cancelled,
                        cancelled: summary.cancelled,
                        media_files_found: summary.items.len(),
                        files_completed: summary.sidecars_created
                            + summary.sidecars_skipped
                            + summary.sidecar_errors,
                        files_total: summary.sidecars_created
                            + summary.sidecars_skipped
                            + summary.sidecar_errors,
                        files_considered: summary.sidecars_created
                            + summary.sidecars_skipped
                            + summary.sidecar_errors,
                        sidecars_created: summary.sidecars_created,
                        sidecars_skipped: summary.sidecars_skipped,
                        sidecar_errors: summary.sidecar_errors,
                        issues: summary.issues,
                        truncated: summary.truncated,
                        error: None,
                    }
                }
                Ok(Err(_)) | Err(_) => ScanStatus {
                    active: false,
                    error: Some("storage scan failed".to_owned()),
                    ..ScanStatus::default()
                },
            };
            *shared_status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = final_status;
        });
        Ok(())
    }
}

pub fn scan_once() -> Result<library::ScanSummary, Box<dyn Error>> {
    scan_missing_once(&ScanOptions::default())
}

pub fn scan_missing_once(options: &ScanOptions) -> Result<library::ScanSummary, Box<dyn Error>> {
    scan_missing_with_progress(options, |_, _| {}, || false)
}

fn scan_missing_with_progress(
    options: &ScanOptions,
    mut report_progress: impl FnMut(usize, usize),
    mut cancellation_requested: impl FnMut() -> bool,
) -> Result<library::ScanSummary, Box<dyn Error>> {
    options.validate()?;
    let root = library::LibraryRoot::from_env()?.ok_or("RAWARR_LIBRARY_PATH is required")?;
    let metadata_dir = secure_config::AgeVault::metadata_dir();
    let recipient = fs::read_to_string(secure_config::AgeVault::public_recipient_path())?;
    let recipient = recipient.trim();
    if !secure_config::is_post_quantum_recipient(recipient) {
        return Err("shared age-recipient is missing or invalid".into());
    }

    let device_id = devices::load_or_create_device_id()?;
    let mut summary = root.scan();
    let total = summary
        .items
        .iter()
        .filter(|file| options.matches(file))
        .take(options.max_files.unwrap_or(usize::MAX))
        .count();
    let mut completed = 0usize;
    report_progress(completed, total);
    let cache_path = scan_cache_path();
    let mut cache = load_scan_cache(&cache_path);
    let mut processed = 0usize;
    let media_root = fs::canonicalize(std::env::var("RAWARR_LIBRARY_PATH")?)?;
    for file in &summary.items {
        if cancellation_requested() {
            summary.cancelled = true;
            break;
        }
        if !options.matches(file) {
            continue;
        }
        if options.max_files.is_some_and(|limit| processed >= limit) {
            break;
        }
        processed += 1;
        let media_path = media_root.join(&file.path);
        let key = cache_key(&file.path);
        let fingerprint = ScanFingerprint {
            size_bytes: file.size_bytes,
            modified_at_unix_seconds: file.modified_at_unix_seconds,
        };
        let cached = cache.get(&key) == Some(&fingerprint);
        let sidecar_exists = secure_config::AgeVault::library_sidecar_exists_with_recipient(
            &metadata_dir,
            recipient,
            &device_id,
            &summary.library_name,
            &file.path,
        )
        .unwrap_or(false);
        if cached && sidecar_exists {
            summary.sidecars_skipped += 1;
            completed += 1;
            report_progress(completed, total);
            continue;
        }
        let hashes = match hash_file(&media_path) {
            Ok(hashes) => hashes,
            Err(_) => {
                summary.sidecar_errors += 1;
                completed += 1;
                report_progress(completed, total);
                continue;
            }
        };
        let probe = match probe_media(&media_path) {
            Ok(probe) => probe,
            Err(_) => {
                summary.sidecar_errors += 1;
                completed += 1;
                report_progress(completed, total);
                continue;
            }
        };
        let sidecar = FileDerivedSidecar {
            schema_version: 1,
            device_id: device_id.clone(),
            library_name: summary.library_name.clone(),
            scanned_at_unix_seconds: summary.scanned_at_unix_seconds,
            file: file.clone(),
            hashes,
            format: probe.format,
            streams: probe.streams,
            sidecar_modified_at_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        match serde_json::to_vec(&sidecar)
            .map_err(|_| secure_config::VaultError::InvalidConfig)
            .and_then(|json| {
                secure_config::AgeVault::store_library_sidecar_with_recipient(
                    &metadata_dir,
                    recipient,
                    &device_id,
                    &summary.library_name,
                    &file.path,
                    &json,
                )
            }) {
            Ok(()) => {
                summary.sidecars_created += 1;
                cache.insert(key, fingerprint);
            }
            Err(_) => summary.sidecar_errors += 1,
        }
        completed += 1;
        report_progress(completed, total);
    }
    if save_scan_cache(&cache_path, &cache).is_err() {
        summary.sidecar_errors += 1;
    }
    Ok(summary)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ScanFingerprint {
    size_bytes: u64,
    modified_at_unix_seconds: Option<u64>,
}

fn scan_cache_path() -> PathBuf {
    PathBuf::from(std::env::var_os("RAWARR_DATA_DIR").unwrap_or_else(|| "/data".into()))
        .join("storage-sidecar-cache.json")
}

fn cache_key(relative_path: &str) -> String {
    hex(&Sha256::digest(relative_path.as_bytes()))
}

fn load_scan_cache(path: &Path) -> BTreeMap<String, ScanFingerprint> {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save_scan_cache(
    path: &Path,
    cache: &BTreeMap<String, ScanFingerprint>,
) -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let parent = path.parent().ok_or("storage data directory is invalid")?;
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = parent.join(format!(
        ".storage-sidecar-cache-{}-{nonce}.tmp",
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
    file.write_all(&serde_json::to_vec(cache)?)?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileDerivedSidecar {
    schema_version: u32,
    device_id: String,
    library_name: String,
    scanned_at_unix_seconds: u64,
    file: library::MediaFile,
    hashes: FileHashes,
    format: serde_json::Value,
    streams: Vec<serde_json::Value>,
    sidecar_modified_at_unix_seconds: u64,
}

#[derive(Serialize)]
struct FileHashes {
    md5: String,
    sha1: String,
    sha256: String,
}

#[derive(serde::Deserialize)]
struct ProbeOutput {
    #[serde(default)]
    format: serde_json::Value,
    #[serde(default)]
    streams: Vec<serde_json::Value>,
}

fn hash_file(path: &Path) -> Result<FileHashes, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut md5 = Md5::new();
    let mut sha1 = Sha1::new();
    let mut sha256 = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        md5.update(&buffer[..count]);
        sha1.update(&buffer[..count]);
        sha256.update(&buffer[..count]);
    }
    Ok(FileHashes {
        md5: hex(&md5.finalize()),
        sha1: hex(&sha1.finalize()),
        sha256: hex(&sha256.finalize()),
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn probe_media(path: &Path) -> Result<ProbeOutput, Box<dyn Error>> {
    let executable =
        std::env::var("RAWARR_FFPROBE").unwrap_or_else(|_| "ffprobe".to_owned());
    let output = Command::new(executable)
        .args([
            "-v",
            "error",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
        ])
        .arg(path)
        .output()?;
    if !output.status.success() {
        return Err("ffprobe failed".into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_storage_node.rs"]
mod tests;
