// End-to-end tests that run the built binary against fixture files, so a
// change to argument parsing or line handling in main.rs gets caught even
// though main.rs itself has no unit tests.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).expect("fixture should exist")
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rscale"))
        .args(args)
        .output()
        .expect("binary should run")
}

fn run_with_stdin(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rscale"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary should spawn");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().expect("binary should finish")
}

#[test]
fn scales_recipe_to_recipe() {
    let input = fixture("pancakes.recipe");
    let output = run(&[
        "--from",
        "recipe",
        "--to",
        "recipe",
        "--scale",
        "2",
        "--input",
        input.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        read_fixture("pancakes_doubled.recipe")
    );
}

#[test]
fn converts_recipe_to_csv() {
    let input = fixture("pancakes.recipe");
    let output = run(&[
        "--from",
        "recipe",
        "--to",
        "csv",
        "--scale",
        "1",
        "--input",
        input.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        read_fixture("pancakes.csv")
    );
}

#[test]
fn converts_units_across_volume_and_weight() {
    let input = fixture("baking.recipe");
    let output = run(&[
        "--from",
        "recipe",
        "--to",
        "recipe",
        "--scale",
        "1",
        "--unit",
        "g",
        "--input",
        input.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        read_fixture("baking_grams.recipe")
    );
}

#[test]
fn reads_from_stdin_and_writes_to_stdout() {
    let input = read_fixture("pancakes.recipe");
    let output = run_with_stdin(&["--from", "recipe", "--to", "csv", "--scale", "1"], &input);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        read_fixture("pancakes.csv")
    );
}

#[test]
fn skips_unparsable_lines_but_still_succeeds() {
    let input = fixture("bad_input.recipe");
    let output = run(&[
        "--from",
        "recipe",
        "--to",
        "recipe",
        "--scale",
        "1",
        "--input",
        input.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        read_fixture("bad_input_skipped.recipe")
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("line 2:"), "stderr was: {stderr}");
}

#[test]
fn dry_run_reports_errors_and_exits_nonzero_without_writing() {
    let input = fixture("bad_input.recipe");
    let output = run(&[
        "--from",
        "recipe",
        "--to",
        "recipe",
        "--scale",
        "1",
        "--input",
        input.to_str().unwrap(),
        "--dry-run",
    ]);

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("3 line(s) checked, 1 error(s)"),
        "stdout was: {stdout}"
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("line 2:"), "stderr was: {stderr}");
}

#[test]
fn rejects_unknown_format() {
    let output = run(&["--from", "yaml", "--to", "csv", "--scale", "1"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unknown format: yaml"), "stderr was: {stderr}");
}
