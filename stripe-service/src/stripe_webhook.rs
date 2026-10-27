use hmac::{Hmac, Mac};
use serde::Deserialize;
use std::collections::HashMap;
use sha2::Sha256;
use uuid::Uuid;

const SIGNATURE_TOLERANCE_SECONDS: u64 = 300;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SignatureError {
    Malformed,
    Expired,
    Invalid,
}

pub(crate) fn verify_signature(
    body: &[u8],
    signature_header: &str,
    secret: &str,
    now: u64,
) -> Result<(), SignatureError> {
    let mut timestamp = None;
    let mut signatures = Vec::new();
    for part in signature_header.split(',') {
        let Some((name, value)) = part.trim().split_once('=') else {
            return Err(SignatureError::Malformed);
        };
        match name {
            "t" => {
                if timestamp.is_some() {
                    return Err(SignatureError::Malformed);
                }
                timestamp = Some(value.parse::<u64>().map_err(|_| SignatureError::Malformed)?);
            }
            "v1" => signatures.push(decode_hex(value).ok_or(SignatureError::Malformed)?),
            _ => {}
        }
    }
    let timestamp = timestamp.ok_or(SignatureError::Malformed)?;
    if signatures.is_empty() {
        return Err(SignatureError::Malformed);
    }
    if now.abs_diff(timestamp) > SIGNATURE_TOLERANCE_SECONDS {
        return Err(SignatureError::Expired);
    }

    let mut message = timestamp.to_string().into_bytes();
    message.push(b'.');
    message.extend_from_slice(body);
    for signature in signatures {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC accepts keys of every length");
        mac.update(&message);
        if mac.verify_slice(&signature).is_ok() {
            return Ok(());
        }
    }
    Err(SignatureError::Invalid)
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.is_empty() || value.len() % 2 != 0 {
        return None;
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])?;
            let low = hex_nibble(pair[1])?;
            Some((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct StripeEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub created: u64,
    pub data: StripeEventData,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StripeEventData {
    pub object: StripeEventObject,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StripeEventObject {
    pub id: String,
    pub customer: Option<StripeReference>,
    pub metadata: Option<HashMap<String, String>>,
    pub status: Option<String>,
    pub current_period_end: Option<u64>,
    pub cancel_at_period_end: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum StripeReference {
    Id(String),
    Object { id: String },
}

impl StripeReference {
    pub(crate) fn id(&self) -> &str {
        match self {
            Self::Id(id) | Self::Object { id } => id,
        }
    }
}

impl StripeEventObject {
    pub(crate) fn rawrarr_account_id(&self) -> Result<Option<Uuid>, ()> {
        self.metadata
            .as_ref()
            .and_then(|metadata| metadata.get("rawrarr_account_id"))
            .map(|id| Uuid::parse_str(id).map_err(|_| ()))
            .transpose()
    }

    pub(crate) fn stripe_customer_id(&self) -> Option<&str> {
        self.customer.as_ref().map(StripeReference::id)
    }
}

pub(crate) fn is_billing_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "checkout.session.completed"
            | "customer.subscription.created"
            | "customer.subscription.updated"
            | "customer.subscription.deleted"
            | "customer.subscription.paused"
            | "customer.subscription.resumed"
            | "customer.subscription.trial_will_end"
            | "invoice.paid"
            | "invoice.payment_failed"
            | "invoice.payment_action_required"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signed_header(body: &[u8], secret: &str, timestamp: u64) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(body);
        let signature = mac.finalize().into_bytes();
        let signature = signature.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        format!("t={timestamp},v1={signature}")
    }

    #[test]
    fn accepts_valid_signature_within_tolerance() {
        let body = br#"{"id":"evt_1"}"#;
        let header = signed_header(body, "whsec_test", 1_700_000_000);
        assert_eq!(
            verify_signature(body, &header, "whsec_test", 1_700_000_250),
            Ok(())
        );
    }

    #[test]
    fn rejects_changed_body_and_wrong_secret() {
        let body = br#"{"id":"evt_1"}"#;
        let header = signed_header(body, "whsec_test", 1_700_000_000);
        assert_eq!(
            verify_signature(br#"{"id":"evt_2"}"#, &header, "whsec_test", 1_700_000_000),
            Err(SignatureError::Invalid)
        );
        assert_eq!(
            verify_signature(body, &header, "whsec_other", 1_700_000_000),
            Err(SignatureError::Invalid)
        );
    }

    #[test]
    fn rejects_stale_and_malformed_signatures() {
        let body = br#"{}"#;
        let header = signed_header(body, "whsec_test", 1_700_000_000);
        assert_eq!(
            verify_signature(body, &header, "whsec_test", 1_700_000_301),
            Err(SignatureError::Expired)
        );
        assert_eq!(
            verify_signature(body, "t=nope,v1=abcd", "whsec_test", 1_700_000_000),
            Err(SignatureError::Malformed)
        );
        assert_eq!(
            verify_signature(body, "t=1700000000,v1=xyz", "whsec_test", 1_700_000_000),
            Err(SignatureError::Malformed)
        );
    }

    #[test]
    fn accepts_any_matching_v1_during_secret_rotation() {
        let body = br#"{}"#;
        let header = signed_header(body, "whsec_current", 1_700_000_000);
        let current = header.split_once(',').unwrap().1;
        let mut mac = HmacSha256::new_from_slice(b"whsec_previous").unwrap();
        mac.update(b"1700000000.");
        mac.update(body);
        let previous = mac.finalize().into_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let rotated = format!("t=1700000000,v1={previous},{current}");
        assert_eq!(
            verify_signature(body, &rotated, "whsec_current", 1_700_000_000),
            Ok(())
        );
    }

    #[test]
    fn only_subscription_related_events_enter_the_inbox() {
        assert!(is_billing_event("customer.subscription.updated"));
        assert!(is_billing_event("invoice.payment_failed"));
        assert!(!is_billing_event("payment_intent.succeeded"));
        assert!(!is_billing_event("account.updated"));
    }

    #[test]
    fn extracts_account_owner_and_customer_references() {
        let event: StripeEvent = serde_json::from_str(r#"{
            "id":"evt_123","type":"customer.subscription.updated","created":1700000000,
            "data":{"object":{"id":"sub_123","customer":{"id":"cus_123"},
            "metadata":{"rawrarr_account_id":"9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"},
            "status":"trialing","current_period_end":1700003600,"cancel_at_period_end":false}}
        }"#).unwrap();
        assert_eq!(event.data.object.stripe_customer_id(), Some("cus_123"));
        assert_eq!(event.data.object.rawrarr_account_id().unwrap().unwrap().to_string(), "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d");
        assert_eq!(event.data.object.status.as_deref(), Some("trialing"));
    }

    #[test]
    fn rejects_malformed_rawrarr_account_metadata() {
        let object: StripeEventObject = serde_json::from_str(r#"{"id":"cus_1","metadata":{"rawrarr_account_id":"not-a-uuid"}}"#).unwrap();
        assert!(object.rawrarr_account_id().is_err());
    }
}
