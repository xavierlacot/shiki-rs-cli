use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_shiki-rs-cli"))
        .args(args)
        .output()
        .expect("the binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("utf-8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("utf-8 stderr")
}

#[test]
fn single_file_matches_shiki_output() {
    let output = cli(&[fixture("sample.ts").to_str().unwrap(), "--lang=ts"]);
    assert!(output.status.success());
    let expected = std::fs::read_to_string(fixture("expected/sample.ts.monokai.html")).unwrap();
    assert_eq!(stdout(&output), expected);
}

#[test]
fn json_mode_matches_shiki_output() {
    let output = cli(&[fixture("input.json").to_str().unwrap(), "--json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let expected = std::fs::read_to_string(fixture("expected/input.json")).unwrap();
    assert_eq!(stdout(&output), expected);
}

#[test]
fn theme_option_is_used() {
    let output = cli(&[
        fixture("sample.ts").to_str().unwrap(),
        "--lang=ts",
        "--theme=github-dark",
    ]);
    assert!(output.status.success());
    assert!(stdout(&output).starts_with(r#"<pre class="shiki github-dark""#));
}

#[test]
fn missing_file_argument_fails_with_usage() {
    let output = cli(&[]);
    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("Usage:"));
}

#[test]
fn unreadable_file_fails() {
    let output = cli(&[fixture("missing.ts").to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("cannot read"));
}

#[test]
fn unknown_theme_fails() {
    let output = cli(&[fixture("sample.ts").to_str().unwrap(), "--theme=nope"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("nope"));
}

#[test]
fn invalid_json_item_fails() {
    let dir = std::env::temp_dir().join(format!("shiki-rs-cli-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("bad.json");
    std::fs::write(&file, r#"[{"lang": "ts"}]"#).unwrap();
    let output = cli(&[file.to_str().unwrap(), "--json"]);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains(r#""input" must be a string"#));
}

#[test]
fn lists_themes_and_languages() {
    let themes = stdout(&cli(&["--list-themes"]));
    assert_eq!(themes.lines().count(), 65);
    assert!(themes.lines().any(|t| t == "monokai"));

    let languages = stdout(&cli(&["--list-languages"]));
    assert!(languages.lines().any(|l| l == "typescript (ts, cts, mts)"));
}
