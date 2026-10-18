use super::*;

#[test]
fn scan_interval_defaults_to_one_day() {
    assert_eq!(
        parse_scan_interval(None).unwrap(),
        Some(Duration::from_secs(86_400))
    );
}

#[test]
fn scan_options_validate_bounds_and_match_requested_sample_files() {
    let options: ScanOptions = serde_json::from_value(serde_json::json!({
        "kind": "movie",
        "minimumSizeBytes": 1_000_000_000u64,
        "maximumSizeBytes": 2_000_000_000u64,
        "maxFiles": 3
    })).unwrap();
    options.validate().unwrap();
    let make_file = |size_bytes| library::MediaFile {
        path: "movie.mkv".to_owned(),
        name: "movie.mkv".to_owned(),
        kind: library::MediaKind::Movie,
        size_bytes,
        modified_at_unix_seconds: Some(1),
    };
    assert!(options.matches(&make_file(1_000_000_000)));
    assert!(options.matches(&make_file(2_000_000_000)));
    assert!(!options.matches(&make_file(999_999_999)));
    assert!(serde_json::from_value::<ScanOptions>(serde_json::json!({"unknown": true})).is_err());
    assert!(serde_json::from_value::<ScanOptions>(serde_json::json!({"maxFiles": 11}))
        .unwrap().validate().is_err());
}

#[test]
fn scan_cache_key_does_not_disclose_relative_paths() {
    assert_eq!(cache_key("Example Film/movie.mkv").len(), 64);
    assert_ne!(cache_key("Example Film/movie.mkv"), "Example Film/movie.mkv");
}

#[test]
fn scan_interval_can_be_disabled_or_configured() {
    assert_eq!(parse_scan_interval(Some("0")).unwrap(), None);
    assert_eq!(
        parse_scan_interval(Some("60")).unwrap(),
        Some(Duration::from_secs(60))
    );
    assert_eq!(
        parse_scan_interval(Some("604800")).unwrap(),
        Some(Duration::from_secs(604_800))
    );
}

#[test]
fn scan_interval_rejects_invalid_or_unsafe_values() {
    for value in ["", "abc", "-1", "1", "59", "604801"] {
        assert!(
            parse_scan_interval(Some(value)).is_err(),
            "accepted {value:?}"
        );
    }
}

#[test]
fn hashes_are_calculated_in_one_pass_with_standard_digests() {
    let root = std::env::temp_dir().join(format!("rawrarr-hash-{}", std::process::id()));
    std::fs::write(&root, b"abc").unwrap();
    let hashes = hash_file(&root).unwrap();
    let _ = std::fs::remove_file(root);
    assert_eq!(hashes.md5, "900150983cd24fb0d6963f7d28e17f72");
    assert_eq!(hashes.sha1, "a9993e364706816aba3e25717850c26c9cd0d89d");
    assert_eq!(
        hashes.sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
