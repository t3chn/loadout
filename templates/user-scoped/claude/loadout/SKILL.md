---
name: loadout
description: Manage project skills via `loadout` (machine-first, JSON-first).
---

# loadout (Claude wrapper)
Purpose: manage project-scoped skills via `loadout` (JSON-first, agent-first).

## Preconditions
- `loadout` binary is available in `PATH`.
- You are inside a git worktree.

## Protocol (agent-first)
1) Get candidates:
   - `loadout suggest --target claude --query "<need>" --limit 10`
2) Show the user a short list (max 10): `qualified_id`, `title`, `tags`, 1-line `description`.
3) Get the selection (id or `source:id`).
4) Apply:
   - `loadout add --target claude <id...>` (additive)
   - `loadout set --target claude <id...>` (replace selection)
   - `loadout remove --target claude <id...>` (remove)
5) If you get `SOURCE_UNTRUSTED`:
   - ask for explicit confirmation
   - `loadout source trust <source> --yes`
   - retry step 4
6) Report:
   - `loadout status --target claude`

## Rules
- Do not manually create/remove `.claude/skills/_loadout__*`; only use `loadout`.
- Do not add new sources without explicit user confirmation.
