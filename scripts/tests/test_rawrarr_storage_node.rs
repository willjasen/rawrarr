use super::*;

#[test]
fn scan_interval_defaults_to_six_hours() {
    assert_eq!(
        parse_scan_interval(None).unwrap(),
        Some(Duration::from_secs(21_600))
    );
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
