//! Normalized movie metadata lookup types for provider adapters.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// Build a shareable provider record while omitting node-local paths and
/// byte-size observations that belong to storage-node sidecars.
pub(crate) fn arr_text_record(provider: &str, id_field: &str, id: u64, source: &Value) -> Value {
    let mut text = source.clone();
    sanitize_arr_text_fields(&mut text);
    let mut record = match text {
        Value::Object(fields) => fields,
        _ => Map::new(),
    };
    record.insert("schemaVersion".into(), json!(2));
    record.insert("provider".into(), json!(provider));
    record.insert(id_field.into(), json!(id));
    Value::Object(record)
}

pub(crate) fn sanitize_arr_text_fields(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            fields.retain(|key, _| {
                let normalized: String = key.chars().filter(|character| character.is_ascii_alphanumeric())
                    .map(|character| character.to_ascii_lowercase()).collect();
                !normalized.ends_with("path")
                    && !matches!(normalized.as_str(), "size" | "sizeondisk" | "filesize" | "filesizebytes" | "bytes" | "totalbytes")
            });
            for field in fields.values_mut() { sanitize_arr_text_fields(field); }
        }
        Value::Array(items) => {
            for item in items { sanitize_arr_text_fields(item); }
        }
        _ => {}
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieSearchRequest {
    pub query: String,
    pub year: Option<u16>,
}

impl MovieSearchRequest {
    pub fn validate(&self) -> Result<String, SearchError> {
        let query = self.query.trim();
        if query.is_empty() || query.chars().count() > 120 {
            return Err(SearchError::InvalidQuery);
        }
        if self.year.is_some_and(|year| !(1870..=2100).contains(&year)) {
            return Err(SearchError::InvalidYear);
        }
        let mut term = query.to_owned();
        if let Some(year) = self.year {
            term.push(' ');
            term.push_str(&year.to_string());
        }
        Ok(term)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieSearchResponse {
    pub provider: &'static str,
    pub candidates: Vec<MovieCandidate>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieCandidate {
    pub tmdb_id: u64,
    pub title: String,
    pub original_title: String,
    pub year: Option<u16>,
    pub overview: String,
    pub poster_url: Option<String>,
    #[serde(skip)]
    pub(crate) poster_source: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RadarrImage {
    pub cover_type: String,
    pub remote_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarrMovieLookupItem {
    pub tmdb_id: u64,
    pub title: String,
    #[serde(default)]
    pub original_title: String,
    pub year: Option<u16>,
    #[serde(default)]
    pub overview: String,
    #[serde(default)]
    pub images: Vec<RadarrImage>,
}

impl From<RadarrMovieLookupItem> for MovieCandidate {
    fn from(item: RadarrMovieLookupItem) -> Self {
        Self {
            tmdb_id: item.tmdb_id,
            title: item.title,
            original_title: item.original_title,
            year: item.year,
            overview: item.overview,
            poster_url: None,
            poster_source: item
                .images
                .into_iter()
                .find(|image| image.cover_type.eq_ignore_ascii_case("poster"))
                .and_then(|image| image.remote_url),
        }
    }
}

#[derive(Debug)]
pub enum SearchError {
    InvalidQuery,
    InvalidYear,
    Upstream,
}
