```
██╗      ██████╗  █████╗ ██████╗  ██████╗ ██╗   ██╗████████╗
██║     ██╔═══██╗██╔══██╗██╔══██╗██╔═══██╗██║   ██║╚══██╔══╝
██║     ██║   ██║███████║██║  ██║██║   ██║██║   ██║   ██║
██║     ██║   ██║██╔══██║██║  ██║██║   ██║██║   ██║   ██║
███████╗╚██████╔╝██║  ██║██████╔╝╚██████╔╝╚██████╔╝   ██║
╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝  ╚═════╝  ╚═════╝    ╚═╝
```

<div align="center">

[![Typing SVG](https://readme-typing-svg.demolab.com?font=Fira+Code&weight=500&size=20&duration=2500&pause=800&color=00FF00&center=true&vCenter=true&width=520&lines=%24+loadout+--help;manifest+%2B+lock+%2B+symlinks;codex+%7C+claude;agent-first+skill+management)](https://git.io/typing-svg)

![CI](https://img.shields.io/github/actions/workflow/status/t3chn/loadout/ci.yml?branch=main&style=flat-square&logo=github&logoColor=00ff00&label=ci&labelColor=000000&color=00ff00)
![License](https://img.shields.io/github/license/t3chn/loadout?style=flat-square&label=license&labelColor=000000&color=00ff00)
![Rust](https://img.shields.io/badge/rust-stable-000000?style=flat-square&logo=rust&logoColor=00ff00)
![prek](https://img.shields.io/badge/prek-enabled-000000?style=flat-square&logo=pre-commit&logoColor=00ff00)
![policy](https://img.shields.io/badge/policy-english--only-000000?style=flat-square&labelColor=000000&color=00ff00)

</div>

---

<details open>
<summary><b>📌 ~/problem</b></summary>
<br>

Projects often copy skills from a central skills repository. That creates drift:

- the project copy diverges from the source
- improvements get lost or duplicated
- each client (Codex vs Claude) ends up with a different layout and UX

</details>

<details open>
<summary><b>🧠 ~/solution</b></summary>
<br>

Loadout makes skill usage reproducible and agent-friendly:

- **manifest + lock** (pinned commit SHAs) committed in the project
- per-project source clones in `.codex/.loadout/` (gitignored)
- symlink export into `.codex/skills/` and `.claude/skills/`
- explicit trust gate for third-party sources
- JSON by default on stdout (including errors)

Docs:

- `docs/PRD_Skill_Manager.md`
- `docs/Agent_Playbook.md`

</details>

<details>
<summary><b>🚀 ~/quickstart</b></summary>
<br>

```bash
# in a git project
loadout init --primary-url <skills_repo_url> --primary-ref main

loadout suggest --target codex --query "pdf" --limit 10
loadout add --target codex pdf-processing

# if you select third-party skills, you must trust the source explicitly
loadout source trust third --yes
```

</details>

<details>
<summary><b>🧪 ~/dev</b></summary>
<br>

```bash
cargo test
uvx prek run --all-files
```

</details>

---

<details>
<summary><b>🗒️ ~/notes</b></summary>
<br>

- v1 targets macOS/Linux (symlink export). Windows fallback export is a roadmap item.

</details>

## License

MIT
