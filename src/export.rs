use crate::error::LoadoutError;
use std::fs;
use std::path::{Path, PathBuf};

pub const MANAGED_PREFIX: &str = "_loadout__";

pub fn export_name(source: &str, primary_source: &str, id: &str) -> String {
    if source == primary_source {
        format!("{MANAGED_PREFIX}{id}")
    } else {
        format!("{MANAGED_PREFIX}{source}__{id}")
    }
}

pub fn list_managed_exports(export_root: &Path) -> Result<Vec<PathBuf>, LoadoutError> {
    let mut out = Vec::new();
    if !export_root.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(export_root).map_err(|e| LoadoutError::Io {
        path: export_root.to_path_buf(),
        message: e.to_string(),
    })? {
        let entry = entry.map_err(|e| LoadoutError::Io {
            path: export_root.to_path_buf(),
            message: e.to_string(),
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(MANAGED_PREFIX) {
            out.push(entry.path());
        }
    }
    Ok(out)
}

pub fn ensure_dir_symlink(link_path: &Path, target_dir: &Path) -> Result<(), LoadoutError> {
    if link_path.exists() || fs::symlink_metadata(link_path).is_ok() {
        let meta = fs::symlink_metadata(link_path).map_err(|e| LoadoutError::Io {
            path: link_path.to_path_buf(),
            message: e.to_string(),
        })?;

        if !meta.file_type().is_symlink() {
            return Err(LoadoutError::ExportPathNotSymlink(link_path.to_path_buf()));
        }

        let current = fs::read_link(link_path).map_err(|e| LoadoutError::Io {
            path: link_path.to_path_buf(),
            message: e.to_string(),
        })?;
        if current == target_dir {
            return Ok(());
        }

        fs::remove_file(link_path).map_err(|e| LoadoutError::Io {
            path: link_path.to_path_buf(),
            message: e.to_string(),
        })?;
    }

    create_dir_symlink(target_dir, link_path)
}

pub fn remove_managed_export(path: &Path) -> Result<(), LoadoutError> {
    let meta = fs::symlink_metadata(path).map_err(|e| LoadoutError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    if !meta.file_type().is_symlink() {
        return Err(LoadoutError::ExportPathNotSymlink(path.to_path_buf()));
    }
    fs::remove_file(path).map_err(|e| LoadoutError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    Ok(())
}

#[cfg(unix)]
fn create_dir_symlink(target: &Path, link: &Path) -> Result<(), LoadoutError> {
    std::os::unix::fs::symlink(target, link).map_err(|e| LoadoutError::Io {
        path: link.to_path_buf(),
        message: e.to_string(),
    })
}

#[cfg(windows)]
fn create_dir_symlink(target: &Path, link: &Path) -> Result<(), LoadoutError> {
    std::os::windows::fs::symlink_dir(target, link).map_err(|e| LoadoutError::Io {
        path: link.to_path_buf(),
        message: e.to_string(),
    })
}
