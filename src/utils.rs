
/// Collapses runs of ASCII whitespace into a single space.
///
/// Leading and trailing ASCII whitespace is removed as part of the collapse.
/// Non-ASCII whitespace is preserved.
///
/// ```
/// use tagsoup::normalize_whitespace;
///
/// assert_eq!(normalize_whitespace("hello\n\t world\r\nthere"), "hello world there");
/// assert_eq!(normalize_whitespace("  hello world  "), "hello world");
/// assert_eq!(normalize_whitespace("hello\u{00a0}world"), "hello\u{00a0}world");
/// assert!(normalize_whitespace(" \n\t\r ").is_empty());
/// ```
pub fn normalize_whitespace(s: &str) -> String {
	let mut result = String::with_capacity(s.len());
	for word in s.split_ascii_whitespace() {
		if !result.is_empty() {
			result.push(' ');
		}
		result.push_str(word);
	}
	return result;
}
