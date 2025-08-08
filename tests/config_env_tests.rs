use std::fs;

#[test]
fn loads_env_overrides_from_cherry_env() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();
    let env_path = dir.join("cherry.env");
    fs::write(&env_path, r#"
GITHUB_OWNER="org"
GITHUB_REPO="repo"
BASE_BRANCH="main"
TARGET_BRANCH="release"
CHERRY_PICK_SOURCE_BRANCH="main"
BRANCH_NAME_TEMPLATE="ch/{task_id}"
ONLY_FORKED_REPOS=true
DAYS_BACK=14
"#).unwrap();

    // Change CWD for this test
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir).unwrap();

    let cfg = gh_cherry::config::Config::load(None).expect("config load");

    // restore CWD
    std::env::set_current_dir(prev).unwrap();

    assert_eq!(cfg.github.owner, "org");
    assert_eq!(cfg.github.repo, "repo");
    assert_eq!(cfg.github.base_branch, "main");
    assert_eq!(cfg.github.target_branch, "release");
    assert_eq!(cfg.github.cherry_pick_source_branch, "main");
    assert_eq!(cfg.github.branch_name_template, "ch/{task_id}");
    assert!(cfg.ui.only_forked_repos);
    assert_eq!(cfg.ui.days_back, 14);
}
