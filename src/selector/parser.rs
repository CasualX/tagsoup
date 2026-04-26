use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ParseSelectorErrorKind {
	InvalidSelector,
	InvalidAttributeName,
	InvalidAttributeValue,
}

impl ParseSelectorErrorKind {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::InvalidSelector => "invalid selector",
			Self::InvalidAttributeName => "invalid attribute name",
			Self::InvalidAttributeValue => "invalid attribute value",
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParseSelectorError {
	pub span: SourceSpan,
	pub kind: ParseSelectorErrorKind,
}

impl fmt::Display for ParseSelectorError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}:{} {}", self.span.start, self.span.end, self.kind.as_str())
	}
}

impl error::Error for ParseSelectorError {
	fn description(&self) -> &str {
		self.kind.as_str()
	}
}

enum LexerState {
	Selector,
	AttrName,
	AttrValue,
	NthExpr,
}

pub struct Parser<'a> {
	input: &'a [u8],
	position: usize,
	steps: Vec<Step<'a>>,
	state: LexerState,
}

impl<'a> Parser<'a> {
	pub fn parse(input: &'a str) -> Result<Vec<Step<'a>>, ParseSelectorError> {
		let mut parser = Parser { input: input.as_bytes(), position: 0, steps: Vec::new(), state: LexerState::Selector };
		parser.skip_whitespace();

		if parser.is_eof() {
			return Err(parser.error_at(parser.position, parser.position, ParseSelectorErrorKind::InvalidSelector));
		}

		loop {
			let saw_whitespace = parser.skip_whitespace();
			if parser.is_eof() {
				break;
			}

			if matches!(parser.peek(), Some(b',')) {
				let comma_position = parser.position;
				parser.bump();

				if parser.steps.is_empty() || parser.last_is_combinator() || matches!(parser.steps.last(), Some(Step::SelectorList)) {
					return Err(parser.error_at(comma_position, parser.position, ParseSelectorErrorKind::InvalidSelector));
				}

				parser.steps.push(Step::SelectorList);
				continue;
			}

			if let Some(combinator) = parser.next_combinator() {
				let combinator_position = parser.position;
				parser.bump();

				if parser.last_is_combinator() || parser.steps.is_empty() {
					return Err(parser.error_at(combinator_position, parser.position, ParseSelectorErrorKind::InvalidSelector));
				}

				parser.steps.push(Step::Combinator(combinator));
				continue;
			}

			if saw_whitespace && !parser.steps.is_empty() && !parser.last_is_combinator() && !matches!(parser.steps.last(), Some(Step::SelectorList)) {
				parser.steps.push(Step::Combinator(Combinator::Descendant));
			}

			parser.parse_compound_selector()?;
		}

		if parser.last_is_combinator() {
			let position = parser.position;
			return Err(parser.error_at(position, position, ParseSelectorErrorKind::InvalidSelector));
		}
		if matches!(parser.steps.last(), Some(Step::SelectorList)) {
			let position = parser.position;
			return Err(parser.error_at(position, position, ParseSelectorErrorKind::InvalidSelector));
		}

		Ok(parser.steps)
	}

	fn parse_compound_selector(&mut self) -> Result<(), ParseSelectorError> {
		let mut parsed = false;

		loop {
			match self.peek() {
				Some(b'*') if !parsed => {
					self.bump();
					self.steps.push(Step::Filter(Filter::Universal));
					parsed = true;
				}
				Some(b'#') => {
					self.bump();
					let value = self.next_ident(ParseSelectorErrorKind::InvalidSelector)?;
					self.steps.push(Step::Filter(Filter::Id(value)));
					parsed = true;
				}
				Some(b'.') => {
					self.bump();
					let value = self.next_ident(ParseSelectorErrorKind::InvalidSelector)?;
					self.steps.push(Step::Filter(Filter::Class(value)));
					parsed = true;
				}
				Some(b'[') => {
					self.parse_attribute_selector()?;
					parsed = true;
				}
				Some(b':') => {
					self.parse_pseudo_class()?;
					parsed = true;
				}
				Some(b'>') | Some(b',') | None => break,
				Some(ch) if ch.is_ascii_whitespace() => break,
				Some(_) if !parsed => {
					let value = self.next_ident(ParseSelectorErrorKind::InvalidSelector)?;
					self.steps.push(Step::Filter(Filter::Tag(value)));
					parsed = true;
				}
				Some(_) => {
					return Err(self.error_at(self.position, self.position + 1, ParseSelectorErrorKind::InvalidSelector));
				}
			}
		}

		if parsed {
			Ok(())
		}
		else {
			Err(self.error_at(self.position, self.position, ParseSelectorErrorKind::InvalidSelector))
		}
	}

	fn parse_pseudo_class(&mut self) -> Result<(), ParseSelectorError> {
		self.expect(b':')?;
		let name = self.next_pseudo_ident()?;

		match name {
			"empty" => self.steps.push(Step::Filter(Filter::Empty)),
			"first-child" => self.steps.push(Step::Filter(Filter::FirstChild)),
			"last-child" => self.steps.push(Step::Filter(Filter::LastChild)),
			"only-child" => self.steps.push(Step::Filter(Filter::OnlyChild)),
			"nth-child" => {
				self.skip_whitespace();
				self.expect(b'(')?;
				self.skip_whitespace();
				self.state = LexerState::NthExpr;
				let expr = self.parse_nth_expr()?;
				self.skip_whitespace();
				self.expect(b')')?;
				self.state = LexerState::Selector;
				self.steps.push(Step::Filter(Filter::NthChild(expr)));
			}
			"nth-last-child" => {
				self.skip_whitespace();
				self.expect(b'(')?;
				self.skip_whitespace();
				self.state = LexerState::NthExpr;
				let expr = self.parse_nth_expr()?;
				self.skip_whitespace();
				self.expect(b')')?;
				self.state = LexerState::Selector;
				self.steps.push(Step::Filter(Filter::NthLastChild(expr)));
			}
			_ => return Err(self.error_at(self.position, self.position, ParseSelectorErrorKind::InvalidSelector)),
		}

		Ok(())
	}

	fn next_pseudo_ident(&mut self) -> Result<&'a str, ParseSelectorError> {
		self.slurp(|ch| ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'-'))
			.ok_or_else(|| self.error_at(self.position, self.position, ParseSelectorErrorKind::InvalidSelector))
	}

	fn parse_nth_expr(&mut self) -> Result<NthExpr, ParseSelectorError> {
		let start = self.position;
		while let Some(ch) = self.peek() {
			if ch == b')' {
				break;
			}
			self.bump();
		}

		let raw = unsafe_as_str(&self.input[start..self.position]);
		let expr: String = raw.bytes().filter(|ch| !ch.is_ascii_whitespace()).map(char::from).collect();
		if expr.is_empty() {
			return Err(self.error_at(start, self.position, ParseSelectorErrorKind::InvalidSelector));
		}

		if expr.eq_ignore_ascii_case("even") {
			return Ok(NthExpr::Even);
		}
		if expr.eq_ignore_ascii_case("odd") {
			return Ok(NthExpr::Odd);
		}

		let Some(n_pos) = expr.bytes().position(|ch| ch.eq_ignore_ascii_case(&b'n')) else {
			let b = expr.parse().map_err(|_| self.error_at(start, self.position, ParseSelectorErrorKind::InvalidSelector))?;
			return Ok(NthExpr::Formula { a: 0, b });
		};

		let a = match &expr[..n_pos] {
			"" | "+" => 1,
			"-" => -1,
			value => value.parse().map_err(|_| self.error_at(start, self.position, ParseSelectorErrorKind::InvalidSelector))?,
		};
		let b = match &expr[n_pos + 1..] {
			"" => 0,
			value => value.parse().map_err(|_| self.error_at(start, self.position, ParseSelectorErrorKind::InvalidSelector))?,
		};

		Ok(NthExpr::Formula { a, b })
	}

	fn parse_attribute_selector(&mut self) -> Result<(), ParseSelectorError> {
		self.state = LexerState::AttrName;
		self.expect(b'[')?;
		self.skip_whitespace();
		let name = self.next_ident(ParseSelectorErrorKind::InvalidAttributeName)?;
		self.skip_whitespace();

		match self.peek() {
			Some(b']') => {
				self.bump();
				self.steps.push(Step::Filter(Filter::AttrExists(name)));
			}
			Some(b'^') => {
				self.bump();
				self.expect(b'=')?;
				self.skip_whitespace();
				self.state = LexerState::AttrValue;
				let value = self.parse_attr_value()?;
				self.skip_whitespace();
				self.expect(b']')?;
				self.steps.push(Step::Filter(Filter::AttrStartsWith { name, value }));
			}
			Some(b'$') => {
				self.bump();
				self.expect(b'=')?;
				self.skip_whitespace();
				self.state = LexerState::AttrValue;
				let value = self.parse_attr_value()?;
				self.skip_whitespace();
				self.expect(b']')?;
				self.steps.push(Step::Filter(Filter::AttrEndsWith { name, value }));
			}
			Some(b'~') => {
				self.bump();
				self.expect(b'=')?;
				self.skip_whitespace();
				self.state = LexerState::AttrValue;
				let value = self.parse_attr_value()?;
				self.skip_whitespace();
				self.expect(b']')?;
				self.steps.push(Step::Filter(Filter::AttrWord { name, value }));
			}
			Some(b'|') => {
				self.bump();
				self.expect(b'=')?;
				self.skip_whitespace();
				self.state = LexerState::AttrValue;
				let value = self.parse_attr_value()?;
				self.skip_whitespace();
				self.expect(b']')?;
				self.steps.push(Step::Filter(Filter::AttrHyphen { name, value }));
			}
			Some(b'*') => {
				self.bump();
				self.expect(b'=')?;
				self.skip_whitespace();
				self.state = LexerState::AttrValue;
				let value = self.parse_attr_value()?;
				self.skip_whitespace();
				self.expect(b']')?;
				self.steps.push(Step::Filter(Filter::AttrContains { name, value }));
			}
			Some(b'=') => {
				self.bump();
				self.skip_whitespace();
				self.state = LexerState::AttrValue;
				let value = self.parse_attr_value()?;
				self.skip_whitespace();
				self.expect(b']')?;
				self.steps.push(Step::Filter(Filter::AttrEquals { name, value }));
			}
			_ => {
				return Err(self.error_at(self.position, self.position, ParseSelectorErrorKind::InvalidAttributeName));
			}
		}

		self.state = LexerState::Selector;
		Ok(())
	}

	fn parse_attr_value(&mut self) -> Result<&'a str, ParseSelectorError> {
		match self.peek() {
			Some(b'"') | Some(b'\'') => self.parse_quoted_attr_value(),
			Some(_) => {
				self.slurp(|ch| ch != b']' && !ch.is_ascii_whitespace())
					.ok_or_else(|| self.error_at(self.position, self.position, ParseSelectorErrorKind::InvalidAttributeValue))
			}
			None => Err(self.error_at(self.position, self.position, ParseSelectorErrorKind::InvalidAttributeValue)),
		}
	}

	fn parse_quoted_attr_value(&mut self) -> Result<&'a str, ParseSelectorError> {
		let start = self.position;
		let quote = self.bump().unwrap();
		let value_start = self.position;

		while let Some(ch) = self.peek() {
			if ch == quote {
				let value = unsafe_as_str(&self.input[value_start..self.position]);
				self.bump();
				return Ok(value);
			}
			self.bump();
		}

		Err(self.error_at(start, self.position, ParseSelectorErrorKind::InvalidAttributeValue))
	}

	#[inline]
	fn slurp(&mut self, f: impl Fn(u8) -> bool) -> Option<&'a str> {
		let start = self.position;
		while let Some(&ch) = self.input.get(self.position) {
			if f(ch) {
				self.position += 1;
			}
			else {
				break;
			}
		}

		(self.position != start).then(|| unsafe_as_str(&self.input[start..self.position]))
	}

	#[inline]
	fn next_ident(&mut self, kind: ParseSelectorErrorKind) -> Result<&'a str, ParseSelectorError> {
		let is_ident_char = match self.state {
			LexerState::AttrName => is_attr_ident_char as fn(u8) -> bool,
			LexerState::Selector | LexerState::AttrValue | LexerState::NthExpr => is_selector_ident_char as fn(u8) -> bool,
		};

		self.slurp(is_ident_char).ok_or_else(|| self.error_at(self.position, self.position, kind))
	}

	fn skip_whitespace(&mut self) -> bool {
		let start = self.position;
		self.slurp(|ch| ch.is_ascii_whitespace());
		self.position != start
	}

	fn expect(&mut self, expected: u8) -> Result<(), ParseSelectorError> {
		match self.bump() {
			Some(ch) if ch == expected => Ok(()),
			_ => Err(self.error_at(self.position, self.position, ParseSelectorErrorKind::InvalidSelector)),
		}
	}

	fn last_is_combinator(&self) -> bool {
		matches!(self.steps.last(), Some(Step::Combinator(_)))
	}

	fn next_combinator(&self) -> Option<Combinator> {
		match self.peek() {
			Some(b'>') => Some(Combinator::Child),
			Some(b'+') => Some(Combinator::NextSibling),
			Some(b'~') => Some(Combinator::SubsequentSibling),
			_ => None,
		}
	}

	fn is_eof(&self) -> bool {
		self.position >= self.input.len()
	}

	fn peek(&self) -> Option<u8> {
		self.input.get(self.position).copied()
	}

	fn bump(&mut self) -> Option<u8> {
		let ch = self.peek()?;
		self.position += 1;
		Some(ch)
	}

	fn error_at(&self, start: usize, end: usize, kind: ParseSelectorErrorKind) -> ParseSelectorError {
		ParseSelectorError { span: SourceSpan::new(start, end), kind }
	}
}

fn is_selector_ident_char(ch: u8) -> bool {
	ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'-')
}

fn is_attr_ident_char(ch: u8) -> bool {
	is_selector_ident_char(ch) || matches!(ch, b':' | b'@')
}
