use super::*;
#[test]
fn series_uses_episode_statistics_and_profile_id() {
    let item = normalize(
        serde_json::json!({"title":"Fixture", "tvdbId":42, "qualityProfileId":7,
        "statistics":{"episodeFileCount":3,"episodeCount":8,"sizeOnDisk":1234}}),
        &[serde_json::json!({"id":7,"name":"HD"})],
    );
    assert_eq!(item["episodeFileCount"], 3);
    assert_eq!(item["episodeCount"], 8);
    assert_eq!(item["sizeOnDisk"], 1234);
    assert_eq!(item["qualityProfile"], "HD");
    assert_eq!(item["hasFile"], true);
    assert_eq!(item["tvdbId"], 42);
    assert!(item.get("tmdbId").is_none());
}
#[test]
fn missing_statistics_are_zero_and_missing_profile_is_null() {
    let item = normalize(serde_json::json!({"title":"Empty"}), &[]);
    assert_eq!(item["hasFile"], false);
    assert_eq!(item["episodeCount"], 0);
    assert!(item["qualityProfile"].is_null());
}

#[test]
fn scrape_targets_only_series_with_a_tvdb_id_and_sonarr_poster_url() {
    assert!(has_poster(&serde_json::json!({"images":[{"coverType":"poster","url":"/MediaCover/1/poster.jpg"}]})));
    assert!(!has_poster(&serde_json::json!({"images":[{"coverType":"banner","url":"/MediaCover/1/banner.jpg"}]})));
    assert!(!has_poster(&serde_json::json!({"images":[{"coverType":"poster"}]})));
}

#[test]
fn corrupt_existing_artwork_is_rescraped_but_storage_errors_are_not_hidden() {
    let valid = secure_config::EncryptedArtwork {
        content_type: "image/jpeg".into(),
        bytes: vec![1, 2, 3],
    };
    assert!(existing_artwork_is_valid(Ok(Some(valid))).unwrap());
    assert!(!existing_artwork_is_valid(Ok(None)).unwrap());
    assert!(!existing_artwork_is_valid(Err(secure_config::VaultError::DecryptionFailed)).unwrap());
    assert!(existing_artwork_is_valid(Err(secure_config::VaultError::StorageFailed)).is_err());
}

#[test]
fn tvdb_remote_poster_urls_are_restricted_to_https_artwork_paths() {
    assert!(validated_tvdb_image_url("https://artworks.thetvdb.com/banners/posters/show.jpg").is_ok());
    assert!(validated_tvdb_image_url("http://artworks.thetvdb.com/banners/posters/show.jpg").is_err());
    assert!(validated_tvdb_image_url("https://evil.example/banners/posters/show.jpg").is_err());
    assert!(validated_tvdb_image_url("https://artworks.thetvdb.com/other/show.jpg").is_err());
    assert!(validated_tvdb_image_url("https://user@artworks.thetvdb.com/banners/posters/show.jpg").is_err());
}

#[test]
fn failed_episode_lookup_preserves_cached_details_and_does_not_block_metadata() {
    let previous = serde_json::json!({"episodes":[{"id": 7, "title":"Cached episode"}]});
    let mut metadata = serde_json::json!({"provider":"sonarr"});
    attach_episodes(&mut metadata, None, Some(&previous));
    assert_eq!(metadata["episodes"][0]["title"], "Cached episode");

    let mut first_scrape = serde_json::json!({"provider":"sonarr"});
    attach_episodes(&mut first_scrape, None, None);
    assert_eq!(first_scrape["episodes"], serde_json::json!([]));
}

#[test]
fn old_config_remains_compatible() {
    let config: secure_config::PersistedConfig =
        serde_json::from_str(r#"{"radarr_url":"","radarr_api_key":""}"#).unwrap();
    assert!(config.sonarr.is_none());
}
