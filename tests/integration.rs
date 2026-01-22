use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {:?} failed: {}\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, content).expect("write file");
}

fn run_loadout(project_dir: &Path, args: &[&str]) -> (bool, serde_json::Value) {
    let bin = env!("CARGO_BIN_EXE_loadout");
    let output = Command::new(bin)
        .current_dir(project_dir)
        .args(args)
        .output()
        .expect("run loadout");

    let ok = output.status.success();
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json output");
    assert_eq!(value["ok"].as_bool(), Some(ok));
    (ok, value)
}

fn read_link(path: &Path) -> PathBuf {
    std::fs::read_link(path).expect("read symlink")
}

#[test]
fn basic_init_add_remove_sync() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let source = root.join("source");
    std::fs::create_dir_all(&source).unwrap();
    git(&source, &["init", "-b", "main"]);
    git(&source, &["config", "user.email", "test@example.com"]);
    git(&source, &["config", "user.name", "test"]);

    write(
        &source.join("catalog/skills.json"),
        r#"{
  "schema_version": 1,
  "skills": [
    {
      "id": "hello",
      "title": "Hello",
      "description": "Test skill",
      "tags": ["test"],
      "targets": {
        "codex": { "path": "skills/hello/codex" },
        "claude": { "path": "skills/hello/claude" }
      }
    }
  ]
}
"#,
    );
    write(
        &source.join("skills/hello/codex/SKILL.md"),
        "# hello (codex)\n",
    );
    write(
        &source.join("skills/hello/claude/SKILL.md"),
        "---\nname: hello\n---\n# hello (claude)\n",
    );
    git(&source, &["add", "-A"]);
    git(&source, &["commit", "-m", "init"]);

    let project = root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    git(&project, &["init", "-b", "main"]);
    git(&project, &["config", "user.email", "test@example.com"]);
    git(&project, &["config", "user.name", "test"]);

    let (ok, _) = run_loadout(
        &project,
        &[
            "init",
            "--primary-url",
            source.to_string_lossy().as_ref(),
            "--primary-ref",
            "main",
        ],
    );
    assert!(ok);

    let (ok, _) = run_loadout(&project, &["add", "--target", "codex", "hello"]);
    assert!(ok);

    let link = project.join(".codex/skills/_loadout__hello");
    assert!(link.exists());
    let expected_target = project.join(".codex/.loadout/sources/primary/skills/hello/codex");
    assert_eq!(
        std::fs::canonicalize(read_link(&link)).expect("canon actual"),
        std::fs::canonicalize(expected_target).expect("canon expected")
    );

    let (ok, _) = run_loadout(&project, &["remove", "--target", "codex", "hello"]);
    assert!(ok);
    assert!(!link.exists());

    // Sync is idempotent.
    let (ok, _) = run_loadout(&project, &["sync", "--target", "codex"]);
    assert!(ok);
    assert!(!link.exists());
}

#[test]
fn trust_gate_for_third_party_source() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let primary = root.join("primary");
    std::fs::create_dir_all(&primary).unwrap();
    git(&primary, &["init", "-b", "main"]);
    git(&primary, &["config", "user.email", "test@example.com"]);
    git(&primary, &["config", "user.name", "test"]);

    write(
        &primary.join("catalog/skills.json"),
        r#"{
  "schema_version": 1,
  "skills": [
    {
      "id": "one",
      "title": "One",
      "description": "Primary skill",
      "tags": ["test"],
      "targets": { "codex": { "path": "skills/one/codex" } }
    }
  ]
}
"#,
    );
    write(&primary.join("skills/one/codex/SKILL.md"), "# one\n");
    git(&primary, &["add", "-A"]);
    git(&primary, &["commit", "-m", "init"]);

    let third = root.join("third");
    std::fs::create_dir_all(&third).unwrap();
    git(&third, &["init", "-b", "main"]);
    git(&third, &["config", "user.email", "test@example.com"]);
    git(&third, &["config", "user.name", "test"]);

    write(
        &third.join("catalog/skills.json"),
        r#"{
  "schema_version": 1,
  "skills": [
    {
      "id": "two",
      "title": "Two",
      "description": "Third skill",
      "tags": ["test"],
      "targets": { "codex": { "path": "skills/two/codex" } }
    }
  ]
}
"#,
    );
    write(&third.join("skills/two/codex/SKILL.md"), "# two\n");
    git(&third, &["add", "-A"]);
    git(&third, &["commit", "-m", "init"]);

    let project = root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    git(&project, &["init", "-b", "main"]);
    git(&project, &["config", "user.email", "test@example.com"]);
    git(&project, &["config", "user.name", "test"]);

    let (ok, _) = run_loadout(
        &project,
        &[
            "init",
            "--primary-url",
            primary.to_string_lossy().as_ref(),
            "--primary-ref",
            "main",
        ],
    );
    assert!(ok);

    let (ok, _) = run_loadout(
        &project,
        &[
            "source",
            "add",
            "third",
            "--url",
            third.to_string_lossy().as_ref(),
            "--ref",
            "main",
        ],
    );
    assert!(ok);

    let (ok, value) = run_loadout(&project, &["add", "--target", "codex", "third:two"]);
    assert!(!ok);
    assert_eq!(value["error"]["code"], "SOURCE_UNTRUSTED");

    let (ok, _) = run_loadout(&project, &["source", "trust", "third", "--yes"]);
    assert!(ok);

    let (ok, _) = run_loadout(&project, &["add", "--target", "codex", "third:two"]);
    assert!(ok);
    assert!(project.join(".codex/skills/_loadout__third__two").exists());
}
