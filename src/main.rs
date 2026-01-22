mod catalog;
mod cli;
mod error;
mod export;
mod git;
mod json;
mod paths;
mod sources;
mod state;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    let result = run(cli);

    match result {
        Ok(value) => {
            println!("{}", json::ok(value));
        }
        Err(err) => {
            println!("{}", json::err(&err));
            std::process::exit(1);
        }
    }
}

fn run(cli: cli::Cli) -> Result<serde_json::Value, error::LoadoutError> {
    use cli::{Command, SourceCommand};

    match cli.command {
        Command::Doctor => doctor(),
        Command::Init {
            primary_url,
            primary_ref,
            trust_primary,
            writable_primary,
        } => init(primary_url, primary_ref, trust_primary, writable_primary),
        Command::Catalog { target } => catalog_cmd(target),
        Command::Suggest {
            target,
            query,
            limit,
        } => suggest_cmd(target, query, limit),
        Command::Add { target, ids } => add_cmd(target, ids),
        Command::Set { target, ids } => set_cmd(target, ids),
        Command::Remove { target, ids } => remove_cmd(target, ids),
        Command::Sync { target } => sync_cmd(target),
        Command::Status { target } => status_cmd(target),
        Command::Source { command } => match command {
            SourceCommand::Add {
                name,
                url,
                ref_name,
            } => source_add_cmd(name, url, ref_name),
            SourceCommand::Trust { name, yes } => source_trust_cmd(name, yes),
            SourceCommand::Pin { name, to } => source_pin_cmd(name, to),
            SourceCommand::Status { name } => source_status_cmd(name),
        },
    }
}

fn doctor() -> Result<serde_json::Value, error::LoadoutError> {
    let git_version = std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let repo_root = match paths::Paths::discover() {
        Ok(p) => Some(p.repo_root.to_string_lossy().to_string()),
        Err(_) => None,
    };

    Ok(serde_json::json!({
        "git_version": git_version,
        "repo_root": repo_root,
    }))
}

fn init(
    primary_url: String,
    primary_ref: String,
    trust_primary: bool,
    writable_primary: bool,
) -> Result<serde_json::Value, error::LoadoutError> {
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    state::ensure_runtime_dirs(&paths)?;

    let mut manifest = state::Manifest {
        schema_version: 1,
        primary_source: "primary".to_string(),
        sources: std::collections::BTreeMap::new(),
        targets: std::collections::BTreeMap::new(),
    };

    manifest.sources.insert(
        "primary".to_string(),
        state::SourceSpec {
            url: primary_url,
            ref_name: primary_ref,
        },
    );

    state::save_manifest(&paths, &manifest)?;

    // Pin primary to current HEAD of ref.
    let primary_spec = manifest.sources.get("primary").expect("just inserted");
    let primary_clone = sources::ensure_clone(&paths, "primary", primary_spec)?;
    sources::checkout_ref(&primary_clone, &primary_spec.ref_name)?;
    let pinned_sha = sources::head_sha(&primary_clone)?;

    let mut lock = state::Lock {
        schema_version: 1,
        sources: std::collections::BTreeMap::new(),
    };
    lock.sources.insert(
        "primary".to_string(),
        state::LockedSource {
            pinned_sha: pinned_sha.clone(),
        },
    );
    state::save_lock(&paths, &lock)?;

    let mut trust = state::load_trust(&paths)?;
    if trust_primary {
        trust.trusted_sources.insert("primary".to_string());
    }
    if writable_primary {
        trust.writable_sources.insert("primary".to_string());
    }
    state::save_trust(&paths, &trust)?;

    Ok(serde_json::json!({
        "manifest": paths.manifest,
        "lock": paths.lock,
        "primary": {
            "clone_dir": primary_clone,
            "pinned_sha": pinned_sha,
            "trusted": trust.trusted_sources.contains("primary"),
            "writable": trust.writable_sources.contains("primary"),
        }
    }))
}

fn catalog_cmd(target: cli::Target) -> Result<serde_json::Value, error::LoadoutError> {
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    let manifest = state::load_manifest(&paths)?;
    let lock = state::load_lock(&paths)?;

    let skills = collect_catalog_items(&paths, &manifest, &lock, target)?;

    Ok(serde_json::json!({
        "target": state::target_key(target),
        "primary_source": manifest.primary_source,
        "sources": manifest.sources,
        "lock": lock.sources,
        "skills": skills,
    }))
}

fn suggest_cmd(
    target: cli::Target,
    query: String,
    limit: usize,
) -> Result<serde_json::Value, error::LoadoutError> {
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    let manifest = state::load_manifest(&paths)?;
    let lock = state::load_lock(&paths)?;

    let skills = collect_catalog_items(&paths, &manifest, &lock, target)?;

    let mut suggestions = catalog::score_suggestions(&skills, &query);
    suggestions.truncate(limit);

    Ok(serde_json::json!({
        "target": state::target_key(target),
        "query": query,
        "suggestions": suggestions,
    }))
}

fn collect_catalog_items(
    paths: &paths::Paths,
    manifest: &state::Manifest,
    lock: &state::Lock,
    target: cli::Target,
) -> Result<Vec<catalog::CatalogItem>, error::LoadoutError> {
    let mut skills = Vec::new();
    for source_name in manifest.sources.keys() {
        let (clone_dir, _) = sources::ensure_checked_out(paths, manifest, lock, source_name)?;
        let catalog_file = catalog::read_catalog(&clone_dir)?;
        skills.extend(catalog::catalog_for_target(
            source_name,
            &catalog_file,
            target,
        ));
    }
    Ok(skills)
}

fn add_cmd(
    target: cli::Target,
    ids: Vec<String>,
) -> Result<serde_json::Value, error::LoadoutError> {
    mutate_selection(target, ids, Mutation::Add)
}

fn set_cmd(
    target: cli::Target,
    ids: Vec<String>,
) -> Result<serde_json::Value, error::LoadoutError> {
    mutate_selection(target, ids, Mutation::Set)
}

fn remove_cmd(
    target: cli::Target,
    ids: Vec<String>,
) -> Result<serde_json::Value, error::LoadoutError> {
    mutate_selection(target, ids, Mutation::Remove)
}

fn sync_cmd(target: cli::Target) -> Result<serde_json::Value, error::LoadoutError> {
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    let manifest = state::load_manifest(&paths)?;
    let lock = state::load_lock(&paths)?;
    let trust = state::load_trust(&paths)?;

    sync_target(&paths, &manifest, &lock, &trust, target)
}

fn status_cmd(target: cli::Target) -> Result<serde_json::Value, error::LoadoutError> {
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    let manifest = state::load_manifest(&paths)?;
    let lock = state::load_lock(&paths)?;
    let trust = state::load_trust(&paths)?;

    let selected = manifest.target(target).skills;
    Ok(serde_json::json!({
        "repo_root": paths.repo_root,
        "target": state::target_key(target),
        "selected": selected,
        "sources": manifest.sources,
        "lock": lock.sources,
        "trust": trust,
    }))
}

fn source_add_cmd(
    name: String,
    url: String,
    ref_name: String,
) -> Result<serde_json::Value, error::LoadoutError> {
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    state::ensure_runtime_dirs(&paths)?;

    let mut manifest = state::load_manifest(&paths)?;
    if manifest.sources.contains_key(&name) {
        // idempotent
        return Ok(
            serde_json::json!({ "added": false, "reason": "already_exists", "source": name }),
        );
    }
    manifest
        .sources
        .insert(name.clone(), state::SourceSpec { url, ref_name });
    state::save_manifest(&paths, &manifest)?;

    // Pin new source to HEAD.
    let source_spec = manifest.sources.get(&name).expect("just inserted");
    let clone_dir = sources::ensure_clone(&paths, &name, source_spec)?;
    sources::checkout_ref(&clone_dir, &source_spec.ref_name)?;
    let pinned_sha = sources::head_sha(&clone_dir)?;

    let mut lock = match state::load_lock(&paths) {
        Ok(lock) => lock,
        Err(error::LoadoutError::LockMissing(_)) => state::Lock {
            schema_version: 1,
            sources: std::collections::BTreeMap::new(),
        },
        Err(other) => return Err(other),
    };
    lock.sources.insert(
        name.clone(),
        state::LockedSource {
            pinned_sha: pinned_sha.clone(),
        },
    );
    state::save_lock(&paths, &lock)?;

    Ok(serde_json::json!({
        "added": true,
        "source": name,
        "clone_dir": clone_dir,
        "pinned_sha": pinned_sha,
    }))
}

fn source_trust_cmd(name: String, yes: bool) -> Result<serde_json::Value, error::LoadoutError> {
    if !yes {
        return Err(error::LoadoutError::MissingRequiredFlag(
            "--yes".to_string(),
        ));
    }
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    let manifest = state::load_manifest(&paths)?;
    if !manifest.sources.contains_key(&name) {
        return Err(error::LoadoutError::UnknownSource(name));
    }

    let mut trust = state::load_trust(&paths)?;
    trust.trusted_sources.insert(name.clone());
    state::save_trust(&paths, &trust)?;
    Ok(serde_json::json!({ "trusted": true, "source": name }))
}

fn source_pin_cmd(name: String, to: String) -> Result<serde_json::Value, error::LoadoutError> {
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    let manifest = state::load_manifest(&paths)?;
    let mut lock = state::load_lock(&paths)?;

    if !manifest.sources.contains_key(&name) {
        return Err(error::LoadoutError::UnknownSource(name));
    }

    let source_spec = manifest.sources.get(&name).expect("checked");
    let clone_dir = sources::ensure_clone(&paths, &name, source_spec)?;
    if sources::is_dirty(&clone_dir)? {
        return Err(error::LoadoutError::SourceDirty(name));
    }
    let new_sha = if to == "HEAD" {
        sources::head_sha(&clone_dir)?
    } else {
        if !state::validate_sha(&to) {
            return Err(error::LoadoutError::InvalidArgument(format!(
                "invalid sha: {to}"
            )));
        }
        // ensure sha exists
        sources::ensure_commit_exists(&clone_dir, &to)?;
        to
    };

    lock.sources.insert(
        name.clone(),
        state::LockedSource {
            pinned_sha: new_sha.clone(),
        },
    );
    state::save_lock(&paths, &lock)?;

    Ok(serde_json::json!({ "source": name, "pinned_sha": new_sha }))
}

fn source_status_cmd(name: String) -> Result<serde_json::Value, error::LoadoutError> {
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    let manifest = state::load_manifest(&paths)?;
    let lock = state::load_lock(&paths)?;
    let trust = state::load_trust(&paths)?;

    let source_spec = manifest
        .sources
        .get(&name)
        .ok_or_else(|| error::LoadoutError::UnknownSource(name.clone()))?;
    let locked = lock.sources.get(&name).map(|l| l.pinned_sha.clone());
    let clone_dir = paths.clone_dir(&name);
    Ok(serde_json::json!({
        "source": name,
        "url": source_spec.url,
        "ref": source_spec.ref_name,
        "pinned_sha": locked,
        "clone_dir": clone_dir,
        "trusted": trust.trusted_sources.contains(&name),
        "writable": trust.writable_sources.contains(&name),
    }))
}

#[derive(Clone, Copy)]
enum Mutation {
    Add,
    Set,
    Remove,
}

fn mutate_selection(
    target: cli::Target,
    ids: Vec<String>,
    mutation: Mutation,
) -> Result<serde_json::Value, error::LoadoutError> {
    let paths = paths::Paths::discover().map_err(|_| error::LoadoutError::NotGitWorktree)?;
    let mut manifest = state::load_manifest(&paths)?;
    let lock = state::load_lock(&paths)?;
    let trust = state::load_trust(&paths)?;

    let before = manifest.target(target).skills;
    let mut desired = before.clone();

    // Resolve ids/aliases via catalogs so manifest stores canonical ids.
    let mut resolved: Vec<String> = Vec::new();
    for raw in ids {
        let (source_name, query) = parse_skill_ref(&manifest, &raw)?;
        if !trust.trusted_sources.contains(&source_name) {
            return Err(error::LoadoutError::SourceUntrusted(source_name));
        }
        let (clone_dir, _) = sources::ensure_checked_out(&paths, &manifest, &lock, &source_name)?;
        let catalog_file = catalog::read_catalog(&clone_dir)?;
        let skill = catalog::resolve_skill_id(&catalog_file, &query)?;

        let canonical = if source_name == manifest.primary_source {
            skill.id.clone()
        } else {
            format!("{source_name}:{}", skill.id)
        };
        resolved.push(canonical);
    }

    match mutation {
        Mutation::Add => {
            desired.extend(resolved);
        }
        Mutation::Set => {
            desired = resolved;
        }
        Mutation::Remove => {
            let remove_set: std::collections::HashSet<String> = resolved.into_iter().collect();
            desired.retain(|s| !remove_set.contains(s));
        }
    }

    state::normalize_skill_list(&mut desired);
    *manifest.ensure_target_mut(target) = state::TargetSpec {
        skills: desired.clone(),
    };
    state::save_manifest(&paths, &manifest)?;

    let sync_result = sync_target(&paths, &manifest, &lock, &trust, target)?;

    Ok(serde_json::json!({
        "target": state::target_key(target),
        "before": before,
        "after": desired,
        "sync": sync_result,
    }))
}

fn parse_skill_ref(
    manifest: &state::Manifest,
    raw: &str,
) -> Result<(String, String), error::LoadoutError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(error::LoadoutError::InvalidSkillRef(raw.to_string()));
    }

    if let Some((left, right)) = raw.split_once(':') {
        let source = left.trim();
        let query = right.trim();
        if source.is_empty() || query.is_empty() {
            return Err(error::LoadoutError::InvalidSkillRef(raw.to_string()));
        }
        if !manifest.sources.contains_key(source) {
            return Err(error::LoadoutError::UnknownSource(source.to_string()));
        }
        Ok((source.to_string(), query.to_string()))
    } else {
        if !manifest.sources.contains_key(&manifest.primary_source) {
            return Err(error::LoadoutError::UnknownSource(
                manifest.primary_source.clone(),
            ));
        }
        Ok((manifest.primary_source.clone(), raw.to_string()))
    }
}

fn validate_rel_catalog_path(rel_path: &str) -> Result<(), error::LoadoutError> {
    use std::path::{Component, Path};
    let rel_path = rel_path.trim();
    if rel_path.is_empty() {
        return Err(error::LoadoutError::InvalidCatalogPath(
            rel_path.to_string(),
        ));
    }

    for c in Path::new(rel_path).components() {
        match c {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(error::LoadoutError::InvalidCatalogPath(
                    rel_path.to_string(),
                ))
            }
            _ => {}
        }
    }
    Ok(())
}

fn sync_target(
    paths: &paths::Paths,
    manifest: &state::Manifest,
    lock: &state::Lock,
    trust: &state::Trust,
    target: cli::Target,
) -> Result<serde_json::Value, error::LoadoutError> {
    let selected = manifest.target(target).skills;

    // Enforce trust for sources used by selection.
    let mut needed_sources: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for raw in &selected {
        let (source_name, _) = parse_skill_ref(manifest, raw)?;
        needed_sources.insert(source_name);
    }
    for source in &needed_sources {
        if !trust.trusted_sources.contains(source) {
            return Err(error::LoadoutError::SourceUntrusted(source.clone()));
        }
    }

    // Ensure clones at pinned sha.
    for source in &needed_sources {
        let _ = sources::ensure_checked_out(paths, manifest, lock, source)?;
    }

    let export_root = paths.export_root(target).to_path_buf();
    std::fs::create_dir_all(&export_root).map_err(|e| error::LoadoutError::Io {
        path: export_root.clone(),
        message: e.to_string(),
    })?;

    let mut desired_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut applied: Vec<serde_json::Value> = Vec::new();

    for raw in &selected {
        let (source_name, query) = parse_skill_ref(manifest, raw)?;
        let (clone_dir, _) = sources::ensure_checked_out(paths, manifest, lock, &source_name)?;
        let catalog_file = catalog::read_catalog(&clone_dir)?;
        let skill = catalog::resolve_skill_id(&catalog_file, &query)?;

        let qualified_id = if source_name == manifest.primary_source {
            skill.id.clone()
        } else {
            format!("{source_name}:{}", skill.id)
        };

        let Some(rel_path) = catalog::skill_target_path(skill, target) else {
            return Err(error::LoadoutError::SkillUnsupportedTarget {
                qualified_id,
                target: state::target_key(target),
            });
        };

        validate_rel_catalog_path(rel_path)?;
        let target_dir = clone_dir.join(rel_path);
        if !target_dir.exists() || !target_dir.is_dir() || !target_dir.join("SKILL.md").exists() {
            return Err(error::LoadoutError::SkillPathMissing(target_dir));
        }

        let link_name = export::export_name(&source_name, &manifest.primary_source, &skill.id);
        desired_names.insert(link_name.clone());
        let link_path = export_root.join(&link_name);
        export::ensure_dir_symlink(&link_path, &target_dir)?;

        applied.push(serde_json::json!({
            "skill": qualified_id,
            "link": link_path,
            "target_dir": target_dir,
        }));
    }

    // Remove stale managed exports.
    let existing = export::list_managed_exports(&export_root)?;
    let mut removed = Vec::new();
    for path in existing {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if !desired_names.contains(&name) {
            export::remove_managed_export(&path)?;
            removed.push(path);
        }
    }

    Ok(serde_json::json!({
        "export_root": export_root,
        "applied": applied,
        "removed": removed,
    }))
}
