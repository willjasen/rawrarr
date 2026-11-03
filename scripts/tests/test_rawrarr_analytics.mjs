import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [coordinator, analyticsCompose, analyticsServicesCompose, publicCompose, do3Compose, internalDashboardCompose, internalDashboardCaddy, publicWebsite, websiteAnalytics, netlify] = await Promise.all([
  read('rust-arr/src/coordinator.rs'),
  read('docker-compose.analytics.yml'),
  read('docker-compose.analytics-services.yml'),
  read('docker-compose.coordinator.yml'),
  read('deploy/do3/docker-compose.internal-coordinator.yml'),
  read('deploy/do3/docker-compose.analytics-dashboard-internal.yml'),
  read('deploy/do3/analytics-dashboard-internal.Caddyfile'),
  read('website/index.html'),
  read('website/analytics.js'),
  read('netlify.toml'),
]);

test('coordinator analytics are opt-in and only emit a fixed page label', () => {
  assert.match(coordinator, /RAWARR_ANALYTICS_SITE_ID/);
  assert.match(coordinator, /RAWARR_ANALYTICS_UPSTREAM/);
  assert.match(coordinator, /url\.host_str\(\) == Some\("umami"\)/);
  assert.match(coordinator, /url\.port\(\) == Some\(3000\)/);
  assert.match(coordinator, /Method::GET, "script\.js"/);
  assert.match(coordinator, /Method::POST, "api\/send"/);
  assert.match(coordinator, /get\("true-client-ip"\)[\s\S]*?get\("x-forwarded-for"\)/);
  assert.match(coordinator, /outbound\.header\("x-forwarded-for", value\)/);
  assert.doesNotMatch(coordinator, /header::X_FORWARDED_FOR/);
  assert.match(coordinator, /data-auto-track="false"/);
  assert.match(coordinator, /website:'\{site_id\}',url:'\/coordinator',title:'Rawrarr coordinator'/);
  assert.doesNotMatch(coordinator, /umami\.track\(\)/);
});

test('coordinator analytics settings default off and do not start Umami', () => {
  assert.match(analyticsCompose, /RAWARR_ANALYTICS_UPSTREAM: \$\{RAWARR_ANALYTICS_UPSTREAM:-\}/);
  assert.match(analyticsCompose, /RAWARR_ANALYTICS_SITE_ID: \$\{RAWARR_ANALYTICS_SITE_ID:-\}/);
  assert.doesNotMatch(analyticsCompose, /^  umami(-postgres)?:/m);
});

test('Umami and PostgreSQL have a separate explicit opt-in Compose file', () => {
  assert.match(analyticsServicesCompose, /image: ghcr\.io\/umami-software\/umami:3\.4\.1/);
  assert.match(analyticsServicesCompose, /127\.0\.0\.1:\$\{RAWARR_ANALYTICS_PORT:-7300\}:3000/);
  assert.match(analyticsServicesCompose, /umami-postgres-data:/);
  assert.match(analyticsServicesCompose, /external: true/);
  assert.match(analyticsServicesCompose, /name: \$\{RAWARR_ANALYTICS_NETWORK/);
  assert.match(analyticsServicesCompose, /RAWARR_ANALYTICS_DB_PASSWORD/);
  assert.match(analyticsServicesCompose, /DISABLE_TELEMETRY: "1"/);
  assert.doesNotMatch(analyticsServicesCompose, /rawrarr-postgres-data|rawrarr-internal/);
});

test('coordinator overlay leaves analytics upstream opt-in across deployment variants', () => {
  assert.match(analyticsCompose, /RAWARR_ANALYTICS_UPSTREAM: \$\{RAWARR_ANALYTICS_UPSTREAM:-\}/);
  assert.match(analyticsCompose, /RAWARR_ANALYTICS_SITE_ID: \$\{RAWARR_ANALYTICS_SITE_ID:-\}/);
  assert.match(do3Compose, /RAWARR_ROLE: coordinator/);
  assert.match(publicCompose, /RAWARR_ROLE: coordinator/);
});

test('internal dashboard Caddy route stays in the Tailscale namespace and separates dashboard from coordinator', () => {
  assert.match(internalDashboardCompose, /network_mode: service:tailscale/);
  assert.match(internalDashboardCompose, /container_name: rawrarr-int-caddy/);
  assert.match(internalDashboardCompose, /container_name: rawrarr-int-umami/);
  assert.match(internalDashboardCompose, /container_name: rawrarr-int-umami-postgres/);
  assert.match(internalDashboardCompose, /image: rawrarr-umami:559e3c607c23-basepath-analytics/);
  assert.match(internalDashboardCaddy, /handle \/analytics\* \{[\s\S]*reverse_proxy umami:3000/);
  assert.match(internalDashboardCaddy, /handle \{[\s\S]*reverse_proxy 127\.0\.0\.1:8080/);
  assert.doesNotMatch(internalDashboardCompose, /ports:/);
});

test('public website analytics use a separate site and send only a fixed page label', () => {
  assert.match(publicWebsite, /data-website-id="72da7a2e-fe1b-4058-b07b-912d526c939e"/);
  assert.match(publicWebsite, /src="\/analytics\/script\.js" data-host-url="\/analytics"/);
  assert.match(publicWebsite, /data-auto-track="false"/);
  assert.match(websiteAnalytics, /website, url: '\/website', title: 'Rawrarr website'/);
  assert.doesNotMatch(websiteAnalytics, /location\.(href|search|pathname)|document\.referrer/);
  assert.match(netlify, /script-src 'self'/);
  assert.match(netlify, /connect-src 'self'/);
  assert.match(netlify, /from = "\/analytics\/script\.js"[\s\S]*?to = "https:\/\/roarlay\.rawrarr\.net\/analytics\/script\.js"[\s\S]*?status = 200/);
  assert.match(netlify, /from = "\/analytics\/api\/send"[\s\S]*?to = "https:\/\/roarlay\.rawrarr\.net\/analytics\/api\/send"[\s\S]*?status = 200/);
  assert.doesNotMatch(netlify, /\/analytics\/\*/, 'only the tracker and event endpoints are proxied');
});
