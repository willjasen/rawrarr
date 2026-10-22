//! Versioned ML-DSA-65 primitives shared by enrollment and shared records.
//!
//! This module intentionally handles public-key verification only. Root private
//! material is decrypted and used by the browser; node seeds remain in the
//! node-local data directory.
use ml_dsa::{EncodedVerifyingKey, MlDsa65, Signature, VerifyingKey, signature::Verifier};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Digest, Sha256};

pub const ALGORITHM: &str = "ML-DSA-65";
const ROOT_DOMAIN: &[u8] = b"rawrarr-instance-root-v1\0";
const NODE_DOMAIN: &[u8] = b"rawrarr-device-signing-key-v1\0";

pub fn fingerprint(public_key: &[u8]) -> String {
    fingerprint_with_domain(ROOT_DOMAIN, public_key)
}

pub fn node_fingerprint(public_key: &[u8]) -> String {
    fingerprint_with_domain(NODE_DOMAIN, public_key)
}

fn fingerprint_with_domain(domain: &[u8], public_key: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(public_key);
    digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Canonical JSON matching the browser signing client: recursively sorted object
/// keys, compact arrays/scalars, and serde's JSON escaping.
pub fn canonical_json(value: &serde_json::Value) -> Result<Vec<u8>, serde_json::Error> {
    fn canonical(value: &serde_json::Value, out: &mut Vec<u8>) -> Result<(), serde_json::Error> {
        match value {
            serde_json::Value::Object(fields) => {
                out.push(b'{');
                let mut keys: Vec<_> = fields.keys().collect();
                keys.sort();
                for (index, key) in keys.into_iter().enumerate() {
                    if index > 0 { out.push(b','); }
                    serde_json::to_writer(&mut *out, key)?;
                    out.push(b':');
                    canonical(&fields[key], out)?;
                }
                out.push(b'}');
            }
            serde_json::Value::Array(items) => {
                out.push(b'[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 { out.push(b','); }
                    canonical(item, out)?;
                }
                out.push(b']');
            }
            scalar => serde_json::to_writer(out, scalar)?,
        }
        Ok(())
    }
    let mut output = Vec::new();
    canonical(value, &mut output)?;
    Ok(output)
}

pub fn verify(public_key: &[u8], signature: &[u8], message: &[u8]) -> bool {
    let Ok(encoded_key) = <EncodedVerifyingKey<MlDsa65> as TryFrom<&[u8]>>::try_from(public_key) else { return false; };
    let public_key = VerifyingKey::<MlDsa65>::decode(&encoded_key);
    let Ok(signature) = Signature::<MlDsa65>::try_from(signature) else { return false; };
    public_key.verify(message, &signature).is_ok()
}

/// Verify the coordinator-issued, instance-root-signed node key certificate.
pub fn verify_certificate(certificate: &serde_json::Value, expected_root_fingerprint: &str) -> bool {
    let string = |key: &str| certificate.get(key).and_then(serde_json::Value::as_str);
    if string("algorithm") != Some(ALGORITHM) { return false; }
    let (Some(root_key_text), Some(root_fingerprint), Some(node_key_hex), Some(node_fingerprint), Some(signature_text)) =
        (string("rootPublicKey"), string("rootFingerprint"), string("signingPublicKeyHex"), string("signingFingerprint"), string("signature")) else { return false; };
    if root_fingerprint != expected_root_fingerprint { return false; }
    let Ok(root_key) = BASE64.decode(root_key_text) else { return false; };
    let Some(node_key) = decode_hex(node_key_hex) else { return false; };
    let Ok(signature) = BASE64.decode(signature_text) else { return false; };
    if fingerprint(&root_key) != root_fingerprint || self::node_fingerprint(&node_key) != node_fingerprint { return false; }
    let Some(payload) = certificate.get("signedPayload") else { return false; };
    if payload.get("domain").and_then(serde_json::Value::as_str) != Some("rawrarr-device-approval-v1")
        || payload.get("publicKeyHex").and_then(serde_json::Value::as_str) != Some(node_key_hex)
        || payload.get("nodeId").and_then(serde_json::Value::as_str) != string("nodeId")
        || payload.get("role").and_then(serde_json::Value::as_str) != string("role")
    { return false; }
    let Ok(message) = canonical_json(payload) else { return false; };
    verify(&root_key, &signature, &message)
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() % 2 != 0 { return None; }
    value.as_bytes().chunks_exact(2).map(|pair| {
        let digit = |byte: u8| match byte {
            b'0'..=b'9' => Some(byte - b'0'), b'a'..=b'f' => Some(byte - b'a' + 10), _ => None,
        };
        Some(digit(pair[0])? * 16 + digit(pair[1])?)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ml_dsa::{MlDsa65, SigningKey, signature::{Keypair, Signer}};

    #[test]
    fn canonical_json_is_stable_and_signatures_bind_exact_payload() {
        let left = serde_json::json!({"z": 1, "a": {"y": true, "x": "ok"}});
        let right = serde_json::json!({"a": {"x": "ok", "y": true}, "z": 1});
        let bytes = canonical_json(&left).unwrap();
        assert_eq!(bytes, canonical_json(&right).unwrap());
        assert_eq!(bytes, br#"{"a":{"x":"ok","y":true},"z":1}"#);

        let seed = [7u8; 32];
        let signing_key = SigningKey::<MlDsa65>::from_seed(&seed.into());
        let verifying_key = signing_key.verifying_key();
        let signature = signing_key.sign(&bytes).encode();
        let public_key = verifying_key.encode();
        assert!(verify(public_key.as_ref(), signature.as_ref(), &bytes));
        assert!(!verify(public_key.as_ref(), signature.as_ref(), b"tampered"));
        assert_eq!(fingerprint(public_key.as_ref()).len(), 64);
        assert_eq!(node_fingerprint(public_key.as_ref()).len(), 64);
        assert_ne!(fingerprint(public_key.as_ref()), node_fingerprint(public_key.as_ref()));
    }
}
