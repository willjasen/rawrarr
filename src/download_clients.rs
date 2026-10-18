//! Provider-neutral contracts for download client adapters.
//!
//! Concrete providers such as qBittorrent and SABnzbd should translate their
//! API-specific details into these types. This module does not configure or
//! contact any download client.

use std::{future::Future, pin::Pin};

/// Boxed future keeps adapter implementations usable through `dyn DownloadClient`
/// without requiring an async-trait macro dependency.
pub type ClientFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ClientError>> + Send + 'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientDescriptor {
    /// Stable application-owned identifier for this configured client.
    pub id: String,
    /// User-facing label, such as "Seedbox" or "Home Usenet".
    pub name: String,
    /// Adapter identifier, such as `qbittorrent` or `sabnzbd`.
    pub provider: String,
    pub protocol: DownloadProtocol,
    pub capabilities: Vec<ClientCapability>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadProtocol {
    BitTorrent,
    Usenet,
    Http,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientCapability {
    Queue,
    Add,
    Pause,
    Resume,
    Remove,
    Categories,
    Priorities,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueState {
    Queued,
    Downloading,
    Paused,
    Verifying,
    Completed,
    Failed,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueItem {
    pub id: String,
    pub name: String,
    pub state: QueueState,
    /// Normalized integer percent in the inclusive range 0 through 100.
    pub progress_percent: u8,
    pub size_bytes: Option<u64>,
    pub downloaded_bytes: Option<u64>,
    pub rate_bytes_per_second: Option<u64>,
    pub category: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadPriority {
    Low,
    Normal,
    High,
}

/// A transfer source without provider-specific fields.
///
/// `Uri` may contain credentials or signed query parameters; adapters and
/// callers must not include it in logs or error text. `Payload` supports
/// clients that accept torrent or NZB files directly.
pub enum DownloadSource {
    Uri(String),
    Payload {
        file_name: String,
        media_type: Option<String>,
        bytes: Vec<u8>,
    },
}

pub struct AddDownloadRequest {
    pub title: String,
    pub source: DownloadSource,
    /// Optional organization hint. Providers may map this to a category or label.
    pub category: Option<String>,
    pub priority: DownloadPriority,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadReceipt {
    /// Identifier assigned by the client, opaque to rawrarr.
    pub client_item_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientErrorKind {
    Unavailable,
    Unauthorized,
    InvalidConfiguration,
    Rejected,
    Unsupported,
    UnexpectedResponse,
}

/// Safe, normalized error category. Provider response bodies and credentials
/// are deliberately excluded from this type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientError {
    pub kind: ClientErrorKind,
}

/// Adapter contract shared by all download methods.
///
/// Provider implementations own their connection details and normalize the
/// results into rawrarr's protocol-independent models.
pub trait DownloadClient: Send + Sync {
    fn descriptor(&self) -> &ClientDescriptor;

    fn test_connection(&self) -> ClientFuture<'_, ()>;

    fn list_queue(&self) -> ClientFuture<'_, Vec<QueueItem>>;

    fn add(&self, request: AddDownloadRequest) -> ClientFuture<'_, DownloadReceipt>;
}
