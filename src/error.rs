use serde::Serialize;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum LoadoutError {
    #[error("not a git worktree (run inside a git repo)")]
    NotGitWorktree,

    #[error("manifest not found: {0}")]
    ManifestMissing(PathBuf),

    #[error("lock not found: {0}")]
    LockMissing(PathBuf),

    #[error("lock entry missing for source: {0}")]
    LockEntryMissing(String),

    #[error("catalog not found: {0}")]
    CatalogMissing(PathBuf),

    #[error("invalid json in {path}: {message}")]
    JsonInvalid { path: PathBuf, message: String },

    #[error("io error: {path}: {message}")]
    Io { path: PathBuf, message: String },

    #[error("git failed: {args:?}")]
    GitFailed {
        args: Vec<String>,
        status: i32,
        stdout: String,
        stderr: String,
    },

    #[error("unknown source: {0}")]
    UnknownSource(String),

    #[error("source is not trusted: {0}")]
    SourceUntrusted(String),

    #[error("source clone is dirty: {0}")]
    SourceDirty(String),

    #[error("skill not found: {0}")]
    SkillNotFound(String),

    #[error("skill is ambiguous: {query}")]
    SkillAmbiguous { query: String, matches: Vec<String> },

    #[error("skill does not support target: {qualified_id} target={target}")]
    SkillUnsupportedTarget {
        qualified_id: String,
        target: String,
    },

    #[error("export path exists but is not a symlink: {0}")]
    ExportPathNotSymlink(PathBuf),

    #[error("invalid skill ref: {0}")]
    InvalidSkillRef(String),

    #[error("skill path missing: {0}")]
    SkillPathMissing(PathBuf),

    #[error("missing required flag: {0}")]
    MissingRequiredFlag(String),

    #[error("invalid catalog path: {0}")]
    InvalidCatalogPath(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

#[derive(Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl LoadoutError {
    pub fn to_error_body(&self) -> ErrorBody {
        let code = match self {
            LoadoutError::NotGitWorktree => "NOT_GIT_WORKTREE",
            LoadoutError::ManifestMissing(_) => "MANIFEST_MISSING",
            LoadoutError::LockMissing(_) => "LOCK_MISSING",
            LoadoutError::LockEntryMissing(_) => "LOCK_ENTRY_MISSING",
            LoadoutError::CatalogMissing(_) => "CATALOG_MISSING",
            LoadoutError::JsonInvalid { .. } => "JSON_INVALID",
            LoadoutError::Io { .. } => "IO",
            LoadoutError::GitFailed { .. } => "GIT_FAILED",
            LoadoutError::UnknownSource(_) => "UNKNOWN_SOURCE",
            LoadoutError::SourceUntrusted(_) => "SOURCE_UNTRUSTED",
            LoadoutError::SourceDirty(_) => "SOURCE_DIRTY",
            LoadoutError::SkillNotFound(_) => "SKILL_NOT_FOUND",
            LoadoutError::SkillAmbiguous { .. } => "SKILL_AMBIGUOUS",
            LoadoutError::SkillUnsupportedTarget { .. } => "SKILL_UNSUPPORTED_TARGET",
            LoadoutError::ExportPathNotSymlink(_) => "EXPORT_PATH_NOT_SYMLINK",
            LoadoutError::InvalidSkillRef(_) => "INVALID_SKILL_REF",
            LoadoutError::SkillPathMissing(_) => "SKILL_PATH_MISSING",
            LoadoutError::MissingRequiredFlag(_) => "MISSING_REQUIRED_FLAG",
            LoadoutError::InvalidCatalogPath(_) => "INVALID_CATALOG_PATH",
            LoadoutError::InvalidArgument(_) => "INVALID_ARGUMENT",
        };

        let details = match self {
            LoadoutError::GitFailed {
                args,
                status,
                stdout,
                stderr,
            } => Some(serde_json::json!({
                "args": args,
                "status": status,
                "stdout": stdout,
                "stderr": stderr,
            })),
            LoadoutError::SkillAmbiguous { matches, .. } => {
                Some(serde_json::json!({ "matches": matches }))
            }
            LoadoutError::ManifestMissing(path)
            | LoadoutError::LockMissing(path)
            | LoadoutError::CatalogMissing(path)
            | LoadoutError::ExportPathNotSymlink(path)
            | LoadoutError::SkillPathMissing(path) => Some(serde_json::json!({ "path": path })),
            LoadoutError::JsonInvalid { path, .. } | LoadoutError::Io { path, .. } => {
                Some(serde_json::json!({ "path": path }))
            }
            _ => None,
        };

        ErrorBody {
            code,
            message: self.to_string(),
            details,
        }
    }
}
