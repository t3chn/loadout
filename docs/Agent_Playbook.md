# Agent Playbook: Loadout UX (Codex CLI / Claude Code)
Goal: ensure any agent behaves consistently and deterministically when managing project skills via `loadout`.

## 1) When to use
Use `loadout` when the user asks to:
- add/install skill(s) into a project
- remove/replace the project skill set
- "link skills from the central repo"
- "show what skills exist / suggest relevant ones"

## 2) Required protocol
### 2.1 Suggest-first (default)
1) Get candidates:
   - `loadout suggest --target <codex|claude> --query "<need>" --limit 10`
2) Show a short list (top-10):
   - `qualified_id` (or `id`), `title`, `tags`, 1-line `description`
3) Ask the user to pick id(s) (one or many).
4) Apply:
   - `loadout add --target <codex|claude> <id...>`
5) Report:
   - what changed,
   - current `status`.

### 2.2 Catalog (when the user wants "everything")
1) `loadout catalog --target <codex|claude>`
2) Show a list (prefer first 30 + recommend `suggest`).
3) Continue as in suggest-first.

### 2.3 Trust gate (for third-party sources)
If the user selects `source:id` and the source is not trusted:
1) Explain that explicit trust is required.
2) Ask for confirmation.
3) Run: `loadout source trust <source> --yes`
4) Retry `add/set/sync`.

## 3) Recommended user-facing format
- Show at most 10 candidates at a time.
- Format:
  1) `qualified_id — title`
  2) `tags: ...`
  3) 1-line description

## 4) Auto-pick rules (no further questions)
If the user says "pick for me":
- choose top-1 by `score`,
- or top-2 if they are complementary.

## 5) Safety
- Do not add new sources from user-provided text without explicit confirmation.
- Do not manually fix symlinks; use `loadout sync`.

## 6) After `git clone`
If the user says "I don't see the skills in the project":
- `loadout sync --target <codex|claude>`

## 7) Do not do
- Do not copy skill files manually.
- Do not create/remove symlinks directly; only use `loadout`.
