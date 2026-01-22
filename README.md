# Loadout

Machine-first skill manager for project-scoped skills across Codex CLI and Claude Code.

## Problem

Projects frequently copy skills from a central skills repository. That creates drift:

- the project copy diverges from the source
- improvements get lost or re-done
- each client (Codex vs Claude) ends up with a different layout and UX

## What this repo does

Loadout makes skill usage reproducible and agent-friendly:

- Uses a **manifest + lock** (pinned commit SHAs) committed in the project.
- Clones skill sources into a **gitignored runtime cache** per project.
- Exports selected skills into `.codex/skills/` and `.claude/skills/` via **symlinks**.
- Enforces **explicit trust** for third-party sources (supply-chain gating).
- Outputs **JSON by default** (including errors), so agents can parse reliably.

## Project layout

- `src/` — `loadout` CLI (Rust)
- `src/bin/no_cyrillic.rs` — repo policy check: English-only (no Cyrillic)
- `docs/` — PRD + agent playbook
- `templates/user-scoped/` — user-scoped wrapper skill templates

## Quick start (local dev)

```bash
cargo test
uvx prek run --all-files
```

## Notes

- v1 targets macOS/Linux. Windows fallback export is a roadmap item.

## License

MIT
