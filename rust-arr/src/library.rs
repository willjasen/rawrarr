//! Media inventory scanning and age-encrypted sidecar creation for one library root.

use serde::Serialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_DEPTH: usize = 64;
const MAX_WALK_ENTRIES: usize = 250_000;
const MAX_MEDIA_ITEMS: usize = 10_000;

#[derive(Clone, Debug)]
pub struct LibraryRoot {
    path: PathBuf,
    name: String,
}

impl LibraryRoot {
    pub fn from_env() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let Some(path) = env::var_os("RAWARR_LIBRARY_PATH") else {
            return Ok(None);
        };
        let path = PathBuf::from(path);
        if path.as_os_str().is_empty() {
            return Ok(None);
        }
        let path = fs::canonicalize(path)?;
        if !path.is_dir() {
            return Err("RAWARR_LIBRARY_PATH must point to a directory".into());
        }

        let name = env::var("RAWARR_LIBRARY_NAME")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| path.file_name().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "Media library".to_owned());

        Ok(Some(Self { path, name }))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn scan(&self) -> ScanSummary {
        scan_root(&self.path, &self.name)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct LibraryInfo {
    pub configured: bool,
    pub name: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Movie,
    Television,
    Music,
    Book,
    Subtitle,
    Other,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct MediaCounts {
    pub movies: usize,
    pub television: usize,
    pub music: usize,
    pub books: usize,
    pub subtitles: usize,
    pub other: usize,
}

impl MediaCounts {
    fn increment(&mut self, kind: MediaKind) {
        match kind {
            MediaKind::Movie => self.movies += 1,
            MediaKind::Television => self.television += 1,
            MediaKind::Music => self.music += 1,
            MediaKind::Book => self.books += 1,
            MediaKind::Subtitle => self.subtitles += 1,
            MediaKind::Other => self.other += 1,
        }
    }

    pub fn total(&self) -> usize {
        self.movies + self.television + self.music + self.books + self.subtitles + self.other
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFile {
    pub path: String,
    pub name: String,
    pub kind: MediaKind,
    pub size_bytes: u64,
    pub modified_at_unix_seconds: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSidecar {
    pub schema_version: u32,
    pub device_id: String,
    pub library_name: String,
    pub scanned_at_unix_seconds: u64,
    pub file: MediaFile,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub library_name: String,
    pub scanned_at_unix_seconds: u64,
    pub counts: MediaCounts,
    pub items: Vec<MediaFile>,
    pub issues: usize,
    pub truncated: bool,
    pub sidecars_created: usize,
    pub sidecars_skipped: usize,
    pub sidecar_errors: usize,
    pub cancelled: bool,
}

fn scan_root(root: &Path, name: &str) -> ScanSummary {
    let mut summary = ScanSummary {
        library_name: name.to_owned(),
        scanned_at_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        counts: MediaCounts::default(),
        items: Vec::new(),
        issues: 0,
        truncated: false,
        sidecars_created: 0,
        sidecars_skipped: 0,
        sidecar_errors: 0,
        cancelled: false,
    };
    let mut pending = vec![(root.to_path_buf(), 0usize)];
    let mut visited = 0usize;

    'walk: while let Some((directory, depth)) = pending.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => {
                summary.issues += 1;
                continue;
            }
        };
        let mut child_directories = Vec::new();

        for entry in entries {
            if visited >= MAX_WALK_ENTRIES {
                summary.truncated = true;
                break 'walk;
            }
            visited += 1;

            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    summary.issues += 1;
                    continue;
                }
            };
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => {
                    summary.issues += 1;
                    continue;
                }
            };
            if file_type.is_symlink() {
                continue;
            }

            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if file_type.is_dir() {
                if file_name.starts_with('.') {
                    continue;
                }
                if depth >= MAX_DEPTH {
                    summary.truncated = true;
                } else {
                    child_directories.push(path);
                }
                continue;
            }
            if !file_type.is_file() || file_name.starts_with('.') {
                continue;
            }

            let Some(kind) = classify(&path) else {
                continue;
            };
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => {
                    summary.issues += 1;
                    continue;
                }
            };
            let relative = match path.strip_prefix(root) {
                Ok(relative) => relative.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };

            summary.counts.increment(kind);
            summary.items.push(MediaFile {
                path: relative,
                name: file_name,
                kind,
                size_bytes: metadata.len(),
                modified_at_unix_seconds: metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs()),
            });
            if summary.items.len() >= MAX_MEDIA_ITEMS {
                summary.truncated = true;
                break 'walk;
            }
        }

        child_directories.sort();
        pending.extend(
            child_directories
                .into_iter()
                .rev()
                .map(|path| (path, depth + 1)),
        );
    }

    summary.items.sort_by(|a, b| {
        (a.kind as u8, a.path.to_lowercase()).cmp(&(b.kind as u8, b.path.to_lowercase()))
    });
    summary
}

fn classify(path: &Path) -> Option<MediaKind> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    match extension.as_str() {
        "mkv" | "mp4" | "m4v" | "avi" | "mov" | "mpeg" | "mpg" | "m2v" | "ts" | "webm" | "wmv"
        | "flv" | "ogv" => {
            if looks_like_episode(path) {
                Some(MediaKind::Television)
            } else {
                Some(MediaKind::Movie)
            }
        }
        "mp3" | "flac" | "m4a" | "aac" | "ogg" | "opus" | "wav" | "wma" | "aiff" | "alac" => {
            Some(MediaKind::Music)
        }
        "epub" | "mobi" | "azw" | "azw3" | "pdf" | "cbz" | "cbr" => Some(MediaKind::Book),
        "srt" | "ass" | "ssa" | "vtt" | "sub" => Some(MediaKind::Subtitle),
        "iso" | "img" => Some(MediaKind::Other),
        _ => None,
    }
}

fn looks_like_episode(path: &Path) -> bool {
    if path.components().any(|component| {
        let component = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        component.starts_with("season ")
            || component.starts_with("season_")
            || component.starts_with("season-")
    }) {
        return true;
    }

    let Some(stem) = path.file_stem() else {
        return false;
    };
    let name = stem.to_string_lossy().to_ascii_lowercase();
    let bytes = name.as_bytes();
    for start in 0..bytes.len().saturating_sub(3) {
        if bytes[start] != b's' {
            continue;
        }
        let mut index = start + 1;
        let season_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() && index - season_start < 2 {
            index += 1;
        }
        if index == season_start || index >= bytes.len() || bytes[index] != b'e' {
            continue;
        }
        index += 1;
        let episode_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index > episode_start {
            return true;
        }
    }
    false
}
