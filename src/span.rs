
/// Span of the information in the parsed source.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Span {
	/// The byte offset of the start of the span.
	pub start: u32,
	/// The byte offset of the end of the span.
	pub end: u32,
}

impl Span {
	/// Constructor.
	#[inline]
	pub const fn new(start: usize, end: usize) -> Span {
		Span {
			start: start as u32,
			end: end as u32,
		}
	}

	/// Returns the line and column of the span in the source.
	///
	/// This runs in _O(n)_ time, where n is the length of the source.
	pub fn resolve<'a>(&self, source: &'a str) -> ResolvedSpan<'a> {
		let text = &source[self.start as usize..self.end as usize];
		let (start_line, start_column) = line_col(self.start as usize, source);
		let (end_line, end_column) = line_col(self.end as usize, source);
		ResolvedSpan {
			text,
			start_line,
			start_column,
			end_line,
			end_column,
		}
	}
}

pub(crate) fn combined_span(lhs: Span, rhs: Span) -> Span {
	let start = lhs.start.min(rhs.start);
	let end = lhs.end.max(rhs.end);
	Span { start, end }
}

/// Span of the information in the parsed source, with line and column information.
///
/// Lines are 1-indexed and columns are 0-indexed.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ResolvedSpan<'a> {
	pub text: &'a str,
	pub start_line: u32,
	pub start_column: u32,
	pub end_line: u32,
	pub end_column: u32,
}

impl<'a> ResolvedSpan<'a> {
	/// Gets a snippet of the span's text, limited to `max_len` characters.
	pub fn snippet(&self, max_len: usize) -> &'a str {
		let line = self.text.lines().next().unwrap_or("");
		let max_len = usize::min(max_len, line.len());
		&line[..max_len]
	}
}

fn line_col(offset: usize, source: &str) -> (u32, u32) {
	let mut line = 1; // Lines are 1-indexed.
	let mut column = 0; // Columns are 0-indexed.

	for (i, c) in source.char_indices() {
		if i >= offset {
			break;
		}

		if c == '\n' {
			line += 1;
			column = 0;
		}
		else {
			column += 1;
		}
	}

	(line, column)
}
