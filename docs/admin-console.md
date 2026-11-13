# Rawrarr admin console

## Paused preview

The standalone sample admin-console preview is paused. Its Dockerfile and Compose
launcher have been removed, so this repository cannot build or start the preview.
The static mockup files remain as inactive design source only; they are not served
by the Rawrarr coordinator or any other application service.

This source change does not stop a container that was already started from an
older revision. Inspect its Compose labels and stop only the standalone preview
service on that host; do not change coordinator, subscriber, or Serve routes.
No local `rawrarr-admin` container was present when this change was made.

Before reactivating the preview or connecting it to live data, provide a reviewed
non-root runtime image, a supported launch path, and administrator authentication
and authorization enforced by the service. A browser-only gate or localhost/
Tailscale network boundary is not sufficient. Give any reporting service a
least-privilege database role and expose only the minimum aggregate/account
fields needed; do not reuse the coordinator's write credential. Add audit and
access-expiry behavior before production user details are displayed.

## Reporting contract

The first live reporting version should derive account totals from `rawrarr_users` and must distinguish facts from unavailable features. The current schema does not track trial or subscription entitlements, and the documented trial/subscription model remains proposed. Do not infer either status from account age or show sample financial amounts as live data. Integrate those metrics only after billing/trial state is implemented and has a defined source of truth.

The planned console has no mutable account controls. Keep future admin actions
out of the initial live reporting release unless their authorization, audit
trail, confirmation, and recovery behavior are separately designed.
