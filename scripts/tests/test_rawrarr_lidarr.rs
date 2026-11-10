use super::*;

#[test]
fn artists_and_releases_keep_provider_and_external_stable_ids() {
    let artist = normalize_artist(
        serde_json::json!({
            "id": 7, "artistName": "Fixture Artist", "foreignArtistId": "musicbrainz-artist",
            "qualityProfileId": 2, "monitored": true, "path": "/music/Fixture Artist",
            "statistics": {"albumCount": 3, "trackCount": 24, "trackFileCount": 20, "sizeOnDisk": 4096}
        }),
        &[serde_json::json!({"id": 2, "name": "Lossless"})],
    )
    .unwrap();
    assert_eq!(artist["provider"], "lidarr");
    assert_eq!(artist["foreignArtistId"], "musicbrainz-artist");
    assert_eq!(artist["albumCount"], 3);
    assert_eq!(artist["trackFileCount"], 20);
    assert_eq!(artist["qualityProfile"], "Lossless");

    let release = normalize_release(
        serde_json::json!({
            "id": 9, "artistId": 7, "title": "Fixture Album",
            "foreignAlbumId": "musicbrainz-release", "releaseDate": "2024-01-02",
            "statistics": {"trackCount": 8, "trackFileCount": 7, "sizeOnDisk": 2048}
        }),
        &[(7, "Fixture Artist".into())].into_iter().collect(),
    )
    .unwrap();
    assert_eq!(release["provider"], "lidarr");
    assert_eq!(release["artistName"], "Fixture Artist");
    assert_eq!(release["foreignAlbumId"], "musicbrainz-release");
    assert_eq!(release["trackCount"], 8);
}

#[test]
fn malformed_artists_and_releases_are_not_exposed_as_valid_inventory() {
    assert!(normalize_artist(serde_json::json!({"artistName":"No ID"}), &[]).is_none());
    assert!(normalize_release(
        serde_json::json!({"id": 4, "title":"No artist"}),
        &Default::default()
    )
    .is_none());
}

#[test]
fn service_validation_rejects_non_lidarr_instances() {
    let status = RadarrStatus {
        app_name: "Sonarr".into(),
        version: "4.0.0".into(),
        url_base: String::new(),
    };
    assert!(validate_status(&status).is_err());
}
