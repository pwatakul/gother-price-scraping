# Price Scraper
<!-- One-liner description -->

## Quick Start
```bash
# Clone
git clone [repo-url]

# Install dependencies

# Setup environment
cp .env.example .env

# Run
```

## Project Structure
```
price-scraper/
├── .claude/CONTEXT.md       ← Read this first (Claude context)
├── docs/
│   ├── requirements/        ← Feature requirements (versioned)
│   ├── design/
│   │   ├── architecture/    ← Draw.io diagrams + PNG exports
│   │   ├── ui/              ← Figma exports + design tokens
│   │   └── data-model.md    ← DB schema & entity relationships
│   ├── decisions/           ← Architecture Decision Records (ADRs)
│   ├── sprints/             ← Sprint planning & retrospectives
│   └── CHANGELOG.md
├── src/                     ← Source code
└── tests/
```

## Documentation
- [Context & Status](.claude/CONTEXT.md)
- [Requirements](docs/requirements/)
- [Architecture](docs/design/architecture/)
- [Data Model](docs/design/data-model.md)
- [Decisions](docs/decisions/)
- [Changelog](docs/CHANGELOG.md)

## For Contributors
1. Read `.claude/CONTEXT.md` first — it tells you where everything is
2. Read the active sprint in `docs/sprints/`
3. Check `docs/decisions/` before making architectural changes
4. When requirements change: create a new version file, don't overwrite
