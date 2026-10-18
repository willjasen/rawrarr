//! Cache-first quality profile names in encrypted shared movie metadata.
use crate::{RadarrClient, secure_config::AgeVault};
use serde_json::{Value, json};

pub(crate) trait MovieCache {
    fn load(&self, id: u64) -> Result<Option<Value>, &'static str>;
    fn store(&self, id: u64, metadata: &Value) -> Result<(), &'static str>;
}

struct SharedCache(Option<AgeVault>);

impl MovieCache for SharedCache {
    fn load(&self, id: u64) -> Result<Option<Value>, &'static str> {
        if let Some(vault) = &self.0 {
            return vault.load_movie_metadata(id).map_err(|_| "Could not read shared movie metadata");
        }
        if AgeVault::metadata_dir().join("movies/text").join(format!("{id}.json.aged")).exists() {
            return Err("Unlock the age vault to read shared movie quality profiles");
        }
        Ok(None)
    }

    fn store(&self, id: u64, metadata: &Value) -> Result<(), &'static str> {
        let bytes = serde_json::to_vec(metadata).map_err(|_| "Invalid movie metadata")?;
        match &self.0 {
            Some(vault) => vault.store_movie_metadata(id, &bytes),
            None => AgeVault::store_movie_metadata_with_shared_recipient(id, &bytes),
        }.map_err(|_| "Could not save quality profiles to shared movie metadata")
    }
}

fn profile_name(metadata: &Value) -> Option<&str> {
    metadata.get("qualityProfile")
        .and_then(|value| value.as_str().or_else(|| value.get("name").and_then(Value::as_str)))
        .filter(|name| !name.trim().is_empty())
}

fn cached_name<'a>(metadata: &'a Value, movie: &Value) -> Option<&'a str> {
    // A changed assignment must not keep displaying the old profile.
    let saved_id = metadata.get("qualityProfileId").and_then(Value::as_u64)
        .or_else(|| metadata.pointer("/qualityProfile/id").and_then(Value::as_u64));
    if saved_id.is_some() && saved_id != movie.get("qualityProfileId").and_then(Value::as_u64) {
        return None;
    }
    profile_name(metadata)
}

fn read_cache(movies: &mut [Value], cache: &impl MovieCache) -> Result<Vec<Option<Value>>, &'static str> {
    movies.iter_mut().map(|movie| {
        let metadata = match movie.get("tmdbId").and_then(Value::as_u64).filter(|id| *id > 0) {
            Some(id) => cache.load(id)?,
            None => None,
        };
        if let Some(name) = metadata.as_ref().and_then(|metadata| cached_name(metadata, movie)) {
            movie["qualityProfile"] = json!({"name": name});
        }
        Ok(metadata)
    }).collect()
}

fn merge_profiles(
    movies: &mut [Value], cached: &[Option<Value>], profiles: &[Value], cache: &impl MovieCache,
) -> Result<(), &'static str> {
    for (movie, saved) in movies.iter_mut().zip(cached) {
        if saved.as_ref().and_then(|metadata| cached_name(metadata, movie)).is_some() {
            continue;
        }
        let profile_id = movie.get("qualityProfileId").and_then(Value::as_u64);
        let name = profile_name(movie).map(str::to_owned).or_else(|| {
            profiles.iter().find(|profile| profile_id.is_some() && profile.get("id").and_then(Value::as_u64) == profile_id)
                .and_then(|profile| profile.get("name").and_then(Value::as_str))
                .filter(|name| !name.trim().is_empty()).map(str::to_owned)
        });
        let Some(name) = name else { continue };
        movie["qualityProfile"] = json!({"name": name});
        let Some(id) = movie.get("tmdbId").and_then(Value::as_u64).filter(|id| *id > 0) else { continue };
        let mut metadata = saved.clone().unwrap_or_else(|| json!({"schemaVersion": 1, "provider": "radarr", "tmdbId": id}));
        if !metadata.is_object() { return Err("Invalid shared movie metadata"); }
        // Keep existing fields; seed descriptive metadata without copying local paths.
        for key in ["title", "originalTitle", "year", "overview", "genres"] {
            if metadata.get(key).is_none() {
                if let Some(value) = movie.get(key) { metadata[key] = value.clone(); }
            }
        }
        if metadata.get("images").is_none() {
            if let Some(images) = movie.get("images").and_then(Value::as_array) {
                metadata["images"] = json!(images.iter().map(|image| json!({
                    "coverType": image.get("coverType"), "remoteUrl": image.get("remoteUrl")
                })).collect::<Vec<_>>());
            }
        }
        metadata["qualityProfileId"] = json!(profile_id);
        metadata["qualityProfile"] = movie["qualityProfile"].clone();
        cache.store(id, &metadata)?;
    }
    Ok(())
}

pub(crate) async fn enrich(
    client: &RadarrClient, vault: Option<AgeVault>, movies: Vec<Value>,
) -> Result<Vec<Value>, &'static str> {
    enrich_cached(client, SharedCache(vault), movies).await
}

async fn enrich_cached(
    client: &RadarrClient, cache: impl MovieCache + Send + 'static, mut movies: Vec<Value>,
) -> Result<Vec<Value>, &'static str> {
    let (mut movies, cached, cache) = tokio::task::spawn_blocking(move || {
        let cached = read_cache(&mut movies, &cache)?;
        Ok::<_, &'static str>((movies, cached, cache))
    }).await.map_err(|_| "Could not read shared movie metadata")??;
    let profiles = if movies.iter().any(|movie| profile_name(movie).is_none()) {
        client.http.get(format!("{}/api/v3/qualityprofile", client.base_url))
            .header("X-Api-Key", client.api_key.clone())
            .header(reqwest::header::ACCEPT, "application/json")
            .send().await.map_err(|_| "Radarr quality profile request failed")?
            .error_for_status().map_err(|_| "Radarr quality profile request failed")?
            .json::<Vec<Value>>().await.map_err(|_| "Invalid Radarr quality profiles")?
    } else { Vec::new() };
    tokio::task::spawn_blocking(move || {
        merge_profiles(&mut movies, &cached, &profiles, &cache)?;
        Ok(movies)
    }).await.map_err(|_| "Could not save shared movie metadata")?
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_quality_profiles.rs"]
mod tests;
