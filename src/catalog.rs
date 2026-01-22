use crate::cli::Target;
use crate::error::LoadoutError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatalogFile {
    pub schema_version: u32,
    pub skills: Vec<SkillEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub targets: BTreeMap<String, TargetEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetEntry {
    pub path: String,
}

pub fn read_catalog(source_dir: &Path) -> Result<CatalogFile, LoadoutError> {
    let path = source_dir.join("catalog").join("skills.json");
    let data = fs::read_to_string(&path).map_err(|_| LoadoutError::CatalogMissing(path.clone()))?;
    serde_json::from_str(&data).map_err(|e| LoadoutError::JsonInvalid {
        path,
        message: e.to_string(),
    })
}

#[derive(Debug, Serialize, Clone)]
pub struct CatalogItem {
    pub source: String,
    pub qualified_id: String,
    pub id: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub target_path: String,
}

pub fn catalog_for_target(
    source_name: &str,
    catalog: &CatalogFile,
    target: Target,
) -> Vec<CatalogItem> {
    let target_key = match target {
        Target::Codex => "codex",
        Target::Claude => "claude",
    };

    catalog
        .skills
        .iter()
        .filter_map(|skill| {
            let target_entry = skill.targets.get(target_key)?;
            Some(CatalogItem {
                source: source_name.to_string(),
                qualified_id: format!("{source_name}:{}", skill.id),
                id: skill.id.clone(),
                title: skill.title.clone(),
                description: skill.description.clone(),
                tags: skill.tags.clone(),
                aliases: skill.aliases.clone(),
                target_path: target_entry.path.clone(),
            })
        })
        .collect()
}

pub fn resolve_skill_id<'a>(
    catalog: &'a CatalogFile,
    query: &str,
) -> Result<&'a SkillEntry, LoadoutError> {
    let q = query.trim();
    if q.is_empty() {
        return Err(LoadoutError::SkillNotFound(query.to_string()));
    }

    let mut matches: Vec<&SkillEntry> = catalog
        .skills
        .iter()
        .filter(|skill| {
            skill.id.eq_ignore_ascii_case(q)
                || skill.aliases.iter().any(|a| a.eq_ignore_ascii_case(q))
        })
        .collect();

    if matches.is_empty() {
        return Err(LoadoutError::SkillNotFound(query.to_string()));
    }
    if matches.len() > 1 {
        matches.sort_by(|a, b| a.id.cmp(&b.id));
        return Err(LoadoutError::SkillAmbiguous {
            query: query.to_string(),
            matches: matches.into_iter().map(|s| s.id.clone()).collect(),
        });
    }

    Ok(matches.pop().expect("len==1"))
}

#[derive(Debug, Serialize, Clone)]
pub struct Suggestion {
    pub source: String,
    pub qualified_id: String,
    pub id: String,
    pub score: i32,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
}

pub fn score_suggestions(items: &[CatalogItem], query: &str) -> Vec<Suggestion> {
    let q = query.trim();
    let q_lower = q.to_ascii_lowercase();
    let tokens: Vec<String> = q
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect();

    let mut out: Vec<Suggestion> = items
        .iter()
        .map(|item| {
            let mut score = 0;
            let id_lower = item.id.to_ascii_lowercase();

            if id_lower == q_lower {
                score += 100;
            } else if !q_lower.is_empty() && id_lower.starts_with(&q_lower) {
                score += 40;
            }

            let aliases_lower: Vec<String> = item
                .aliases
                .iter()
                .map(|a| a.to_ascii_lowercase())
                .collect();
            if aliases_lower.iter().any(|a| a == &q_lower) {
                score += 100;
            } else if !q_lower.is_empty() && aliases_lower.iter().any(|a| a.starts_with(&q_lower)) {
                score += 40;
            }

            for token in &tokens {
                if item.tags.iter().any(|t| t.to_ascii_lowercase() == *token) {
                    score += 20;
                }
                if item.title.to_ascii_lowercase().contains(token) {
                    score += 10;
                }
                if item.description.to_ascii_lowercase().contains(token) {
                    score += 5;
                }
            }

            Suggestion {
                source: item.source.clone(),
                qualified_id: item.qualified_id.clone(),
                id: item.id.clone(),
                score,
                title: item.title.clone(),
                description: item.description.clone(),
                tags: item.tags.clone(),
            }
        })
        .collect();

    out.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.qualified_id.cmp(&b.qualified_id))
    });

    out
}

pub fn skill_target_path(skill: &SkillEntry, target: Target) -> Option<&str> {
    let key = match target {
        Target::Codex => "codex",
        Target::Claude => "claude",
    };
    skill.targets.get(key).map(|t| t.path.as_str())
}
