---
title: "REQ-010: Production Deployment (GCP)"
type: requirement
version: "1.0"
date: 2026-08-17
status: Active
tags: [requirement, deployment, operations, gcp]
related: ["[[ADR-015-gcp-single-vm-deployment]]", "[[ADR-014-cookie-session-authentication]]", "[[REQ-009-v1.0]]"]
---

# REQ-010: Production Deployment (GCP)
_Version: 1.0_

## Raw Requirement (plain language)
> can you help to deploy all thing on gcp

## Live environment

| | |
|---|---|
| **URL** | **https://34-124-161-138.nip.io** |
| Login | `admin` / password in Secret Manager (`admin-password`) |
| GCP project | `gother-price-intel` |
| VM | `price-app`, `e2-small`, Debian 12, `asia-southeast1-a` |
| Static IP | `34.124.161.138` |
| Images | `asia-southeast1-docker.pkg.dev/gother-price-intel/app/{backend,frontend}` |
| App directory on VM | `/opt/price-app` |

## Deploying

```bash
cd gother-price-code
./deploy.sh                # build on Cloud Build, roll out, verify
./deploy.sh --no-build     # roll out the current :latest without rebuilding
```

The script refuses to report success unless an unauthenticated `/api/hotels` still returns **401** — a rollout that loses the auth layer must not pass silently.

**Rollback:** images are tagged with the git short SHA as well as `latest`.
```bash
gcloud compute ssh price-app --project=gother-price-intel --zone=asia-southeast1-a --tunnel-through-iap
cd /opt/price-app
sudo sed -i 's|/backend:latest|/backend:<previous-sha>|' .env
sudo docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

**SSH** is via IAP only; there is no public port 22.
```bash
gcloud compute ssh price-app --project=gother-price-intel --zone=asia-southeast1-a --tunnel-through-iap
```

## Configuration

Secrets live in **Secret Manager** and are never committed: `jwt-secret`, `admin-password`, `postgres-password`, `serpapi-key`, `gemini-api-key`. The VM's `.env` (mode 600, `/opt/price-app/.env`) is generated from them.

Production-only settings, on top of the normal application config:

| Variable | Value | Why |
|---|---|---|
| `COOKIE_SECURE` | `true` | Session cookie is HTTPS-only. Must stay `false` locally, where a Secure cookie would never be sent and login would fail with no visible cause |
| `ALLOWED_ORIGIN` | `https://34-124-161-138.nip.io` | Names the exact CORS origin. Must be paired with an explicit method list — a wildcard alongside credentials makes `tower-http` panic at startup |
| `POSTGRES_PASSWORD` | alphanumeric only | **Must not contain `/` or `+`.** A `/` in a URL's userinfo terminates the authority component and `DATABASE_URL` parses with a corrupt port |
| `ADMIN_PASSWORD` | from Secret Manager | Set before first boot so the `admin1234!` default is never created on a public host |

## Security posture

- **Firewall:** only `tcp:80,443` from `0.0.0.0/0`. Postgres, Redis, RabbitMQ and the backend publish no host ports at all. The default network's public SSH and RDP rules were deleted; SSH is IAP-only.
- **TLS:** Let's Encrypt via Caddy, auto-renewing. The certificate persists in the `caddy_data` volume — **do not `docker compose down -v`** in production; it discards the certificate (and the database).
- **VM service account** holds only `secretmanager.secretAccessor`, `artifactregistry.reader`, `logging.logWriter`.
- The site is publicly reachable and was probed by scanners within a minute of going up. Authentication is the only barrier.

## Accepted risks

| Risk | Status |
|---|---|
| **No backups** | **Declined by stakeholder.** A lost disk loses all price history permanently — it cannot be re-scraped, because you cannot scrape the past. Reversible at any time with a `pg_dump` cron to GCS |
| Single point of failure | One VM; a zone outage or bad disk means rebuild |
| OS patching | `unattended-upgrades` not configured |
| Session revocation | None; a stolen cookie is valid up to 12 hours (REQ-009) |
| nip.io rate limits | Shared Let's Encrypt limit; certificate is requested once and persisted |

## Verification performed (2026-08-17)
- [x] HTTPS with a valid Let's Encrypt certificate; HTTP 308-redirects to HTTPS
- [x] `5432 / 6379 / 5672 / 15672 / 8080 / 22` all closed from the internet
- [x] `/api/health` 200 without a cookie; `/api/hotels` **401** without a cookie
- [x] Login with the Secret Manager password → 200, cookie carries `HttpOnly; SameSite=Strict; **Secure**`
- [x] `admin1234!` → **401** (never created)
- [x] Data migrated intact: 23 hotels, 2 groups, 594 price rows, 37 jobs, 1 schedule, 26 migrations
- [x] Exactly one backend container — the scheduler-singleton constraint holds
- [x] End-to-end scrape through the public URL: job `completed`, 594 → 601 price rows
- [x] VM reboot: stack returned in ~45s, certificate reused rather than re-issued

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-17 | Initial — first production deployment |
