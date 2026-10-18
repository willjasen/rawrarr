//! Cache-first quality profile names in encrypted shared movie metadata.
use crate::{RadarrClient, metadata, secure_config::AgeVault};
use serde_json::{Value, json};

pub(crate) trait MovieCache: Sync {
    fn load(&self, id: u64) -> Result<Option<Value>, &'static str>;
    fn store(&self, id: u64, metadata: &Value) -> Result<(), &'static str>;
}

struct SharedCache(Option<AgeVault>);

impl MovieCache for SharedCache {
    fn load(&self, id: u64) -> Result<Option<Value>, &'static str> {
        if let Some(vault) = &self.0 {
            return vault
                .load_movie_metadata(id)
                .map_err(|_| "Could not read shared movie metadata");
        }
        if AgeVault::metadata_dir()
            .join("movies/text")
            .join(format!("{id}.json.aged"))
            .exists()
        {
            return Err("Unlock the age vault to read shared movie quality profiles");
        }
        Ok(None)
    }

    fn store(&self, id: u64, metadata: &Value) -> Result<(), &'static str> {
        let bytes = serde_json::to_vec(metadata).map_err(|_| "Invalid movie metadata")?;
        match &self.0 {
            Some(vault) => vault.store_movie_metadata(id, &bytes),
            None => return Err("Unlock the age vault to save shared movie metadata"),
        }
        .map_err(|_| "Could not save quality profiles to shared movie metadata")
    }
}

fn profile_name(metadata: &Value) -> Option<&str> {
    metadata
        .get("qualityProfile")
        .and_then(|value| {
            value
                .as_str()
                .or_else(|| value.get("name").and_then(Value::as_str))
        })
        .filter(|name| !name.trim().is_empty())
}

fn cached_name<'a>(metadata: &'a Value, movie: &Value) -> Option<&'a str> {
    // A changed assignment must not keep displaying the old profile.
    let saved_id = metadata
        .get("qualityProfileId")
        .and_then(Value::as_u64)
        .or_else(|| {
            metadata
                .pointer("/qualityProfile/id")
                .and_then(Value::as_u64)
        });
    if saved_id.is_some() && saved_id != movie.get("qualityProfileId").and_then(Value::as_u64) {
        return None;
    }
    profile_name(metadata)
}

fn read_cache(
    movies: &mut [Value],
    cache: &impl MovieCache,
) -> Result<Vec<Option<Value>>, &'static str> {
    // Bound cold-cache decryption to eight workers, rather than one process per
    // movie in sequence or thousands of simultaneous subprocesses.
    let chunk_size = movies.len().div_ceil(8).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = movies
            .chunks_mut(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter_mut()
                        .map(|movie| {
                            let metadata = match movie
                                .get("tmdbId")
                                .and_then(Value::as_u64)
                                .filter(|id| *id > 0)
                            {
                                Some(id) => cache.load(id)?,
                                None => None,
                            };
                            if let Some(name) = metadata
                                .as_ref()
                                .and_then(|metadata| cached_name(metadata, movie))
                            {
                                movie["qualityProfile"] = json!({"name": name});
                            }
                            Ok(metadata)
                        })
                        .collect::<Result<Vec<_>, &'static str>>()
                })
            })
            .collect();
        let mut result = Vec::with_capacity(workers.len() * chunk_size);
        for worker in workers {
            let values: Vec<Option<Value>> = worker
                .join()
                .map_err(|_| "Could not read shared movie metadata")??;
            result.extend(values);
        }
        Ok(result)
    })
}

fn merge_profiles(
    movies: &mut [Value],
    cached: &[Option<Value>],
    profiles: &[Value],
    cache: &impl MovieCache,
) -> Result<(), &'static str> {
    for (movie, saved) in movies.iter_mut().zip(cached) {
        let Some(id) = movie.get("tmdbId").and_then(Value::as_u64).filter(|id| *id > 0) else {
            continue;
        };
        let name = profile_name(movie).map(str::to_owned).or_else(|| {
            let profile_id = movie.get("qualityProfileId").and_then(Value::as_u64);
            profiles
                .iter()
                .find(|profile| {
                    profile_id.is_some() && profile.get("id").and_then(Value::as_u64) == profile_id
                })
                .and_then(|profile| profile.get("name").and_then(Value::as_str))
                .filter(|name| !name.trim().is_empty())
                .map(str::to_owned)
        });
        if let Some(name) = &name { movie["qualityProfile"] = json!({"name": name}); }
        let mut metadata = metadata::arr_text_record("radarr", "tmdbId", id, movie);
        if let Some(saved) = saved.as_ref().filter(|saved| saved.is_object()) {
            // Preserve locally derived values that are not present in Radarr's API record.
            if let Some(artwork_source) = saved.get("artworkSource") {
                metadata["artworkSource"] = artwork_source.clone();
            }
        }
        if let Some(name) = name {
            metadata["qualityProfile"] = json!({
                "id": movie.get("qualityProfileId").and_then(Value::as_u64),
                "name": name,
            });
        }
        if saved.as_ref() != Some(&metadata) {
            cache.store(id, &metadata)?;
        }
    }
    Ok(())
}

pub(crate) async fn enrich(
    client: &RadarrClient,
    vault: Option<AgeVault>,
    movies: Vec<Value>,
) -> Result<Vec<Value>, &'static str> {
    enrich_cached(client, SharedCache(vault), movies).await
}

async fn enrich_cached(
    client: &RadarrClient,
    cache: impl MovieCache + Send + 'static,
    mut movies: Vec<Value>,
) -> Result<Vec<Value>, &'static str> {
    let (mut movies, cached, cache) = tokio::task::spawn_blocking(move || {
        let cached = read_cache(&mut movies, &cache)?;
        Ok::<_, &'static str>((movies, cached, cache))
    })
    .await
    .map_err(|_| "Could not read shared movie metadata")??;
    let profiles = if movies.iter().any(|movie| profile_name(movie).is_none()) {
        client
            .http
            .get(format!("{}/api/v3/qualityprofile", client.base_url))
            .header("X-Api-Key", client.api_key.clone())
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|_| "Radarr quality profile request failed")?
            .error_for_status()
            .map_err(|_| "Radarr quality profile request failed")?
            .json::<Vec<Value>>()
            .await
            .map_err(|_| "Invalid Radarr quality profiles")?
    } else {
        Vec::new()
    };
    tokio::task::spawn_blocking(move || {
        merge_profiles(&mut movies, &cached, &profiles, &cache)?;
        Ok(movies)
    })
    .await
    .map_err(|_| "Could not save shared movie metadata")?
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_quality_profiles.rs"]
mod tests;
