use assert_cmd::prelude::*;
use std::fs;
use std::process::Command;

fn cleanup() {
    let _ = fs::remove_file("bookmarks.yaml");
}

#[test]
fn test_add_and_list_bookmark() -> Result<(), Box<dyn std::error::Error>> {
    // Clean all existing bookmarks
    cleanup();

    // Add a bookmark
    let mut cmd = Command::cargo_bin("bookmarker")?;
    cmd.arg("add")
        .arg("gh")
        .arg("https://github.com")
        .arg("--desc")
        .arg("The place for code")
        .arg("--tags")
        .arg("code,dev");
    cmd.assert().success();

    // List the bookmark
    let mut cmd2 = Command::cargo_bin("bookmarker")?;
    cmd2.arg("list");
    cmd2.assert()
        .success()
        .stdout(predicates::str::contains("gh"))
        .stdout(predicates::str::contains("https://github.com"))
        .stdout(predicates::str::contains("code, dev"));

    cleanup();
    Ok(())
}
