use super::NodeRole;
#[test]
fn three_roles_only() {
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
fn media_is_exclusive_to_storage() {
    assert!(NodeRole::Storage.validate_media(true).is_ok());
    assert!(NodeRole::Storage.validate_media(false).is_err());
    for role in [NodeRole::Coordinator, NodeRole::Metadata] {
        assert!(role.validate_media(false).is_ok());
        assert!(role.validate_media(true).is_err());
    }
}
