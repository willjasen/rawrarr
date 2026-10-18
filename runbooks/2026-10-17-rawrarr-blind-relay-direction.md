# Rawrarr blind public relay direction

## Objective and initial state

Record the user's future requirement for a public Rawrarr domain that reaches a subscriber's self-hosted node without giving the relay access to node data or configuration. See [device architecture](../docs/device-architecture.md) and [private self-hosted product direction](2026-10-17-rawrarr-private-self-hosted-direction.md).

## Direction recorded

- A subscriber's node initiates an authenticated outbound connection so the node can stay behind NAT.
- The relay routes by an opaque device/instance ID and forwards encrypted traffic. Rawrarr on the node terminates the user's TLS connection and enforces passkey authentication and authorization.
- Do not terminate application TLS at the relay or serve relay-controlled executable UI code for the private session.
- The relay must see some metadata for routing, including online state and transient tunnel selection; it may also observe source IP, connection timing, byte counts, and hostname/SNI. Minimize persistent records and telemetry.
- IDs and routing tickets are not user credentials. The node remains the authority for user identity, trust, and content access.

## Verification and remaining work

Updated the product and device architecture documents. `git diff --check` and relative-link checks passed. This is an architectural requirement only; there is no relay implementation. Design hostname/TLS passthrough, certificate issuance, tunnel authentication, direct-connect fallback, access controls, abuse handling, and offline behavior before implementation. A WebRTC DataChannel is another encrypted transport option, but implementing a normal HTTP application over it would require a browser-side protocol adapter.
