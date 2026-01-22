use crate::cli::Target;
use crate::error::LoadoutError;
use crate::paths::Paths;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub schema_version: u32,
    pub primary_source: String,
    pub sources: BTreeMap<String, SourceSpec>,
    pub targets: BTreeMap<String, TargetSpec>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceSpec {
    pub url: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TargetSpec {
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lock {
    pub schema_version: u32,
    pub sources: BTreeMap<String, LockedSource>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedSource {
    pub pinned_sha: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Trust {
    pub schema_version: u32,
    #[serde(default)]
    pub trusted_sources: BTreeSet<String>,
    #[serde(default)]
    pub writable_sources: BTreeSet<String>,
}

impl Manifest {
    pub fn ensure_target_mut(&mut self, target: Target) -> &mut TargetSpec {
        let key = target_key(target);
        self.targets.entry(key).or_default()
    }

    pub fn target(&self, target: Target) -> TargetSpec {
        self.targets
            .get(&target_key(target))
            .cloned()
            .unwrap_or_default()
    }
}

pub fn target_key(target: Target) -> String {
    match target {
        Target::Codex => "codex".to_string(),
        Target::Claude => "claude".to_string(),
    }
}

pub fn load_manifest(paths: &Paths) -> Result<Manifest, LoadoutError> {
    read_json(&paths.manifest).map_err(|err| match err {
        LoadoutError::Io { .. } => LoadoutError::ManifestMissing(paths.manifest.clone()),
        other => other,
    })
}

pub fn save_manifest(paths: &Paths, manifest: &Manifest) -> Result<(), LoadoutError> {
    write_json_pretty(&paths.manifest, manifest)
}

pub fn load_lock(paths: &Paths) -> Result<Lock, LoadoutError> {
    read_json(&paths.lock).map_err(|err| match err {
        LoadoutError::Io { .. } => LoadoutError::LockMissing(paths.lock.clone()),
        other => other,
    })
}

pub fn save_lock(paths: &Paths, lock: &Lock) -> Result<(), LoadoutError> {
    write_json_pretty(&paths.lock, lock)
}

pub fn load_trust(paths: &Paths) -> Result<Trust, LoadoutError> {
    // Trust is local runtime state; missing file is treated as empty trust.
    match read_json::<Trust>(&paths.trust) {
        Ok(mut trust) => {
            if trust.schema_version == 0 {
                trust.schema_version = 1;
            }
            Ok(trust)
        }
        Err(LoadoutError::Io { .. }) => Ok(Trust {
            schema_version: 1,
            ..Trust::default()
        }),
        Err(other) => Err(other),
    }
}

pub fn save_trust(paths: &Paths, trust: &Trust) -> Result<(), LoadoutError> {
    write_json_pretty(&paths.trust, trust)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, LoadoutError> {
    let data = fs::read_to_string(path).map_err(|e| LoadoutError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    serde_json::from_str(&data).map_err(|e| LoadoutError::JsonInvalid {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<(), LoadoutError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| LoadoutError::Io {
            path: parent.to_path_buf(),
            message: e.to_string(),
        })?;
    }

    let data = serde_json::to_string_pretty(value).map_err(|e| LoadoutError::JsonInvalid {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    fs::write(path, format!("{data}\n")).map_err(|e| LoadoutError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    Ok(())
}

pub fn ensure_runtime_dirs(paths: &Paths) -> Result<(), LoadoutError> {
    fs::create_dir_all(&paths.sources_dir).map_err(|e| LoadoutError::Io {
        path: paths.sources_dir.clone(),
        message: e.to_string(),
    })?;
    Ok(())
}

pub fn normalize_skill_list(skills: &mut Vec<String>) {
    skills.sort();
    skills.dedup();
}

pub fn validate_sha(sha: &str) -> bool {
    let ok_len = sha.len() >= 7 && sha.len() <= 40;
    ok_len && sha.chars().all(|c| c.is_ascii_hexdigit())
}
