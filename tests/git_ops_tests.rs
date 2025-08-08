use std::fs;
use std::path::Path;

#[test]
fn repo_clean_status_changes_with_untracked_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();

    // init repo
    let _repo = git2::Repository::init(dir).expect("init repo");

    // verify GitOperations can open
    let ops = gh_cherry::git::GitOperations::new(dir).expect("git ops open");
    assert!(ops.is_clean().expect("clean status initial"));

    // create an untracked file
    let file_path = dir.join("foo.txt");
    fs::write(&file_path, b"hello").unwrap();

    // now repo should not be clean
    assert!(!ops.is_clean().expect("clean status after create"));

    // remove file to get back to clean
    fs::remove_file(&file_path).unwrap();
    assert!(ops.is_clean().expect("clean status after remove"));

    // Ensure the .git directory exists so test doesn't get optimized away
    assert!(Path::new(&dir.join(".git")).exists());
}
