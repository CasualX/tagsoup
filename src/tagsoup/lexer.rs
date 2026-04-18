use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenKind<'a> {
	PIOpen, // <?
	PIClose, // ?>
	CommentOpen, // <!--
	CommentClose, // -->
	DoctypeOpen, // <!
	TagOpen, // <
	TagClose, // >
	SelfTagOpen, // </
	SelfTagClose, // />
	CDataOpen, // <![CDATA[
	CDataClose, // ]]>
	Ident(&'a str), // foo._baz, xml:bar, :class, @id, etc.
	Equals, // =
	Quoted(&'a str), // "foo" or 'foo'
	Text(&'a str), // text content between tags
	Error(&'a str), // Invalid token
}

#[expect(dead_code)]
impl<'a> TokenKind<'a> {
	/// Returns the textual representation of the token.
	pub fn as_text(&self) -> &'a str {
		match self {
			TokenKind::PIOpen => "<?",
			TokenKind::PIClose => "?>",
			TokenKind::CommentOpen => "<!--",
			TokenKind::CommentClose => "-->",
			TokenKind::DoctypeOpen => "<!",
			TokenKind::TagOpen => "<",
			TokenKind::TagClose => ">",
			TokenKind::SelfTagOpen => "</",
			TokenKind::SelfTagClose => "/>",
			TokenKind::CDataOpen => "<![CDATA[",
			TokenKind::CDataClose => "]]>",
			TokenKind::Ident(name) => name,
			TokenKind::Equals => "=",
			TokenKind::Quoted(value) => value,
			TokenKind::Text(value) => value,
			TokenKind::Error(value) => value,
		}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Token<'a> {
	pub kind: TokenKind<'a>,
	pub span: Span,
}

#[cfg(debug_assertions)]
#[inline]
fn unsafe_as_str(bytes: &[u8]) -> &str {
	std::str::from_utf8(bytes).unwrap()
}
#[cfg(not(debug_assertions))]
#[inline]
fn unsafe_as_str(bytes: &[u8]) -> &str {
	unsafe { std::str::from_utf8_unchecked(bytes) }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum LexerState {
	TagSoup, // Normal parsing state
	ProcessingInstruction, // Inside a processing instruction
	Comment, // Inside a comment
	TagAttrs, // Inside a tag
	TagAttrValue, // Reading an attribute value inside a tag
	PIAttrValue, // Reading an attribute value inside a processing instruction
	CData, // Inside a CDATA section
}

#[derive(Clone)]
pub struct Lexer<'a> {
	input: &'a [u8],
	position: usize,
	state: LexerState,
}

impl<'a> Lexer<'a> {
	pub const fn new(input: &'a str) -> Lexer<'a> {
		Lexer { input: input.as_bytes(), position: 0, state: LexerState::TagSoup }
	}

	#[inline]
	fn next_less_than(&mut self) -> bool {
		while self.position < self.input.len() {
			if self.input[self.position] == b'<' {
				return true;
			}

			self.position += 1;
		}

		false
	}

	#[inline]
	fn is_raw_text_close_tag_at_position(&self, tag: &[u8]) -> bool {
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

	pub(crate) fn next_raw_text_until_close_tag(&mut self, tag: &str) -> Option<Token<'a>> {
		let start = self.position;
		let tag = tag.as_bytes();

		while self.next_less_than() {
			if self.is_raw_text_close_tag_at_position(tag) {
				break;
			}

			self.position += 1;
		}

		if self.position == start {
			return None;
		}

		let text = unsafe_as_str(&self.input[start..self.position]);
		let span = Span::new(start, self.position);
		Some(Token { kind: TokenKind::Text(text), span })
	}

	// Slurp characters while the predicate returns true, and return the slurped string slice.
	// Returns None if no characters were slurped.
	#[inline]
	fn slurp(&mut self, f: impl Fn(&u8) -> bool) -> Option<(&'a str, Span)> {
		let start = self.position;
		while let Some(c) = self.input.get(self.position) {
			if f(c) {
				self.position += 1;
			}
			else {
				break;
			}
		}
		if self.position == start {
			return None;
		}
		let text = unsafe_as_str(&self.input[start..self.position]);
		let span = Span::new(start, self.position);
		Some((text, span))
	}

	// Skip ASCII whitespace characters.
	#[inline]
	fn skip_whitespace(&mut self) {
		self.slurp(u8::is_ascii_whitespace);
	}

	/// Slurp an identifier `[a-zA-Z0-9-.:_@$]+`.
	#[inline]
	fn next_ident(&mut self) -> Option<(&'a str, Span)> {
		self.slurp(|&c| c >= b'a' && c <= b'z' || c >= b'A' && c <= b'Z' || c >= b'0' && c <= b'9' || c == b'-' || c == b'.' || c == b':' || c == b'_' || c == b'@' || c == b'$')
	}

	/// Slurp an unquoted attribute value.
	#[inline]
	fn next_unquoted_attr_value(&mut self) -> Option<(&'a str, Span)> {
		self.slurp(|&c| {
			let banned = c.is_ascii_whitespace() || c == b'"' || c == b'\'' || c == b'<' || c == b'>' || c == b'=' || c == b'`';
			(c >= 0x21 && c < 0x7F) && !banned
		})
	}

	// Slurp an exact byte sequence.
	#[inline]
	fn next_exact(&mut self, s: &[u8]) -> Option<(&'a str, Span)> {
		let input = &self.input[self.position..];
		if input.starts_with(s) {
			let text = unsafe_as_str(&input[..s.len()]);
			let span = Span::new(self.position, self.position + s.len());
			self.position += s.len();
			Some((text, span))
		}
		else {
			None
		}
	}

	#[inline]
	fn next_quoted(&mut self) -> Option<(&'a str, Span)> {
		let quote_char = *self.input.get(self.position)?;
		if quote_char != b'"' && quote_char != b'\'' {
			return None;
		}
		self.position += 1; // Skip opening quote
		let start = self.position;
		while let Some(&c) = self.input.get(self.position) {
			if c == quote_char {
				let text = unsafe_as_str(&self.input[start..self.position]);
				let span = Span::new(start - 1, self.position + 1); // Include quotes in span
				self.position += 1; // Skip closing quote
				return Some((text, span));
			}
			else {
				self.position += 1;
			}
		}
		// Reached end of input without finding closing quote
		let text = unsafe_as_str(&self.input[start..]);
		let span = Span::new(start - 1, self.input.len());
		self.position = self.input.len(); // Move to end to prevent further tokens
		Some((text, span))
	}

	fn next_until_or_eof(&mut self, end: &[u8]) -> Option<(&'a str, Span)> {
		let start = self.position;
		loop {
			let current = &self.input[self.position..];
			if current.starts_with(end) {
				break;
			}
			else if self.position >= self.input.len() {
				// Reached end of input without finding end sequence
				break;
			}
			else {
				self.position += 1;
			}
		}
		if self.position == start {
			return None;
		}
		let text = unsafe_as_str(&self.input[start..self.position]);
		let span = Span::new(start, self.position);
		Some((text, span))
	}

	fn next_until(&mut self, end: &[u8]) -> Option<(&'a str, Span)> {
		let start = self.position;
		loop {
			let current = &self.input[self.position..];
			if current.starts_with(end) {
				break;
			}
			else if self.position >= self.input.len() {
				// Reached end of input without finding end sequence
				return None;
			}
			else {
				self.position += 1;
			}
		}
		if self.position == start {
			return None;
		}
		let text = unsafe_as_str(&self.input[start..self.position]);
		let span = Span::new(start, self.position);
		Some((text, span))
	}

	fn next_error(&mut self) -> Token<'a> {
		let text = unsafe_as_str(&self.input[self.position..]);
		let span = Span::new(self.position, self.input.len());
		self.position = self.input.len(); // Move to end to prevent further tokens
		Token { kind: TokenKind::Error(text), span }
	}

	fn next_token(&mut self) -> Option<Token<'a>> {
		if self.position >= self.input.len() {
			return None;
		}

		match self.state {
			LexerState::TagSoup => self.next_token_tagsoup(),
			LexerState::ProcessingInstruction => self.next_token_pi(),
			LexerState::Comment => self.next_token_comment(),
			LexerState::TagAttrs => self.next_token_tagattrs(),
			LexerState::TagAttrValue => self.next_token_tag_attr_value(),
			LexerState::PIAttrValue => self.next_token_pi_attr_value(),
			LexerState::CData => self.next_token_cdata(),
		}
	}

	fn next_token_tagsoup(&mut self) -> Option<Token<'a>> {
		if let Some((_text, span)) = self.next_exact(b"<?") {
			self.state = LexerState::ProcessingInstruction;
			return Some(Token { kind: TokenKind::PIOpen, span });
		}

		if let Some((_text, span)) = self.next_exact(b"<!--") {
			self.state = LexerState::Comment;
			return Some(Token { kind: TokenKind::CommentOpen, span });
		}

		if let Some((_text, span)) = self.next_exact(b"<![CDATA[") {
			self.state = LexerState::CData;
			return Some(Token { kind: TokenKind::CDataOpen, span });
		}

		if let Some((_text, span)) = self.next_exact(b"<!") {
			self.state = LexerState::TagAttrs; // Reuse TagAttrs state for DOCTYPE
			return Some(Token { kind: TokenKind::DoctypeOpen, span });
		}

		if let Some((_text, span)) = self.next_exact(b"</") {
			self.state = LexerState::TagAttrs; // Reuse TagAttrs state for closing tags
			return Some(Token { kind: TokenKind::SelfTagOpen, span });
		}

		if let Some((_text, span)) = self.next_exact(b"<") {
			self.state = LexerState::TagAttrs;
			return Some(Token { kind: TokenKind::TagOpen, span });
		}

		if let Some((text, span)) = self.next_until_or_eof(b"<") {
			return Some(Token { kind: TokenKind::Text(text), span });
		}

		if self.position >= self.input.len() {
			return None;
		}

		// Should never reach here, either next_exact("<") or next_until("<") should have matched
		#[cfg(debug_assertions)]
		unreachable!("Lexer in TagSoup state should always match either a tag open or text token");
		#[cfg(not(debug_assertions))]
		return None;
	}

	fn next_token_pi(&mut self) -> Option<Token<'a>> {
		self.skip_whitespace();

		if let Some((_text, span)) = self.next_exact(b"?>") {
			self.state = LexerState::TagSoup;
			return Some(Token { kind: TokenKind::PIClose, span });
		}

		if let Some((text, span)) = self.next_ident() {
			return Some(Token { kind: TokenKind::Ident(text), span });
		}

		if let Some((_text, span)) = self.next_exact(b"=") {
			self.state = LexerState::PIAttrValue;
			return Some(Token { kind: TokenKind::Equals, span });
		}

		if let Some((text, span)) = self.next_quoted() {
			return Some(Token { kind: TokenKind::Quoted(text), span });
		}

		// Fallback to slurp until ">" to avoid getting stuck on malformed processing instructions
		let position = self.position;
		if let Some((text, span)) = self.next_until_or_eof(b">") {

			if self.position >= self.input.len() {
				// Reached end of input without finding ">", return error token
				return Some(Token { kind: TokenKind::Error(text), span });
			}

			// Fallback to slurp until "?>" to avoid getting stuck on malformed processing instructions
			let mut tmp = Lexer { input: &self.input[..self.position + 1], position, state: LexerState::ProcessingInstruction };
			if let Some((text, span)) = tmp.next_until(b"?>") {
				self.position = tmp.position;
				return Some(Token { kind: TokenKind::Error(text), span });
			}

			self.state = LexerState::TagAttrs; // Switch to TagAttrs state to allow parsing closing ">"
			return Some(Token { kind: TokenKind::Error(text), span });
		}

		Some(self.next_error())
	}

	fn next_token_comment(&mut self) -> Option<Token<'a>> {
		if let Some((_text, span)) = self.next_exact(b"-->") {
			self.state = LexerState::TagSoup;
			return Some(Token { kind: TokenKind::CommentClose, span });
		}

		if let Some((text, span)) = self.next_until_or_eof(b"-->") {
			return Some(Token { kind: TokenKind::Text(text), span });
		}

		if self.position >= self.input.len() {
			return None;
		}

		// Should never reach here, either next_exact("-->") or next_until("-->") should have matched
		#[cfg(debug_assertions)]
		unreachable!("Lexer in Comment state should always match either a comment close or text token");
		#[cfg(not(debug_assertions))]
		return None;
	}

	fn next_token_tagattrs(&mut self) -> Option<Token<'a>> {
		self.skip_whitespace();

		if let Some((_text, span)) = self.next_exact(b"/>") {
			self.state = LexerState::TagSoup;
			return Some(Token { kind: TokenKind::SelfTagClose, span });
		}

		if let Some((_text, span)) = self.next_exact(b">") {
			self.state = LexerState::TagSoup;
			return Some(Token { kind: TokenKind::TagClose, span });
		}

		if let Some((text, span)) = self.next_ident() {
			return Some(Token { kind: TokenKind::Ident(text), span });
		}

		if let Some((_text, span)) = self.next_exact(b"=") {
			self.state = LexerState::TagAttrValue;
			return Some(Token { kind: TokenKind::Equals, span });
		}

		if let Some((text, span)) = self.next_quoted() {
			return Some(Token { kind: TokenKind::Quoted(text), span });
		}

		if let Some((text, span)) = self.next_until_or_eof(b">") {
			return Some(Token { kind: TokenKind::Error(text), span });
		}

		Some(self.next_error())
	}

	fn next_token_tag_attr_value(&mut self) -> Option<Token<'a>> {
		self.skip_whitespace();

		if let Some((_text, span)) = self.next_exact(b"/>") {
			self.state = LexerState::TagSoup;
			return Some(Token { kind: TokenKind::SelfTagClose, span });
		}

		if let Some((_text, span)) = self.next_exact(b">") {
			self.state = LexerState::TagSoup;
			return Some(Token { kind: TokenKind::TagClose, span });
		}

		if let Some((text, span)) = self.next_quoted() {
			self.state = LexerState::TagAttrs;
			return Some(Token { kind: TokenKind::Quoted(text), span });
		}

		if let Some((text, span)) = self.next_unquoted_attr_value() {
			self.state = LexerState::TagAttrs;
			return Some(Token { kind: TokenKind::Ident(text), span });
		}

		self.state = LexerState::TagAttrs;
		self.next_token_tagattrs()
	}

	fn next_token_pi_attr_value(&mut self) -> Option<Token<'a>> {
		self.skip_whitespace();

		if let Some((_text, span)) = self.next_exact(b"?>") {
			self.state = LexerState::TagSoup;
			return Some(Token { kind: TokenKind::PIClose, span });
		}

		if let Some((_text, span)) = self.next_exact(b">") {
			self.state = LexerState::TagSoup;
			return Some(Token { kind: TokenKind::TagClose, span });
		}

		if let Some((text, span)) = self.next_quoted() {
			self.state = LexerState::ProcessingInstruction;
			return Some(Token { kind: TokenKind::Quoted(text), span });
		}

		if let Some((text, span)) = self.next_unquoted_attr_value() {
			self.state = LexerState::ProcessingInstruction;
			return Some(Token { kind: TokenKind::Ident(text), span });
		}

		self.state = LexerState::ProcessingInstruction;
		self.next_token_pi()
	}

	fn next_token_cdata(&mut self) -> Option<Token<'a>> {
		if let Some((_text, span)) = self.next_exact(b"]]>") {
			self.state = LexerState::TagSoup;
			return Some(Token { kind: TokenKind::CDataClose, span });
		}

		if let Some((text, span)) = self.next_until_or_eof(b"]]>") {
			return Some(Token { kind: TokenKind::Text(text), span });
		}

		if self.position >= self.input.len() {
			return None;
		}

		// Should never reach here, either next_exact("]]>") or next_until("]]>") should have matched
		#[cfg(debug_assertions)]
		unreachable!("Lexer in CDATA state should always match either a CDATA close or text token");
		#[cfg(not(debug_assertions))]
		return None;
	}
}

impl<'a> Iterator for Lexer<'a> {
	type Item = Token<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.next_token()
	}
}
