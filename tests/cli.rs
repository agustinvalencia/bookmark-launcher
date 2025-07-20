use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
// Use assert_fs to create a temporary home directory for our tests
use assert_fs::TempDir;
use assert_fs::prelude::*;

#[test]
fn test_add_and_list_bookmark() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a temporary directory to act as our "home"
    let temp = TempDir::new()?;
    let bookmark_file = temp.child(".config/bookmarker/bookmarks.yaml");

    // 2. Run the add command, pointing HOME to our temp dir
    let mut cmd = Command::cargo_bin("bookmarker")?;
    cmd.env("HOME", temp.path()); // Critically, override the home directory
    cmd.arg("add")
        .arg("gh")
        .arg("https://github.com")
        .arg("--desc")
        .arg("The place for code");
    cmd.assert().success();

    // 3. Assert that the bookmarks file was created correctly
    bookmark_file.assert(predicate::str::contains("url: https://github.com"));

    // 4. Run the list command and check the output
    let mut cmd2 = Command::cargo_bin("bookmarker")?;
    cmd2.env("HOME", temp.path());
    cmd2.arg("list");
    cmd2.assert()
        .success()
        .stdout(predicate::str::contains("gh"));

    Ok(())
}

// You can similarly update the test_delete_bookmark function
#[test]
fn test_delete_bookmark() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let bookmark_file = temp.child(".config/bookmarker/bookmarks.yaml");

    // 1. Add a bookmark first
    let mut cmd_add = Command::cargo_bin("bookmarker")?;
    cmd_add
        .env("HOME", temp.path())
        .arg("add")
        .arg("tmp")
        .arg("https://tmp.com")
        .arg("-d")
        .arg("temp");
    cmd_add.assert().success();
    bookmark_file.assert(predicate::str::contains("tmp")); // Check it was added

    // 2. Delete the bookmark
    let mut cmd_delete = Command::cargo_bin("bookmarker")?;
    cmd_delete.env("HOME", temp.path()).arg("delete").arg("tmp");
    cmd_delete.assert().success();

    // 3. The file should no longer contain the entry (it might be empty or just not have 'tmp')
    bookmark_file.assert(predicate::str::contains("tmp").not());

    Ok(())
}
