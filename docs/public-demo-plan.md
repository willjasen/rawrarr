# Public Rawrarr demo plan

**Status:** Planned; no demo mode is implemented by this document.
**Date:** 2026-10-20

## Goal

Let a prospective user explore representative Rawrarr screens and workflows in a public browser session, without creating an account, registering a passkey, or connecting their own services. Keep the existing public landing page as the product introduction and entry point; the demo is an optional, interactive next step.

The demo should explain the product as it exists today. It must distinguish working prototype capabilities from planned product direction, and must not imply that the visitor has created a real Rawrarr instance.

## Recommended experience

- Add a clearly labeled **Try the demo** action to the public landing page. Open a dedicated demo route or a separate demo deployment; do not change the authenticated workspace's `/` route.
- Start directly in a pre-populated, read-only sample workspace. Do not ask for a name, email, password, passkey, age identity, or other personal information.
- Provide a small, curated tour of representative screens: workspace overview, sample media library, device/node roles and statuses, and privacy/data-placement explanations. Only include screens that can be presented accurately with fixed sample content.
- Give visitors lightweight interactions such as navigation, filtering, opening a sample detail view, and expanding explanations. Keep the dataset and supported actions bounded; do not offer controls that appear to run real scans, alter nodes, configure services, or change account settings.
- Keep a persistent **Demo data** label and a clear **Back to Rawrarr** / **Sign in** path. Explain that edits and actions are simulated and that no account or live services are connected.
- Provide an explicit way to exit the demo and return to the public landing page. Do not make exiting depend on clearing browser storage.

## Data and isolation

Use deterministic, synthetic fixtures bundled with the demo UI (or a dedicated demo-only data provider). Do not read from production tables, subscriber `/shared` data, `/media`, `/data`, PostgreSQL, real Radarr/Sonarr services, or node APIs. Do not seed sample records into the production database.

Make the demo read-only by construction: expose only the fixture-backed views and actions that are explicitly simulated. Do not establish a demo account, issue a passkey session, set an authenticated session cookie, unlock a vault, or call production mutation endpoints. Keep demo state in memory where practical; any optional UI preferences stored by the browser must be non-sensitive, clearly demo-scoped, and disposable.

Keep the demo and signed-in workspace as distinct modes. A route or client-side flag must never grant access to authenticated pages or APIs. Existing authentication and authorization checks remain authoritative, and demo requests must not be able to select arbitrary upstream node URLs or access subscriber-owned resources.

## Sample content and product truthfulness

Create fictional names, titles, node names, counts, images, and statuses. Use artwork only when it is appropriately licensed for redistribution; otherwise use project-owned illustrations, generated placeholders, or neutral placeholders. Do not copy production screenshots, subscriber records, API responses, paths, credentials, or identifiers into fixtures.

Label capability claims at the point they appear:

- **Available in the current prototype:** accurately describe only features verified against current product documentation and implementation.
- **Planned:** identify roadmap/product-direction items as planned, not as functioning demo features.
- **Simulated:** identify demo-only behavior and sample status as fictional.

Keep security explanations aligned with [Rawrarr security](SECURITY.md): explain privacy goals and meaningful coordinator-visible metadata, and do not describe the demo as a security certification or claim that incomplete flows are fully end-to-end secure.

## Scope boundaries

**Included:** public access with no signup; synthetic sample data; browseable representative screens; restrained read-only interactions; a short orientation to Rawrarr's roles, privacy boundaries, and current maturity.

**Not included in this plan:** creating accounts or trials; passkey or age-identity setup; connecting real services; modifying production state; user-generated demo data; multi-user collaboration; persistence across visitors; a public API or unrestricted backend; analytics that identify visitors.

## Delivery plan

1. **Choose the demo surface.** Confirm whether this belongs in the existing coordinator web app behind a dedicated public route or in a separately deployed static frontend. Preserve the existing landing-page and signed-in route behavior.
2. **Define a representative tour.** Select a small number of screens with accurate, useful content. For each screen, specify its synthetic fixture shape, permitted interactions, and any limitations or planned capabilities that must be labeled.
3. **Build the isolated fixture experience.** Implement fixtures and demo-only navigation. Ensure the demo cannot call subscriber-data, node, service-integration, or mutation APIs.
4. **Verify access and presentation.** Test signed-out entry and exit, keyboard and narrow-screen navigation, fixture-only rendering, and unchanged passkey sign-in and authenticated workspace behavior. Test that protected APIs remain protected when called from the demo.
5. **Publish deliberately.** Review public copy and redistributed assets, deploy only after confirming the demo contains no production data or secrets, and monitor errors without recording sensitive visitor or subscriber data.

## Acceptance criteria

- A visitor can open the demo from the public landing page and explore it without signing up or supplying identity or service credentials.
- Every displayed record and status is synthetic, and the interface visibly identifies the experience as a demo.
- Supported interactions are read-only or explicitly simulated; no demo interaction changes subscriber, node, service, or production database state.
- The demo cannot authenticate as a user, unlock encrypted data, or bypass existing API authorization.
- Landing-page access, passkey sign-in, and the authenticated workspace continue to behave as before.
- Public copy distinguishes implemented, planned, and simulated behavior and accurately describes Rawrarr's privacy/security limitations.
- The demo is usable with keyboard navigation and on narrow screens; its visual assets are safe to publish.

## Decisions to make before implementation

- Whether to host the demo inside the coordinator app or as a separate static site. Prefer a separately deployed static experience if it can represent the selected screens without weakening existing route or authorization boundaries.
- Which current screens are accurate and valuable enough to include in the first tour.
- Whether the demo should support only ephemeral in-memory state or also remember harmless UI preferences in the visitor's browser.
- Which project-owned or redistributable visual assets should represent sample media.

Resolve these choices before implementation; this plan does not select a hosting topology or promise a launch date.
