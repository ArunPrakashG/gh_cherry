use gh_cherry::util::render_branch_name;

#[test]
fn branch_template_renders_task_id() {
    assert_eq!(
        render_branch_name("cherry-pick/{task_id}", "ABC-123"),
        "cherry-pick/ABC-123"
    );
}

#[test]
fn branch_template_multiple_placeholders() {
    assert_eq!(
        render_branch_name("{task_id}/fix-{task_id}", "JIRA-9"),
        "JIRA-9/fix-JIRA-9"
    );
}

#[test]
fn branch_template_without_placeholder_returns_same() {
    assert_eq!(render_branch_name("release", "X-1"), "release");
}
