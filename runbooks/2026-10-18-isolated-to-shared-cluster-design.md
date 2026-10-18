# Isolated to shared cluster design

## Objective

Record the proposed one-way cluster mode transition and clarify how sharing relates to passkeys, age identities, and signing authority.

## Context

Rawrarr has passkey-authenticated accounts, public age recipients with private identities held by users, and a proposed device-signing root. The product guide did not define how another person could join a cluster without transferring these secrets.

## Decision

- New clusters start **isolated** with one owner and may transition once to **shared**.
- Shared mode is irreversible, including after every invitee is removed, because prior access cannot be reliably undone.
- Each person uses their own passkey and age identity. The owner never transfers those credentials.
- Membership is owner-controlled: only the cluster owner may invite or remove members. Invitees cannot invite others. Every device must enforce this policy before applying membership changes; owner succession and recovery remain explicit future design work.
- Use age's native multi-recipient file format for shared content: it encrypts the payload once and includes a recipient stanza per authorized age recipient. Each stanza can recover the same file key; it need not store that recipient's public ID verbatim. Additional recipients add a small amount of header data, not another payload copy. Rawrarr's authenticated membership record stores public recipient IDs for selecting recipients on future files. No extra AES/KEK layer is needed just to satisfy encrypt-once.
- Each member supplies their own age public recipient; private age identities stay with their owners. Adding a recipient to existing ciphertext requires safely updating its wrapped file key/header, not re-encrypting its payload; this operation needs an explicitly supported implementation. New files target only current members.
- Removing a recipient cannot revoke keys or plaintext already obtained. Reusing ciphertext in a new isolated cluster is safe only if the former member never got usable decryption material; otherwise old objects remain potentially accessible to them.
- A person who wants to return to isolated operation creates a new cluster with a new age identity and signing root. That creates a new security domain; it does not reverse the shared cluster's history or revoke prior access.
- The cluster root signing private key remains with its creator and is never an invite credential. Shared governance requires member-scoped signing keys and root-authorized grants, plus designed recovery, succession, revocation, and compromise handling.
- This is a design direction only; multi-person sharing and the transition are not implemented.

## Actions and verification

- Added the decision to the product design guide.
- Recorded the user's constraint that object data is encrypted once; age natively supports one payload with multiple recipient stanzas.
- Clarified in the plan, product guide, and roadmap that age stores only per-file recipient-encrypted key stanzas in the ciphertext header; there is no persistent cluster-wide pre-shared key in `/shared`.
- Corrected the mechanism after checking the age format: it natively supports multiple recipient stanzas around one encrypted payload, so a separate cluster KEK/AES layer is not required for multi-recipient sharing.
- Recorded owner-only membership authority and single-writer commits as the policy for a possible MLS-over-Syncthing design.
- Added a dedicated shared-cluster plan and roadmap milestones covering protocol evaluation, age compatibility, owner-only invites, revocation, cleanup reporting, and recovery decisions.
- Documented the portal cleanup states as pending, node-reported complete, or unconfirmed, separate from the effective access-revocation state.
- Added a genesis-anchored, append-only trust history requirement for isolated and shared clusters, including local head pinning, replicated backups, fork/rollback detection, and explicit lost-history recovery limits.
- Distinguished age-identity loss from signing-root loss: the former can make encrypted metadata/config unrecoverable while media remains; the latter strands trust administration without itself decrypting or erasing content.
- Clarified that a surviving age identity may support root recovery only if explicitly provisioned as a recovery authority; otherwise a replacement root requires explicit device re-enrollment and is a new trust domain.
- Recorded the accepted owner-responsibility model: original media remains unencrypted, age-ID loss can require metadata rebuild, and signing-root loss requires explicit re-enrollment under a new chain; Rawrarr has no secret-recovery bypass.
- Reviewed the existing security, device-signing, and cloud-access boundaries. No implementation or deployment changed.
- `git diff --check` passed after clarifying the per-file age key model and owner-managed key-loss resets. No runtime behavior was exercised because this is documentation only.
- Updated the age-format explanation to distinguish recipient stanzas from public recipient IDs, note modest per-recipient header growth, and identify the authenticated membership record as Rawrarr's source of recipient IDs. `git diff --check` passed; no runtime behavior was exercised.

## Remaining work

- Define the authenticated cluster-mode record and rollback resistance.
- Choose multi-owner grant/revocation rules, invitation expiry, and approval UX.
- Specify key derivation/wrapping formats, membership removal and future-write rekey behavior, historic ciphertext handling, root backup/recovery, owner succession, and compromised-root recovery.
- Update threat model, schemas, roadmap, and tests before implementation.
- Resolve the open choices listed in [the shared cluster plan](../docs/shared-cluster-sharing-plan.md) before freezing protocol and migration details.
