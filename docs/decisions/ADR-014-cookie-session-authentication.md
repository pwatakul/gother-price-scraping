---
title: "ADR-014: httpOnly Cookie Sessions, and No Fallback Signing Key"
type: decision
date: 2026-08-17
status: Accepted
tags: [adr, authentication, security, sessions]
related: ["[[REQ-009-v1.0]]", "[[ADR-008-no-silent-mock-fallback]]"]
---

# ADR-014: httpOnly Cookie Sessions, and No Fallback Signing Key

## Context
The platform had no authentication of any kind: every endpoint and every page
was open to anyone who could reach the host. The request was for "a simple
authentication" with a seeded `admin` account and a role field ready for later.

Two properties of the existing setup narrowed the design before any preference
came into it:

- **Everything is same-origin.** `docker/nginx.conf` proxies `/api` to the
  backend and the frontend calls the relative `/api` base, so the browser only
  ever talks to one origin. A cookie therefore needs no CORS work, no
  `Authorization` header plumbing, and no preflight handling.
- **No auth crates were present.** Whatever was chosen had to be added from
  scratch, so "what's already here" carried no weight.

## Decision

### 1. Session state lives in a signed JWT inside an httpOnly cookie
Not a token in `localStorage`. The decisive difference is that injected
JavaScript — a compromised npm dependency, an XSS hole in a table cell — can
read `localStorage` and cannot read an httpOnly cookie. Given the app is
same-origin, the usual reason to prefer a header-borne token (cross-origin API
calls) does not apply here, so the security difference is the only one left.

The cookie is `SameSite=Strict`, which also means no CSRF token is needed: the
browser will not attach the cookie to a request originating from another site.

`Secure` is deliberately **off**. The app is served over plain HTTP on
localhost; a `Secure` cookie would never be sent and login would fail with no
visible cause. This is recorded as an open risk on [[REQ-009-v1.0]] and must be
turned on when the app moves to HTTPS.

### 2. The backend refuses to start without `JWT_SECRET`
`Config::from_env` returns an error when `JWT_SECRET` is missing or shorter than
32 characters. There is no default value and no generated-at-startup fallback.

A hardcoded default would be readable by anyone with the source and would let
them forge a session cookie for any user and role — the login screen would look
like security while providing none. A random secret generated per boot would be
safer but would silently log every user out on every restart, and the cause
would be invisible.

This is the same principle as [[ADR-008-no-silent-mock-fallback]]: when a
credential is missing, **fail loudly rather than substitute something that looks
like it works.** There, a missing SerpAPI key silently produced fabricated
prices that were reported as successes. Here, a missing signing key would
silently produce forgeable sessions. Both are failures that hide themselves.

### 3. The admin account is seeded at startup, not in the migration
An Argon2 hash embeds its own salt, so seeding one from SQL would commit a
single fixed hash to git permanently — and leave `ADMIN_PASSWORD` nothing to
override, since the migration would already have run. Seeding in `main.rs`
after migrations is idempotent (it checks `COUNT(*) = 0` first), so a password
changed through the UI survives every restart.

Being on the default password is **detected, not remembered**: the API checks
the stored hash against the known default and returns
`using_default_password`. No flag to set, no flag that can go stale.

### 4. The auth gate wraps the whole router, not individual routes
`route_layer` is applied to the entire authenticated router; the two public
routes (`/health`, `/auth/login`, plus `/auth/logout`) are built on a separate
un-layered router and merged. A route added next month is protected because it
was added to the protected router, not because someone remembered to protect it.
`route_layer` rather than `layer` so an unknown path still 404s instead of being
turned into a misleading 401.

### 5. Roles are stored and carried, but enforce nothing
`role` is a column with a CHECK constraint, travels in the token, and renders as
a badge. **No endpoint checks it.** Half-enforcing roles — gating a few
endpoints and not others — is worse than not enforcing them, because it creates
the impression of a boundary that isn't there. Stated explicitly in REQ-009 F-007
so the badge is not mistaken for a restriction.

## Alternatives Considered
| Option | Pros | Cons |
|--------|------|------|
| **A: JWT in httpOnly cookie (chosen)** | Unreadable by JS; no CORS or CSRF work at same origin; stateless | No revocation before expiry |
| B: JWT in `localStorage` + `Authorization` header | Works cross-origin; easy to inspect while debugging | Readable by any injected script — the exact thing worth preventing, for a benefit this app doesn't need |
| C: Server-side sessions in Redis | Instant revocation; Redis is already running | Adds a required dependency to *every* request, including the login page; revocation isn't needed for a single-user internal tool yet |
| D: HTTP Basic auth at nginx | Zero backend code | No logout, no roles, no password change, and no path to either — dead end given roles were explicitly asked for |
| E: A default `JWT_SECRET` with a warning | Nothing breaks on first run | Anyone with the source could forge sessions; warnings in a startup log are not read. Rejected on the ADR-008 precedent |

## Consequences

### Positive
- The API is closed by default; an unauthenticated request gets 401 everywhere
  except two named routes.
- Passwords are Argon2id with per-user salts; identical passwords produce
  different hashes (unit-tested).
- Username enumeration is not possible through the login endpoint — unknown user
  and wrong password return identical responses.
- Adding role enforcement later is a check inside one middleware.

### Negative / Trade-offs
- **No revocation.** A stolen cookie is valid for up to 12 hours, and changing
  the password does not invalidate outstanding tokens. The fix when this matters
  is a `token_version` integer on `users` included in the claims.
- **`JWT_SECRET` is now a hard startup requirement.** Any environment without it
  will fail to boot — deliberate, but it will stop the stack the first time
  someone deploys without reading the change.
- **Everyone is logged out on deploy**, since no sessions existed before.
- Two more crates (`argon2`, `jsonwebtoken`) and a slower login by design —
  Argon2 is deliberately expensive.
- `Secure` off and CORS still `Any` are both correct for localhost and both
  wrong for anything public.

## Verification
Confirmed against the running stack: all 26 migrations replay clean on a fresh
database; boot fails with a named message when `JWT_SECRET` is absent and again
when it is 8 characters; `/api/hotels` returns 401 without a cookie and 200 with
one; an unknown path still returns 404; a token signed with a different secret
and a tampered token are both rejected (unit tests); logout makes the same
request 401 again; restarting the backend neither re-seeds nor resets the
password; and changing the password invalidates the old one and flips
`using_default_password` to false — then back to true when the default is
restored.

## Related
- [[REQ-009-v1.0]] — the requirement this implements
- [[ADR-008-no-silent-mock-fallback]] — the same "fail loudly on a missing
  credential" principle, applied to scraping

## Change Log
| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-17 | Initial — accepted alongside the first authentication implementation |
