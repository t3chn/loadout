# Implementation Checklist: Loadout (v1)
Date: 2026-01-22
Goal: implement a deterministic project skill manager (manifest+lock + git clones + symlink export) with targets `codex` and `claude`.

## A. Skills source repo (primary)
- [ ] Adopt a single structure: "skill = directory with SKILL.md"
- [ ] Add `catalog/skills.json` (schema v1)
  - [ ] Fields: `id`, `title`, `description`, `tags[]`, `aliases[]?`, `targets.{codex,claude}.path`
  - [ ] `targets.*.path` points to the directory containing `SKILL.md`
- [ ] Define `id` rule (kebab-case), unique within a source
- [ ] (Optional) CI validation for the catalog (v2)

## B. User-scoped skill wrapper (for agents)
Goal: Codex/Claude can invoke `loadout` consistently.

### B1. Codex
- [ ] `~/.codex/skills/loadout/`
  - [ ] `SKILL.md`: suggest → choose → add/set → status (see `docs/Agent_Playbook.md`)

### B2. Claude
- [ ] `~/.claude/skills/loadout/`
  - [ ] `SKILL.md`: same protocol, but always uses `--target claude`
- [ ] Verify whether symlinking a shared `SKILL.md` is allowed (policy/platform)

## C. CLI: `loadout` (Rust)
### C1. Commands
- [ ] `init --primary-url --primary-ref?`
- [ ] `catalog --target`
- [ ] `suggest --target --query --limit?`
- [ ] `add|set|remove --target <ids...>`
- [ ] `sync --target`
- [ ] `status --target`
- [ ] `doctor`
- [ ] `source add|trust|pin|status`

### C2. Project files and layout
- [ ] Manifest: `.codex/loadout.json` (committed)
- [ ] Lock: `.codex/loadout.lock.json` (committed)
- [ ] Runtime (gitignored):
  - [ ] `.codex/.loadout/sources/<source>/` (git clone)
  - [ ] `.codex/.loadout/trust.json` (local trust)

### C3. Git operations (via subprocess `git`)
- [ ] `clone` / `fetch` / `checkout <pinned_sha>`
- [ ] `rev-parse` for SHA validation
- [ ] `status --porcelain` for dirty-checks (optional)

### C4. Target exports (symlink)
- [ ] Codex export root: `.codex/skills/`
  - [ ] Create symlink: `.codex/skills/_loadout__<id>` → `<clone>/<path>`
- [ ] Claude export root: `.claude/skills/`
  - [ ] Create symlink: `.claude/skills/_loadout__<id>` → `<clone>/<path>`
- [ ] Cleanup: delete only `_loadout__*` entries (do not touch unrelated dirs)

### C5. Trust/writable gates
- [ ] New sources default to `trusted=false`, `writable=false`
- [ ] `source trust <name> --yes` enables install/sync
- [ ] In v1, commit/push is done by the user inside `.codex/.loadout/sources/<source>/` (helper commands later)

### C6. Output and errors
- [ ] JSON by default (stdout)
- [ ] Stable `error.code` values (e.g. `NOT_GIT_WORKTREE`, `SOURCE_UNTRUSTED`, `CATALOG_INVALID`)

## D. Testing
### D1. Unit
- [ ] Parse `catalog/skills.json`
- [ ] Parse manifest/lock
- [ ] Suggest scoring

### D2. Integration (required)
- [ ] Temp project (tmp):
  - [ ] `init`
  - [ ] `add` (1 skill)
  - [ ] `add` (2nd skill)
  - [ ] `remove`
  - [ ] `set`
  - [ ] `sync` after deleting export directories
- [ ] Temp source repo (tmp git) with minimal catalog and `SKILL.md`
- [ ] Verify symlink points to expected paths

## E. Docs
- [ ] README: quickstart (init → trust → suggest → add → sync)
- [ ] Troubleshooting:
  - [ ] not in git / not a git worktree
  - [ ] missing catalog / invalid schema
  - [ ] untrusted source
  - [ ] symlink unsupported (roadmap)

## F. Rollout
- [ ] Add `.gitignore` template for `.codex/.loadout/` and exported `_loadout__*`
- [ ] Add examples of `.codex/loadout.json` and `.codex/loadout.lock.json`
