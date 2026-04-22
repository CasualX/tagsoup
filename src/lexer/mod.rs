//! TagSoup lexer.

use crate::SourceSpan;

/// Token kind.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
	/// The start of a processing instruction, e.g. `<?xml`.
	///
	/// ```
	/// let input = "<?xml ?>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[0].kind, tagsoup::lexer::TokenKind::PIOpen);
	/// assert_eq!(&input[tokens[0].span.range()], "xml");
	/// ```
	PIOpen,

	/// The end of a processing instruction, i.e. `?>`.
	///
	/// ```
	/// let input = "<?xml ?>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[1].kind, tagsoup::lexer::TokenKind::PIClose);
	/// assert_eq!(&input[tokens[1].span.range()], "?>");
	/// ```
	PIClose,

	/// An HTML comment token, including the full `<!-- ... -->` text.
	///
	/// ```
	/// let input = "<!-- comment -->";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[0].kind, tagsoup::lexer::TokenKind::Comment);
	/// assert_eq!(&input[tokens[0].span.range()], "<!-- comment -->");
	/// ```
	Comment,

	/// A CDATA section token, including the full `<![CDATA[ ... ]]>` text.
	///
	/// ```
	/// let input = "<![CDATA[x < y]]>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[0].kind, tagsoup::lexer::TokenKind::CData);
	/// assert_eq!(&input[tokens[0].span.range()], "<![CDATA[x < y]]>");
	/// ```
	CData,

	/// The identifier after `<!`, such as `DOCTYPE`.
	///
	/// ```
	/// let input = "<!DOCTYPE html>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[0].kind, tagsoup::lexer::TokenKind::DocTypeOpen);
	/// assert_eq!(&input[tokens[0].span.range()], "DOCTYPE");
	/// ```
	DocTypeOpen,

	/// A value inside a doctype declaration.
	///
	/// ```
	/// let input = "<!DOCTYPE html>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[1].kind, tagsoup::lexer::TokenKind::DocTypeValue);
	/// assert_eq!(&input[tokens[1].span.range()], "html");
	/// ```
	DocTypeValue,

	/// The `[` character starting the DTD subset in a doctype declaration.
	///
	/// ```
	/// let input = "<!DOCTYPE html [ ... ]>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[2].kind, tagsoup::lexer::TokenKind::DocTypeSubsetOpen);
	/// assert_eq!(&input[tokens[2].span.range()], "[");
	/// ```
	DocTypeSubsetOpen,

	/// The `]` character ending the DTD subset in a doctype declaration.
	///
	/// ```
	/// let input = "<!DOCTYPE html [ ... ]>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[4].kind, tagsoup::lexer::TokenKind::DocTypeSubsetClose);
	/// assert_eq!(&input[tokens[4].span.range()], "]");
	/// ```
	DocTypeSubsetClose,

	/// The closing `>` of a doctype declaration.
	///
	/// ```
	/// let input = "<!DOCTYPE html>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[2].kind, tagsoup::lexer::TokenKind::DocTypeClose);
	/// assert_eq!(&input[tokens[2].span.range()], ">");
	/// ```
	DocTypeClose,

	/// The tag name after `<`, such as `div`.
	///
	/// ```
	/// let input = "<div>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[0].kind, tagsoup::lexer::TokenKind::TagOpen);
	/// assert_eq!(&input[tokens[0].span.range()], "div");
	/// ```
	TagOpen,

	/// The tag name after `</`, such as `div`.
	///
	/// ```
	/// let input = "</div>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[0].kind, tagsoup::lexer::TokenKind::EndTagOpen);
	/// assert_eq!(&input[tokens[0].span.range()], "div");
	/// ```
	EndTagOpen,

	/// The closing `>` for a start tag or end tag.
	///
	/// ```
	/// let input = "<div>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[1].kind, tagsoup::lexer::TokenKind::TagClose);
	/// assert_eq!(&input[tokens[1].span.range()], ">");
	/// ```
	TagClose,

	/// The closing `/>` for a self-closing tag.
	///
	/// ```
	/// let input = "<br/>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[1].kind, tagsoup::lexer::TokenKind::TagSelfClose);
	/// assert_eq!(&input[tokens[1].span.range()], "/>");
	/// ```
	TagSelfClose,

	/// An attribute name inside a tag or processing instruction.
	///
	/// ```
	/// let input = "<div class=hero>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[1].kind, tagsoup::lexer::TokenKind::AttrName);
	/// assert_eq!(&input[tokens[1].span.range()], "class");
	/// ```
	AttrName,

	/// An attribute value without the leading `=`.
	///
	/// ```
	/// let input = "<div class=hero>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[2].kind, tagsoup::lexer::TokenKind::AttrValue);
	/// assert_eq!(&input[tokens[2].span.range()], "hero");
	/// ```
	AttrValue,

	/// Text content outside of tags.
	///
	/// ```
	/// let input = "hello";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[0].kind, tagsoup::lexer::TokenKind::Text);
	/// assert_eq!(&input[tokens[0].span.range()], "hello");
	/// ```
	Text,

	/// An error token for unrecognized or malformed input.
	///
	/// Returned for malformed attributes, skipping until the next `>` character.
	///
	/// ```
	/// let input = "<div \0bad>";
	/// let tokens: Vec<_> = tagsoup::lexer::Lexer::new(input.as_bytes()).collect();
	/// assert_eq!(tokens[1].kind, tagsoup::lexer::TokenKind::Error);
	/// assert_eq!(&input[tokens[1].span.range()], "\0bad");
	/// ```
	Error,
}

/// Token.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Token {
	/// The kind of token.
	pub kind: TokenKind,
	/// The byte span of the token in the input.
	///
	/// See the [TokenKind] documentation for what the span contains for each token kind.
	pub span: SourceSpan,
}

#[derive(Copy, Clone, Debug)]
enum LexerState {
	TagSoup, // Normal parsing state
	TagAttrs, // Inside a tag, looking for attributes
	TagAttrValue, // After an attribute name, looking for its value
	PIAttrs, // Inside a processing instruction, looking for attributes
	PIAttrValue, // Inside a processing instruction, looking for its value
	DocTypeElements, // Inside a doctype declaration, looking for elements
	DocTypeValue, // Inside a doctype declaration, looking for values
}

/// Lexer for tokenizing TagSoup input.
///
/// Lazy tokenization of the input string, producing a stream of tokens that can be iterated over.
/// Each token contains a `SourceSpan` that indicates the byte range of the token in the input string.
/// The lexer is designed to be robust against malformed input and will produce tokens for whatever it can recognize, even if the overall structure is not valid HTML.
#[derive(Clone)]
pub struct Lexer<'a> {
	input: &'a [u8],
	position: usize,
	doctype_depth: u32,
	state: LexerState,
}

impl<'a> Lexer<'a> {
	/// Constructor.
	#[inline]
	pub const fn new(input: &'a [u8]) -> Lexer<'a> {
		Lexer { input, position: 0, doctype_depth: 0, state: LexerState::TagSoup }
	}

	#[inline]
	fn is_raw_text_end_tag(&self, tag: &[u8]) -> bool {
		let Some(candidate) = self.input.get(self.position..) else {
			return false;
		};

		if !candidate.starts_with(b"</") {
			return false;
		}

		let Some(name) = candidate.get(2..2 + tag.len()) else {
			return false;
		};

		if !name.eq_ignore_ascii_case(tag) {
			return false;
		}

		match candidate.get(2 + tag.len()) {
			Some(b'>') | Some(b'/') => true,
			Some(next) => next.is_ascii_whitespace(),
			None => false,
		}
	}

	/// Reads raw text until the next occurrence of a close tag with the given name.
	///
	/// Skips over any input until `</tag` is found.
	pub fn raw_text(&mut self, tag: &[u8]) -> SourceSpan {
		let start = self.position;

		while let Some(pos) = memchr(b'<', &self.input[self.position..]) {
			self.position += pos;
			if self.is_raw_text_end_tag(tag) {
				return SourceSpan::new(start, self.position);
			}

			self.position += 1;
		}

		self.position = self.input.len();
		SourceSpan::new(start, self.position)
	}

	/// Skips ASCII whitespace characters and returns `true` if any were found.
	fn whitespace(&mut self) -> bool {
		let mut found = false;
		while let Some(&byte) = self.input.get(self.position) {
			if !byte.is_ascii_whitespace() {
				break;
			}
			self.position += 1;
			found = true;
		}
		found
	}

	/// Slurp characters while the predicate returns true, and return the slurped string slice.
	///
	/// Returns None if no characters were slurped.
	#[inline]
	fn slurp(&mut self, f: impl Fn(&u8) -> bool) -> SourceSpan {
		let start = self.position;
		while let Some(c) = self.input.get(self.position) {
			if f(c) {
				self.position += 1;
			}
			else {
				break;
			}
		}
		SourceSpan::new(start, self.position)
	}

	fn slurp_f(&mut self, b: u8, f: impl Fn(usize) -> bool) -> SourceSpan {
		let start = self.position;
		let mut position = self.position;
		while let Some(pos) = memchr(b, &self.input[position..]) {
			position += pos + 1;
			if f(position - 1) {
				self.position = position;
				return SourceSpan::new(start, self.position);
			}
		}
		self.position = self.input.len();
		SourceSpan::new(start, self.position)
	}

	fn tag(&mut self) -> SourceSpan {
		if !is_valid_name_start_char(&self.input[self.position..]) {
			return SourceSpan::new(self.position, self.position);
		}
		self.slurp(|&c| !(c.is_ascii_whitespace() || c == b'/' || c == b'>' || c <= 0x20))
	}

	fn name(&mut self) -> SourceSpan {
		self.slurp(|&c| !(c.is_ascii_whitespace() || c == b'=' || c == b'/' || c == b'>' || c <= 0x20))
	}

	fn value(&mut self) -> SourceSpan {
		let quote_char = self.input[self.position];
		if quote_char != b'"' && quote_char != b'\'' {
			let span = self.slurp(|&c| {
				let banned = c.is_ascii_whitespace() || c == b'"' || c == b'\'' || c == b'<' || c == b'>' || c == b'=' || c == b'`';
				(c >= 0x21 && c < 0x7F) && !banned
			});
			return span;
		}
		let start = self.position;
		self.position += 1; // Skip opening quote
		if let Some(pos) = memchr(quote_char, &self.input[self.position..]) {
			self.position += pos + 1; // Skip closing quote
			let span = SourceSpan::new(start, self.position);
			return span;
		}
		// Reached end of input without finding closing quote
		let span = SourceSpan::new(start, self.input.len());
		self.position = self.input.len();
		span
	}

	fn pi_tag(&mut self) -> SourceSpan {
		if !is_valid_name_start_char(&self.input[self.position..]) {
			return SourceSpan::new(self.position, self.position);
		}
		self.slurp(|&c| !(c.is_ascii_whitespace() || c == b'?' || c == b'>' || c <= 0x20))
	}

	fn pi_name(&mut self) -> SourceSpan {
		self.slurp(|&c| !(c.is_ascii_whitespace() || c == b'=' || c == b'?' || c == b'>' || c <= 0x20))
	}

	fn tag_soup(&mut self) -> Token {
		if self.input.get(self.position) == Some(&b'<') {
			let start = self.position;
			let tail = &self.input[self.position..];

			if tail.starts_with(b"<!--") {
				let span = self.slurp_f(b'>', |pos| self.input.get(pos - 2..pos) == Some(b"--"));
				return Token { kind: TokenKind::Comment, span };
			}
			else if tail.starts_with(b"<?") {
				self.position += 2; // Skip '<?'
				let span = self.pi_tag();
				if span.start == span.end {
					return Token { kind: TokenKind::Text, span: SourceSpan::new(start, self.position) };
				}
				self.state = LexerState::PIAttrs;
				Token { kind: TokenKind::PIOpen, span }
			}
			else if tail.starts_with(b"<![CDATA[") {
				let span = self.slurp_f(b'>', |pos| self.input.get(pos - 2..pos) == Some(b"]]"));
				return Token { kind: TokenKind::CData, span };
			}
			else if tail.starts_with(b"<!") {
				self.position += 2; // Skip '<!'
				let span = self.tag();
				if span.start == span.end {
					return Token { kind: TokenKind::Text, span: SourceSpan::new(start, self.position) };
				}
				self.state = LexerState::DocTypeValue;
				Token { kind: TokenKind::DocTypeOpen, span }
			}
			else if tail.starts_with(b"</") {
				self.position += 2; // Skip '</'
				let span = self.tag();
				if span.start == span.end {
					return Token { kind: TokenKind::Text, span: SourceSpan::new(start, self.position) };
				}
				self.state = LexerState::TagAttrs;
				Token { kind: TokenKind::EndTagOpen, span }
			}
			else { //if tail.starts_with(b"<") {
				self.position += 1; // Skip '<'
				let span = self.tag();
				if span.start == span.end {
					return Token { kind: TokenKind::Text, span: SourceSpan::new(start, self.position) };
				}
				self.state = LexerState::TagAttrs;
				Token { kind: TokenKind::TagOpen, span }
			}
		}
		else {
			let span = self.slurp(|&c| c != b'<');
			Token { kind: TokenKind::Text, span }
		}
	}

	fn tag_attrs(&mut self) -> Token {
		self.whitespace();

		let start = self.position;
		let tail = &self.input[self.position..];

		if tail.starts_with(b"/>") {
			self.position += 2; // Skip '/>'
			self.state = LexerState::TagSoup;
			Token { kind: TokenKind::TagSelfClose, span: SourceSpan::new(start, self.position) }
		}
		else if tail.starts_with(b">") {
			self.position += 1; // Skip '>'
			self.state = LexerState::TagSoup;
			Token { kind: TokenKind::TagClose, span: SourceSpan::new(start, self.position) }
		}
		else {
			let span = self.name();
			if span.start == span.end {
				// No valid attribute name found, skip until the next '>'
				let span = self.slurp(|&c| c != b'>');
				return Token { kind: TokenKind::Error, span };
			}
			self.whitespace();
			let tail = &self.input[self.position..];
			if tail.starts_with(b"=") {
				self.position += 1; // Skip '='
				self.state = LexerState::TagAttrValue;
			}
			else {
				self.state = LexerState::TagAttrs;
			}
			Token { kind: TokenKind::AttrName, span }
		}
	}

	fn tag_attrs_value(&mut self) -> Token {
		self.whitespace();
		let span = self.value();
		self.state = LexerState::TagAttrs;
		Token { kind: TokenKind::AttrValue, span }
	}

	fn pi_attrs(&mut self) -> Token {
		self.whitespace();

		let start = self.position;
		let tail = &self.input[self.position..];

		if tail.starts_with(b"?>") {
			self.position += 2; // Skip '?>'
			self.state = LexerState::TagSoup;
			Token { kind: TokenKind::PIClose, span: SourceSpan::new(start, self.position) }
		}
		else if tail.starts_with(b">") {
			self.position += 1; // Skip '>'
			self.state = LexerState::TagSoup;
			Token { kind: TokenKind::TagClose, span: SourceSpan::new(start, self.position) }
		}
		else {
			let span = self.pi_name();
			if span.start == span.end {
				// No valid attribute name found, skip until the next '>'
				let span = self.slurp(|&c| c != b'>');
				return Token { kind: TokenKind::Error, span };
			}
			self.whitespace();
			let tail = &self.input[self.position..];
			if tail.starts_with(b"=") {
				self.position += 1; // Skip '='
				self.state = LexerState::PIAttrValue;
			}
			else {
				self.state = LexerState::PIAttrs;
			}
			Token { kind: TokenKind::AttrName, span }
		}
	}

	fn pi_attrs_value(&mut self) -> Token {
		self.whitespace();
		let span = self.value();
		self.state = LexerState::PIAttrs;
		Token { kind: TokenKind::AttrValue, span }
	}

	fn doctype_element(&mut self) -> Token {
		self.whitespace();

		let tail = &self.input[self.position..];

		if tail.starts_with(b"<!") {
			self.position += 2; // Skip '<!'
			let span = self.tag();
			self.state = LexerState::DocTypeValue;
			Token { kind: TokenKind::DocTypeOpen, span }
		}
		else if tail.starts_with(b"]") {
			self.position += 1; // Skip ']'
			if self.doctype_depth > 0 {
				self.doctype_depth -= 1;
			}
			self.state = LexerState::DocTypeValue;
			Token { kind: TokenKind::DocTypeSubsetClose, span: SourceSpan::new(self.position - 1, self.position) }
		}
		else {
			// Text is not valid, skip until the next ']' or '<'
			let span = self.slurp(|&c| !(c == b']' || c == b'<'));
			Token { kind: TokenKind::Error, span }
		}
	}

	fn doctype_value(&mut self) -> Token {
		self.whitespace();

		let start = self.position;
		let Some(&c) = self.input.get(self.position) else {
			return Token { kind: TokenKind::Error, span: SourceSpan::new(start, start) };
		};

		if c == b'>' {
			self.position += 1; // Skip '>'
			self.state = if self.doctype_depth == 0 { LexerState::TagSoup } else { LexerState::DocTypeElements };
			Token { kind: TokenKind::DocTypeClose, span: SourceSpan::new(start, self.position) }
		}
		else if c == b'[' {
			self.position += 1; // Skip '['
			self.doctype_depth += 1;
			self.state = LexerState::DocTypeElements;
			Token { kind: TokenKind::DocTypeSubsetOpen, span: SourceSpan::new(start, self.position) }
		}
		else if c == b']' {
			self.position += 1; // Skip ']'
			if self.doctype_depth > 0 {
				self.doctype_depth -= 1;
			}
			Token { kind: TokenKind::DocTypeSubsetClose, span: SourceSpan::new(start, self.position) }
		}
		else {
			let span = self.value();
			if span.start == span.end {
				// No valid doctype value found, skip until the next '>'
				let span = self.slurp(|&c| c != b'>');
				return Token { kind: TokenKind::Error, span };
			}
			Token { kind: TokenKind::DocTypeValue, span }
		}
	}
}

impl<'a> Iterator for Lexer<'a> {
	type Item = Token;

	fn next(&mut self) -> Option<Self::Item> {
		if self.position >= self.input.len() {
			return None;
		}

		match self.state {
			LexerState::TagSoup => Some(self.tag_soup()),
			LexerState::TagAttrs => Some(self.tag_attrs()),
			LexerState::TagAttrValue => Some(self.tag_attrs_value()),
			LexerState::PIAttrs => Some(self.pi_attrs()),
			LexerState::PIAttrValue => Some(self.pi_attrs_value()),
			LexerState::DocTypeElements => Some(self.doctype_element()),
			LexerState::DocTypeValue => Some(self.doctype_value()),
		}
	}
}

fn memchr(byte: u8, haystack: &[u8]) -> Option<usize> {
	for (i, &b) in haystack.iter().enumerate() {
		if b == byte {
			return Some(i);
		}
	}
	None
}

fn is_valid_name_start_char(bytes: &[u8]) -> bool {
	let Some(&c) = bytes.get(0) else {
		return false;
	};

	// ASCII start characters
	if c < 128 {
		return matches!(c, b':' | b'_' | b'A'..=b'Z' | b'a'..=b'z');
	}

	// Non-ASCII characters
	let len = match c {
		0x00..=0x7F => 1,
		0xC0..=0xDF => 2,
		0xE0..=0xEF => 3,
		0xF0..=0xF7 => 4,
		_ => return false,
	};

	let bytes = &bytes[..len.min(bytes.len())];
	let Ok(s) = str::from_utf8(bytes) else {
		return false;
	};
	let Some(chr) = s.chars().next() else {
		return false;
	};

	let is_name_start_char =
		chr >= '\u{C0}' && chr <= '\u{D6}' ||
		chr >= '\u{D8}' && chr <= '\u{F6}' ||
		chr >= '\u{F8}' && chr <= '\u{2FF}' ||
		chr >= '\u{370}' && chr <= '\u{37D}' ||
		chr >= '\u{37F}' && chr <= '\u{1FFF}' ||
		chr >= '\u{200C}' && chr <= '\u{200D}' ||
		chr >= '\u{2070}' && chr <= '\u{218F}' ||
		chr >= '\u{2C00}' && chr <= '\u{2FEF}' ||
		chr >= '\u{3001}' && chr <= '\u{D7FF}' ||
		chr >= '\u{F900}' && chr <= '\u{FDCF}' ||
		chr >= '\u{FDF0}' && chr <= '\u{FFFD}' ||
		chr >= '\u{10000}' && chr <= '\u{EFFFF}';
	if !is_name_start_char {
		return false;
	}

	return true;
}

#[cfg(test)]
mod tests;
