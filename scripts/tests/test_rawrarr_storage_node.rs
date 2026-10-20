use super::*;
use chrono::{TimeZone, Utc};

#[test]
fn scan_schedule_defaults_to_daily_at_four_in_its_configured_timezone() {
    let schedule = ScanSchedule::default_for_timezone("America/New_York".to_owned());
    schedule.validate().unwrap();
    assert_eq!(schedule.frequency, ScanFrequency::Daily);
    assert_eq!(schedule.time, "04:00");
    assert_eq!(schedule.time_zone, "America/New_York");
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
fn scan_schedule_rejects_invalid_times_timezones_and_frequency_fields() {
    let mut schedule = ScanSchedule::default_for_timezone("UTC".to_owned());
    schedule.time = "24:00".to_owned();
    assert!(schedule.validate().is_err());
    schedule.time = "04:00".to_owned();
    schedule.time_zone = "Not/AZone".to_owned();
    assert!(schedule.validate().is_err());
    schedule.time_zone = "UTC".to_owned();
    schedule.frequency = ScanFrequency::Weekly;
    assert!(schedule.validate().is_err());
    schedule.weekday = Some(ScanWeekday::Monday);
    schedule.validate().unwrap();
    schedule.frequency = ScanFrequency::Daily;
    assert!(schedule.validate().is_err());
}

#[test]
fn next_daily_scan_uses_local_time_and_advances_after_the_due_time() {
    let schedule = ScanSchedule::default_for_timezone("America/New_York".to_owned());
    let now = Utc.with_ymd_and_hms(2026, 10, 20, 7, 0, 0).unwrap();
    let next = next_scheduled_run(&schedule, now).unwrap();
    assert_eq!(next, Utc.with_ymd_and_hms(2026, 10, 20, 8, 0, 0).unwrap());
    let after = next_scheduled_run(&schedule, next).unwrap();
    assert_eq!(after, Utc.with_ymd_and_hms(2026, 10, 21, 8, 0, 0).unwrap());
}

#[test]
fn next_weekly_scan_uses_selected_day_and_skips_a_dst_gap() {
    let schedule = ScanSchedule {
        frequency: ScanFrequency::Weekly,
        time: "04:00".to_owned(),
        time_zone: "America/New_York".to_owned(),
        weekday: Some(ScanWeekday::Monday),
    };
    let now = Utc.with_ymd_and_hms(2026, 10, 20, 7, 0, 0).unwrap();
    assert_eq!(
        next_scheduled_run(&schedule, now).unwrap(),
        Utc.with_ymd_and_hms(2026, 10, 26, 8, 0, 0).unwrap()
    );

    let dst_schedule = ScanSchedule {
        frequency: ScanFrequency::Daily,
        time: "02:30".to_owned(),
        time_zone: "America/New_York".to_owned(),
        weekday: None,
    };
    let before_dst = Utc.with_ymd_and_hms(2026, 3, 7, 8, 0, 0).unwrap();
    assert_eq!(
        next_scheduled_run(&dst_schedule, before_dst).unwrap(),
        Utc.with_ymd_and_hms(2026, 3, 9, 6, 30, 0).unwrap()
    );
}

#[test]
fn scan_schedule_round_trips_through_node_local_json() {
    let path = std::env::temp_dir().join(format!(
        "rawrarr-scan-schedule-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let state = PersistedScanSchedule {
        schedule: ScanSchedule {
            frequency: ScanFrequency::Weekly,
            time: "04:30".to_owned(),
            time_zone: "America/New_York".to_owned(),
            weekday: Some(ScanWeekday::Wednesday),
        },
        last_scheduled_run_at: Some(Utc.with_ymd_and_hms(2026, 10, 21, 8, 30, 0).unwrap()),
    };
    save_schedule(&path, &state).unwrap();
    let persisted: PersistedScanSchedule =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let _ = std::fs::remove_file(path);
    assert_eq!(persisted.schedule, state.schedule);
    assert_eq!(persisted.last_scheduled_run_at, state.last_scheduled_run_at);
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
