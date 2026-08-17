---
title: "ADR-015: Single-VM Docker Deployment on GCP"
type: decision
date: 2026-08-17
status: Accepted
tags: [adr, deployment, infrastructure, gcp, cost]
related: ["[[REQ-010-production-deployment]]", "[[ADR-014-cookie-session-authentication]]", "[[ADR-008-no-silent-mock-fallback]]"]
---

# ADR-015: Single-VM Docker Deployment on GCP

## Context
The platform ran only on a developer laptop. Going live raised the question of how to map five services — Postgres, Redis, RabbitMQ, the Rust backend, the nginx-served React frontend — onto GCP.

One property of the backend narrowed the field before cost or preference entered:

**The backend is a single process running four things.** `main.rs` spawns the HTTP server, the RabbitMQ consumer, the cron scheduler, and the partition manager together. The scheduler (`worker/scheduler.rs`) checks `is_due(cron, last_run_at, now)` and writes `last_run_at` only *after* firing, with no lock, no lease, and no conditional update. Two concurrent instances would therefore both observe "due" and both fire the full 5-window grid.

That is not a theoretical race. The SerpAPI budget is roughly 250 searches/month, one scheduled fire over 20 hotels is 100 searches, and a duplicate fire would silently double it. **Exactly one backend instance may run**, and any deployment target had to guarantee that.

## Decision
**One GCE VM (`e2-small`, Debian 12, `asia-southeast1`) running the existing `docker-compose` stack, fronted by Caddy for automatic HTTPS.**

1. **Single VM.** Satisfies the single-instance constraint by construction rather than by configuration that could later be changed by someone who doesn't know why it's set.
2. **Postgres stays a container.** Cloud SQL was offered and declined on cost. Recorded as a known risk below.
3. **Redis stays a container.** It is a pure read-through cache with a TTL — three call sites, no durable state, and losing it costs only re-scrapes. Memorystore's smallest tier is ~$35/month, more than the rest of the deployment combined, to hold data we are free to lose.
4. **RabbitMQ stays a container.** GCP has no managed AMQP service. Pub/Sub would work but is a rewrite of the `queue/` module, and there is no operational benefit at one instance.
5. **Caddy terminates TLS** at `<ip-with-dashes>.nip.io`, obtaining a Let's Encrypt certificate with no DNS to configure, because nip.io resolves the IP embedded in the hostname. `caddy_data` is a named volume so the certificate survives restarts instead of being re-requested.
6. **Images are built by Cloud Build**, not locally. The development Macs are arm64 and the VM is amd64; a locally built image would not run, and cross-building Rust under emulation is impractically slow.
7. **Only ports 80 and 443 are published**, by Caddy alone. `docker-compose.prod.yml` uses `ports: !reset []` to strip the port publishing the base compose file needs for local development. The GCP firewall enforces the same thing independently.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: One VM, all containers (chosen)** | Zero code changes; single-instance guaranteed; ~$20/month | Single point of failure; we own backups and OS patching |
| B: VM + Cloud SQL | Managed backups and point-in-time recovery for the irreplaceable data | ~$10/month more; offered and declined |
| C: Cloud Run + Cloud SQL + Memorystore + Pub/Sub | Fully managed, autoscaling, free TLS | Needs `min=max=1` to protect the scheduler — a VM with extra steps — *plus* a Pub/Sub rewrite, and ~$80/month. Autoscaling is the one feature we must switch off |
| D: GKE | Room to grow | An entire control plane for one container set that must not scale |

## Consequences

### Positive
- Deployed with **no application code changes** beyond two production settings (`COOKIE_SECURE`, `ALLOWED_ORIGIN`).
- ~$20/month all in.
- The dev/prod gap is one overlay file, so local `docker compose up` still reproduces production closely.
- Redeploy is one script; rollback is repointing the image tag to a previous SHA.

### Negative / Trade-offs
- **No automated backups.** Explicitly declined. A deleted disk loses all price history, and price history **cannot be re-created — you cannot scrape the past.** This is the single largest risk carried by the deployment and it is accepted knowingly, not overlooked.
- **Single point of failure.** A zone outage or a bad boot disk takes the site down until it is rebuilt.
- **We own OS patching.** `unattended-upgrades` is not configured.
- **The URL is guessable and public.** It appeared in Certificate Transparency logs immediately; scanners probed `/.git/config` and `/graphql` within a minute of the certificate issuing. Authentication (ADR-014) is genuinely the only thing protecting the data, which is why the seeded default password was never allowed to exist in production — `ADMIN_PASSWORD` is set from Secret Manager before first boot.
- **nip.io is not on the Public Suffix List**, so all `*.nip.io` share one Let's Encrypt rate limit (raised to 250,000/week). Issuance can in principle be refused; the persisted `caddy_data` volume means we ask once.

## Two bugs this shook out, both worth keeping
Recorded because each failed *at startup in production* rather than at compile time:

1. **A `/` in the generated Postgres password.** The password came from `openssl rand -base64`, and a `/` in a URL's userinfo terminates the authority component, so `DATABASE_URL` parsed with a corrupt port — surfacing as `invalid port number`, which names neither the password nor the URL. Production credentials are now generated **alphanumeric-only** so they cannot break a connection string.
2. **`allow_credentials(true)` with `allow_methods(Any)`.** Forbidden by the CORS spec, and `tower-http` enforces it by **panicking when the layer is constructed** — so the server died after reporting four successful subsystem connections. Fixed by naming the methods explicitly, and `build_cors` is now a pure function with tests that construct both branches, since "does not panic" is the property that matters and nothing else was checking it.

## Related
- [[REQ-010-production-deployment]] — the operational procedure
- [[ADR-014-cookie-session-authentication]] — the login that is now the only thing between the internet and the data

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-17 | Initial — accepted on first deployment to GCP |
