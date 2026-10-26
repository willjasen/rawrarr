use super::*;

fn revision(counter: u64, device: &str, digest: &str) -> Revision {
    Revision { schema_version: 1, root_fingerprint: "a".repeat(64), device_id: device.into(), counter,
        operation: "put".into(), ciphertext_digest: Some(digest.into()), signature: format!("sig-{device}-{counter}") }
}

#[test]
fn accepts_only_bounded_age_ciphertext() {
    assert!(valid_ciphertext(b"age-encryption.org/v1\nabc"));
    assert!(!valid_ciphertext(b"{\"radarrApiKey\":\"secret\"}"));
    assert!(!valid_ciphertext(b"age-encryption.org/v1\n"));
    assert!(!valid_ciphertext(&[AGE_MAGIC, &vec![b'x'; MAX_CONFIG_BYTES]].concat()));
}

#[test]
fn merkle_is_stable_and_conflict_order_ignores_wall_clock() {
    let records = vec![revision(5, "node-b", "b"), revision(5, "node-a", "c"), revision(4, "node-z", "z")];
    assert_eq!(merkle_root(&records), merkle_root(&records));
    assert_eq!(winner(&records).unwrap().device_id, "node-b");
    assert_ne!(merkle_root(&records), merkle_root(&records[..2]));
    assert_eq!(merkle_root(&[]), merkle_root(&[]));
}
