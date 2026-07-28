# Instructions for Claude Code

## Read first, always
Before implementing anything, read these files in order:
1. `.claude/CONTEXT.md` — current project status and what's done
2. `docs/sprints/` — find the active sprint file
3. The relevant `docs/requirements/REQ-XXX.md` for the feature being worked on

## How to help me implement
- Check CONTEXT.md for what's already done before writing new code
- Never re-implement something already marked as done
- If a requirement is unclear, ask before implementing
- After implementing a feature, remind me to update CONTEXT.md and the sprint file

## Code conventions
- Language: Backend — Rust (edition 2021); Frontend — TypeScript (strict mode)
- Formatting: Backend — `rustfmt` defaults; Frontend — ESLint with `@typescript-eslint`
- Testing: Backend — `cargo test` (unit tests in-module); Frontend — `npm test` (if configured)
- Error handling: Backend — `AppError` enum (thiserror) returning `AppResult<T>`; map all external errors to `AppError` variants before returning from handlers

## When requirements change
- Do NOT overwrite existing REQ files
- Create a new version: REQ-001-v1.1.md
- Note what changed and why in the Change Log table

## Commit message format
- `feat: description` — new feature
- `fix: description` — bug fix
- `req: REQ-001 v1.1` — requirement updated
- `doc: description` — docs only
- `refactor: description` — code cleanup

## Do not
- Do not delete or overwrite versioned requirement files
- Do not skip reading CONTEXT.md before starting work
- Do not make architectural decisions without creating an ADR in docs/decisions/
