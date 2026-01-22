use crate::error::LoadoutError;
use crate::git;
use crate::paths::Paths;
use crate::state::{Lock, Manifest, SourceSpec};
use std::path::{Path, PathBuf};

pub fn ensure_clone(
    paths: &Paths,
    source_name: &str,
    source: &SourceSpec,
) -> Result<PathBuf, LoadoutError> {
    crate::state::ensure_runtime_dirs(paths)?;
    let clone_dir = paths.clone_dir(source_name);

    if !clone_dir.exists() {
        let args = vec![
            "clone".to_string(),
            "--quiet".to_string(),
            "--".to_string(),
            source.url.clone(),
            clone_dir.to_string_lossy().to_string(),
        ];
        git::git_dyn(&args, None)?;
    }

    // Best-effort fetch: if it fails (offline), we still allow using existing clone.
    let _ = git::git(&["fetch", "--all", "--tags", "--prune"], Some(&clone_dir));
    Ok(clone_dir)
}

pub fn ensure_checked_out(
    paths: &Paths,
    manifest: &Manifest,
    lock: &Lock,
    source_name: &str,
) -> Result<(PathBuf, String), LoadoutError> {
    let source = manifest
        .sources
        .get(source_name)
        .ok_or_else(|| LoadoutError::UnknownSource(source_name.to_string()))?;
    let locked = lock
        .sources
        .get(source_name)
        .ok_or_else(|| LoadoutError::LockEntryMissing(source_name.to_string()))?;

    let clone_dir = ensure_clone(paths, source_name, source)?;

    let dirty = git::git(&["status", "--porcelain"], Some(&clone_dir))?;
    if !dirty.is_empty() {
        return Err(LoadoutError::SourceDirty(source_name.to_string()));
    }

    git::git(
        &[
            "checkout",
            "--detach",
            "--quiet",
            locked.pinned_sha.as_str(),
        ],
        Some(&clone_dir),
    )?;

    Ok((clone_dir, locked.pinned_sha.clone()))
}

pub fn checkout_ref(clone_dir: &Path, ref_name: &str) -> Result<(), LoadoutError> {
    git::git(&["checkout", "--quiet", ref_name], Some(clone_dir))?;
    Ok(())
}

pub fn head_sha(clone_dir: &Path) -> Result<String, LoadoutError> {
    git::git(&["rev-parse", "HEAD"], Some(clone_dir))
}

pub fn is_dirty(clone_dir: &Path) -> Result<bool, LoadoutError> {
    let out = git::git(&["status", "--porcelain"], Some(clone_dir))?;
    Ok(!out.trim().is_empty())
}

pub fn ensure_commit_exists(clone_dir: &Path, sha: &str) -> Result<(), LoadoutError> {
    git::git(
        &["cat-file", "-e", &format!("{sha}^{{commit}}")],
        Some(clone_dir),
    )?;
    Ok(())
}
