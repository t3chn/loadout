use crate::error::LoadoutError;
use std::path::Path;
use std::process::Command;

pub fn git(args: &[&str], cwd: Option<&Path>) -> Result<String, LoadoutError> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    git_dyn(&args, cwd)
}

pub fn git_dyn(args: &[String], cwd: Option<&Path>) -> Result<String, LoadoutError> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd.output().map_err(|e| LoadoutError::Io {
        path: Path::new("git").to_path_buf(),
        message: e.to_string(),
    })?;

    if !output.status.success() {
        let status = output.status.code().unwrap_or(1);
        return Err(LoadoutError::GitFailed {
            args: args.to_vec(),
            status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
