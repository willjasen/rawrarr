//! Normalized movie metadata lookup types for provider adapters.

use serde::{Deserialize, Serialize};

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
