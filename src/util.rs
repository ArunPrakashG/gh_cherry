/// Returns a short prefix of a SHA (up to 8 chars) without panicking on short inputs.
pub fn short_sha(sha: &str) -> &str {
    if sha.len() >= 8 {
        &sha[..8]
    } else {
        sha
    }
}

/// Renders a branch name from a template by replacing `{task_id}` with the given task id.
/// If the template has multiple placeholders, all are replaced. If there is no placeholder,
/// the template is returned unchanged.
pub fn render_branch_name(template: &str, task_id: &str) -> String {
    template.replace("{task_id}", task_id)
}
