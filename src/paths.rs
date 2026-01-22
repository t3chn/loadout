use crate::error::LoadoutError;
use crate::git;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Paths {
    pub repo_root: PathBuf,

    pub manifest: PathBuf,
    pub lock: PathBuf,

    pub sources_dir: PathBuf,
    pub trust: PathBuf,

    pub export_codex: PathBuf,
    pub export_claude: PathBuf,
}

impl Paths {
    pub fn discover() -> Result<Self, LoadoutError> {
        let repo_root = PathBuf::from(git::git(&["rev-parse", "--show-toplevel"], None)?);

        Ok(Self::from_repo_root(repo_root))
    }

    pub fn from_repo_root(repo_root: PathBuf) -> Self {
        let codex_dir = repo_root.join(".codex");
        let claude_dir = repo_root.join(".claude");

        let manifest = codex_dir.join("loadout.json");
        let lock = codex_dir.join("loadout.lock.json");

        let sources_dir = codex_dir.join(".loadout").join("sources");
        let trust = codex_dir.join(".loadout").join("trust.json");

        let export_codex = codex_dir.join("skills");
        let export_claude = claude_dir.join("skills");

        Self {
            repo_root,
            manifest,
            lock,
            sources_dir,
            trust,
            export_codex,
            export_claude,
        }
    }

    pub fn clone_dir(&self, source: &str) -> PathBuf {
        self.sources_dir.join(source)
    }

    pub fn export_root(&self, target: crate::cli::Target) -> &Path {
        match target {
            crate::cli::Target::Codex => &self.export_codex,
            crate::cli::Target::Claude => &self.export_claude,
        }
    }
}
