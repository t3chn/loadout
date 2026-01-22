# PRD: Loadout — Project Skill Manager (Codex CLI + Claude Code)
Version: 2.0 (option A)
Date: 2026-01-22
Owner: (fill in)
Status: Draft

## 1) Problem
### 1.1 Current workflow
- There is a central skills repository ("skills-repo").
- Projects copy a subset of skills (1..N) into the project.
- Improvements are made in the project copy and then manually ported back.

### 1.2 Pain
- Duplicate copies of the same skill (central vs project) drift over time.
- No deterministic, tool-driven protocol for AI agents.
- Codex CLI and Claude Code end up with different UX and layouts.

## 2) Goals / Non-goals
### 2.1 Goals
G1. Install exactly the required skills (1..N) without copying files.
G2. Edits made "from the project" land in the same source repository.
G3. Reproducibility: project state is declared in git (manifest + lock) and can be restored via `sync`.
G4. Universality: one mechanism for Codex CLI and Claude Code (easy to add new targets later).
G5. Agent-first UX: short idempotent commands, JSON-first output, no interactive prompts.

### 2.2 Non-goals (v1)
N1. A full marketplace with publishing/reviews.
N2. Vector/LLM semantic search (v1 uses deterministic lexical ranking over a catalog).
N3. Auto-updates to "latest" without explicit pin/update action.
N4. Windows fallback export (wrappers/copy instead of symlink) — roadmap.

## 3) Solution overview (option A)
### 3.1 One source repo, multiple targets
One source repository can contain implementations for multiple clients:
- `codex` (directory with `SKILL.md` compatible with Codex)
- `claude` (directory with `SKILL.md` compatible with Claude Code)

### 3.2 Projects do not store copies; projects store references
Committed files:
- `.codex/loadout.json` — manifest (desired selection)
- `.codex/loadout.lock.json` — lock (pinned commit SHA per source)

Generated (gitignored):
- `.codex/.loadout/sources/<source>/` — local git clones (not submodules)
- `.codex/skills/_loadout__<id>` — symlinks to selected Codex skills
- `.claude/skills/_loadout__<id>` — symlinks to selected Claude skills

Local trust state (gitignored):
- `.codex/.loadout/trust.json`

Why symlinks instead of submodule+sparse-checkout:
- No reliance on experimental sparse-checkout behavior.
- Editing via a project path edits the underlying source clone.

### 3.3 Multi-source is optional but supported
Primary source is always "ours". Additional public repos can be added as secondary sources, with:
- explicit qualification (`source:id`) for non-primary sources,
- explicit trust gating.

## 4) Terms and data model
### 4.1 Target
Target is a client to export skills into:
- `codex` → `.codex/skills/`
- `claude` → `.claude/skills/`

### 4.2 Source
Source is a git repository containing `catalog/skills.json`.

Each source has:
- `name` (key in manifest)
- `url` (public git URL)
- `ref` (branch/tag to fetch)
- `pinned_sha` (in lock)
- `trusted` (local, required for install/sync)
- `writable` (local, used for push flows; v1 keeps this manual)

### 4.3 Skill ids and qualification
- In the primary source, you can refer to a skill as `id` (e.g. `pdf-processing`).
- In a non-primary source, you must use `source:id` (e.g. `third:pdf-processing`).
- If multiple sources contain the same `id`, an unqualified `id` always resolves to the primary source.

### 4.4 Manifest / lock schemas (v1)
Example `.codex/loadout.json`:
```json
{
  "schema_version": 1,
  "primary_source": "primary",
  "sources": {
    "primary": { "url": "https://github.com/acme/skills", "ref": "main" },
    "third": { "url": "https://github.com/other/skills", "ref": "main" }
  },
  "targets": {
    "codex": { "skills": ["pdf-processing", "third:reporting"] },
    "claude": { "skills": ["pdf-processing"] }
  }
}
```

Example `.codex/loadout.lock.json`:
```json
{
  "schema_version": 1,
  "sources": {
    "primary": { "pinned_sha": "0123456789abcdef0123456789abcdef01234567" },
    "third": { "pinned_sha": "89abcdef0123456789abcdef0123456789abcdef" }
  }
}
```

## 5) Catalog contract (schema v1)
Each source must provide `catalog/skills.json` (no heuristic scanning in v1).

Top-level:
- `schema_version: 1`
- `skills: []`

Skill entry (minimum):
- `id` (unique within the source)
- `title`
- `description`
- `tags[]`
- `aliases[]` (optional)
- `targets` (map):
  - `codex.path` (directory containing `SKILL.md`)
  - `claude.path` (directory containing `SKILL.md`)

## 6) CLI: `loadout` (Rust)
### 6.1 Principles
- JSON on stdout by default (including errors).
- No interactive prompts; confirmations via explicit flags (e.g. `--yes`).
- Idempotent operations.
- Target must always be explicit: `--target codex|claude`.

### 6.2 Commands (v1)
- `loadout init --primary-url <url> [--primary-ref <ref>]`
- `loadout catalog --target <t>`
- `loadout suggest --target <t> --query <q> [--limit N]`
- `loadout add --target <t> <id...>` (additive)
- `loadout set --target <t> <id...>` (replace selection)
- `loadout remove --target <t> <id...>`
- `loadout sync --target <t>` (replay manifest+lock)
- `loadout status --target <t>`
- `loadout doctor`

Source management:
- `loadout source add <name> --url <url> [--ref <ref>]`
- `loadout source trust <name> --yes`
- `loadout source pin <name> --to <sha|HEAD>`
- `loadout source status <name>`

### 6.3 User-scoped skill wrappers (agents)
For agent usage, provide user-scoped wrappers:
- Codex: `~/.codex/skills/loadout/`
- Claude: `~/.claude/skills/loadout/`

The wrapper should call `loadout` with an explicit `--target` and follow `docs/Agent_Playbook.md`.

## 7) Editing and "pushing from a project"
Project symlinks point into local clones under `.codex/.loadout/sources/<source>/`.

Implication:
- editing `./.codex/skills/_loadout__<id>/...` edits files in the source clone.
- to publish improvements: commit/push within that clone.
- then update the project lock pin via `loadout source pin <name> --to HEAD` and commit the lock/manifest changes in the project.

## 8) Supply-chain safety
- A new source is not trusted by default.
- Install/sync must fail deterministically for untrusted sources (`SOURCE_UNTRUSTED`) so the agent can request confirmation.
- Sources are added only via `loadout source add ...` or project manifest (not from arbitrary user text).

## 9) Suggest algorithm (v1, deterministic)
Scoring:
- `id` exact match (case-insensitive): +100
- `id` prefix match: +40
- token matches (split query by non-alnum):
  - in `tags`: +20 per token
  - in `title`: +10
  - in `description`: +5
- alias matches: same as `id` (exact +100, prefix +40)

Sort:
1) `score` desc
2) `qualified_id` asc (stability)

## 10) Acceptance criteria (DoD)
AC-1. In a git worktree with `.codex/loadout.json` + `.codex/loadout.lock.json`:
- `loadout sync --target codex` creates symlinks in `.codex/skills/` according to the manifest.

AC-2. `loadout add --target codex pdf-processing`:
- updates the manifest,
- creates `.codex/skills/_loadout__pdf-processing` symlink.

AC-3. `loadout suggest --target codex --query pdf`:
- returns JSON results including `qualified_id`, `id`, `source`, `score`.

AC-4. Untrusted third-party source:
- `loadout add --target codex third:some-skill` returns `SOURCE_UNTRUSTED`,
- after `loadout source trust third --yes` it succeeds.

## 11) Testing plan
- Unit: catalog parsing, manifest/lock parsing, suggest scoring.
- Integration: temp project + temp git sources:
  - init + add/set/remove + sync
  - verify symlinks point to expected paths

## 12) Roadmap (v2+)
- Windows: wrapper/copy export instead of symlinks.
- Bootstrap: install/update the `loadout` binary for agents.
- Optional: global source cache (instead of per-project clones).

---

## References
1) OpenAI Codex — Skills overview (locations & symlink support):
   https://developers.openai.com/codex/skills/

2) OpenAI Codex — Create a skill (paths for user-scoped and repo-scoped skills):
   https://developers.openai.com/codex/skills/create-skill/

3) Anthropic Claude Code — Skills:
   https://code.claude.com/docs/en/skills
