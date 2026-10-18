use super::*;
#[test]
fn series_uses_episode_statistics_and_profile_id() {
    let item = normalize(serde_json::json!({"title":"Fixture", "tvdbId":42, "qualityProfileId":7,
        "statistics":{"episodeFileCount":3,"episodeCount":8,"sizeOnDisk":1234}}),
        &[serde_json::json!({"id":7,"name":"HD"})]);
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
fn old_config_remains_compatible() {
    let config: secure_config::PersistedConfig = serde_json::from_str(r#"{"radarr_url":"","radarr_api_key":""}"#).unwrap();
    assert!(config.sonarr.is_none());
}
