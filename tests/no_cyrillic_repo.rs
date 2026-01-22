use std::process::Command;

#[test]
fn repo_contains_no_cyrillic_characters() {
    let bin = env!("CARGO_BIN_EXE_no_cyrillic");
    let output = Command::new(bin)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run no_cyrillic");

    assert!(
        output.status.success(),
        "no_cyrillic failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
