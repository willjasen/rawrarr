use super::NodeRole;
#[test]
fn existing_roles_remain_supported() {
    for (text, role) in [
        ("coordinator", NodeRole::Coordinator),
        ("storage", NodeRole::Storage),
        ("metadata", NodeRole::Metadata),
    ] {
        assert_eq!(NodeRole::parse(text), Ok(role));
    }
    for value in ["", "service", "shard", "coordinator,storage", "Metadata"] {
        assert!(NodeRole::parse(value).is_err());
    }
}
#[test]
fn media_is_required_for_legacy_storage_and_optional_for_metadata() {
    assert!(NodeRole::Storage.validate_media(true).is_ok());
    assert!(NodeRole::Storage.validate_media(false).is_err());
    assert!(NodeRole::Metadata.validate_media(false).is_ok());
    assert!(NodeRole::Metadata.validate_media(true).is_ok());
    assert!(NodeRole::Coordinator.validate_media(false).is_ok());
}

#[test]
fn fossil_adds_storage_capability_without_changing_legacy_storage_role() {
    assert!(!NodeRole::Storage.supports_metadata());
    assert!(NodeRole::Storage.supports_storage(false));
    assert!(NodeRole::Metadata.supports_storage(true));
    assert!(!NodeRole::Metadata.supports_storage(false));
    assert!(NodeRole::Metadata.supports_metadata());
    assert!(!NodeRole::Coordinator.supports_metadata());
    assert!(NodeRole::Coordinator.validate_media(true).is_err());
}
