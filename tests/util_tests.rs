use gh_cherry::util::short_sha;

#[test]
fn short_sha_handles_short_and_long() {
    assert_eq!(short_sha("abc"), "abc");
    assert_eq!(short_sha("12345678"), "12345678");
    assert_eq!(short_sha("1234567890"), "12345678");
}
