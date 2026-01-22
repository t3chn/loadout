use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let repo_root = git_stdout(&["rev-parse", "--show-toplevel"], None)
        .map(PathBuf::from)
        .map_err(|e| format!("no_cyrillic: {e}"))?;

    let files = std::env::args().skip(1).collect::<Vec<_>>();
    let files = if files.is_empty() {
        git_ls_files(&repo_root)?
    } else {
        files
    };

    let mut violations = Vec::new();

    for rel in files {
        if contains_cyrillic(&rel) {
            violations.push(Violation::path(rel.clone()));
        }

        let path = if Path::new(&rel).is_absolute() {
            PathBuf::from(&rel)
        } else {
            repo_root.join(&rel)
        };

        if path.is_dir() {
            continue;
        }

        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        if bytes.contains(&0) {
            continue;
        }
        let Ok(text) = std::str::from_utf8(&bytes) else {
            continue;
        };

        if let Some(v) = first_cyrillic_in_text(&rel, text) {
            violations.push(v);
        }
    }

    violations.sort_by_key(|a| a.display_key());
    violations.dedup_by(|a, b| a.display_key() == b.display_key());

    if violations.is_empty() {
        return Ok(());
    }

    eprintln!("Cyrillic characters are not allowed in this repository.");
    for v in &violations {
        eprintln!("{}", v.format());
    }
    Err(format!("found {} violation(s)", violations.len()))
}

fn git_ls_files(repo_root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|e| format!("failed to run git ls-files: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let parts = output.stdout.split(|b| *b == 0);
    Ok(parts
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8_lossy(p).to_string())
        .collect())
}

fn git_stdout(args: &[&str], cwd: Option<&Path>) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[derive(Clone, Debug)]
struct Violation {
    path: String,
    line: Option<usize>,
    col: Option<usize>,
    ch: Option<char>,
}

impl Violation {
    fn path(path: String) -> Self {
        Self {
            path,
            line: None,
            col: None,
            ch: None,
        }
    }

    fn format(&self) -> String {
        match (self.line, self.col, self.ch) {
            (Some(line), Some(col), Some(ch)) => format!(
                "{}:{}:{}: Cyrillic character U+{:04X} '{}'",
                self.path, line, col, ch as u32, ch
            ),
            _ => format!("{}: Cyrillic character in path", self.path),
        }
    }

    fn display_key(&self) -> String {
        match (self.line, self.col) {
            (Some(line), Some(col)) => format!("{}:{line}:{col}", self.path),
            _ => format!("{}:path", self.path),
        }
    }
}

fn first_cyrillic_in_text(path: &str, text: &str) -> Option<Violation> {
    let mut line = 1usize;
    let mut col = 1usize;

    for ch in text.chars() {
        if is_cyrillic(ch) {
            return Some(Violation {
                path: path.to_string(),
                line: Some(line),
                col: Some(col),
                ch: Some(ch),
            });
        }

        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    None
}

fn contains_cyrillic(s: &str) -> bool {
    s.chars().any(is_cyrillic)
}

fn is_cyrillic(ch: char) -> bool {
    matches!(
        ch as u32,
        0x0400..=0x04FF
            | 0x0500..=0x052F
            | 0x1C80..=0x1C8F
            | 0x2DE0..=0x2DFF
            | 0xA640..=0xA69F
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_cyrillic_detects_basic_range() {
        let capital_a = char::from_u32(0x0410).expect("U+0410");
        let small_ya = char::from_u32(0x044F).expect("U+044F");
        assert!(is_cyrillic(capital_a));
        assert!(is_cyrillic(small_ya));
        assert!(!is_cyrillic('A'));
        assert!(!is_cyrillic('z'));
    }

    #[test]
    fn first_cyrillic_reports_position() {
        let cyr_l = char::from_u32(0x043B).expect("U+043B");
        let text = format!("hello\nwor{}d", cyr_l);
        let v = first_cyrillic_in_text("file.txt", &text).expect("violation");
        assert_eq!(v.path, "file.txt");
        assert_eq!(v.line, Some(2));
        assert_eq!(v.col, Some(4));
        assert_eq!(v.ch, Some(cyr_l));
    }
}
