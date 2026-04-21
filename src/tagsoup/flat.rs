use super::*;

// Incrementally parse a document into a flat list of tokens, without building a tree structure.
// The parser will attempt to recover from errors and continue parsing, but will not attempt to infer any structure from the tokens.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TagMarker {
	ElementOpen, // <tag attrs>
	ElementClose, // </tag attrs>
	SelfClosing, // <tag attrs />
	ProcessingInstruction, // <?pi attrs ?>
	Doctype, // <!DOCTYPE attrs>
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlatElement<'a> {
	pub id: Option<&'a str>,
	pub tag: &'a str,
	pub tag_span: SourceSpan,
	pub tag_marker: TagMarker,
	pub element_kind: ElementKind,
	pub attributes: Vec<Attribute<'a>>,
	pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlatComment<'a> {
	pub comment: &'a str,
	pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlatCData<'a> {
	pub data: &'a str,
	pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlatText<'a> {
	pub text: &'a str,
	pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FlatToken<'a> {
	Element(FlatElement<'a>),
	Comment(FlatComment<'a>),
	CData(FlatCData<'a>),
	Text(FlatText<'a>),
}

pub struct FlatDocument<'a> {
	pub tokens: Vec<FlatToken<'a>>,
	pub errors: Vec<ParseError>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum TagCloseMarker {
	TagClose,
	SelfTagClose,
}

fn consume_until_close<'a>(lexer: &mut Lexer<'a>) {
	for token in lexer.by_ref() {
		if matches!(token.kind, TokenKind::TagClose | TokenKind::SelfTagClose) {
			break;
		}
	}
}


fn push_text_token<'a>(tokens: &mut Vec<FlatToken<'a>>, source: &'a str, span: SourceSpan) {
	let text = &source[span.start as usize..span.end as usize];

	if let Some(FlatToken::Text(previous)) = tokens.last_mut()
		&& previous.span.end == span.start
	{
		previous.span = combined_span(previous.span, span);
		previous.text = &source[previous.span.start as usize..previous.span.end as usize];
		return;
	}

	tokens.push(FlatToken::Text(FlatText { text, span }));
}

fn push_source_span_text<'a>(tokens: &mut Vec<FlatToken<'a>>, source: &'a str, span: SourceSpan) {
	push_text_token(tokens, source, span);
}

fn update_element_id<'a>(id: &mut Option<&'a str>, key: &'a str, value: Option<&AttributeValue<'a>>) {
	if key == "id" && id.is_none() {
		*id = value.map(|value| value.value);
	}
}

fn parse_attributes_until_close<'a>(
	lexer: &mut Lexer<'a>,
	errors: &mut Vec<ParseError>,
	allow_self_tag_close: bool,
	eof_span: SourceSpan,
	missing_close_error_kind: ParseErrorKind,
) -> (Vec<Attribute<'a>>, Option<&'a str>, Option<(TagCloseMarker, SourceSpan)>) {
	let mut attributes = Vec::new();
	let mut id = None;
	let mut pending: Option<Token<'a>> = None;

	loop {
		let token = match pending.take() {
			Some(token) => token,
			None => match lexer.next() {
				Some(token) => token,
				None => {
					errors.push(ParseError { span: eof_span, kind: missing_close_error_kind });
					return (attributes, id, None);
				}
			},
		};

		match token.kind {
			TokenKind::TagClose => return (attributes, id, Some((TagCloseMarker::TagClose, token.span))),
			TokenKind::SelfTagClose => {
				if allow_self_tag_close {
					return (attributes, id, Some((TagCloseMarker::SelfTagClose, token.span)));
				}

				errors.push(ParseError { span: token.span, kind: ParseErrorKind::SelfClosingEndTag });
				return (attributes, id, Some((TagCloseMarker::SelfTagClose, token.span)));
			}
			TokenKind::Ident(key) => {
				let key_span = token.span;
				let mut value = None;
				let mut attr_span = key_span;

				match lexer.next() {
					Some(Token { kind: TokenKind::Equals, .. }) => match lexer.next() {
						Some(Token { kind: TokenKind::Quoted(quoted), span }) => {
							value = Some(AttributeValue { value: quoted, span });
							attr_span = combined_span(key_span, span);
						}
						Some(Token { kind: TokenKind::Ident(unquoted), span }) => {
							value = Some(AttributeValue { value: unquoted, span });
							attr_span = combined_span(key_span, span);
						}
						Some(other) => {
							errors.push(ParseError { span: other.span, kind: ParseErrorKind::InvalidAttributeValue });
							pending = Some(other);
						}
						None => {
							errors.push(ParseError { span: key_span, kind: missing_close_error_kind });
							update_element_id(&mut id, key, value.as_ref());
							attributes.push(Attribute { key, value, key_span, span: attr_span });
							return (attributes, id, None);
						}
					},
					Some(next) => pending = Some(next),
					None => {
						errors.push(ParseError { span: key_span, kind: missing_close_error_kind });
						update_element_id(&mut id, key, value.as_ref());
						attributes.push(Attribute { key, value, key_span, span: attr_span });
						return (attributes, id, None);
					}
				}

				update_element_id(&mut id, key, value.as_ref());
				attributes.push(Attribute { key, value, key_span, span: attr_span });
			}
			TokenKind::Error(_) => {
				errors.push(ParseError { span: token.span, kind: ParseErrorKind::InvalidAttribute });
			}
			_ => {
				errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken });
			}
		}
	}
}

fn parse_tag_open<'a>(lexer: &mut Lexer<'a>, source: &'a str, open: Token<'a>, tokens: &mut Vec<FlatToken<'a>>, errors: &mut Vec<ParseError>) -> Option<FlatElement<'a>> {
	let (tag, tag_span) = match lexer.next() {
		Some(Token { kind: TokenKind::Ident(tag), span }) => (tag, span),
		Some(other) => {
			errors.push(ParseError { span: other.span, kind: ParseErrorKind::MissingTagName });
			push_source_span_text(tokens, source, open.span);
			push_source_span_text(tokens, source, other.span);
			return None;
		}
		None => {
			errors.push(ParseError { span: open.span, kind: ParseErrorKind::UnterminatedTag });
			push_source_span_text(tokens, source, open.span);
			return None;
		}
	};

	let (attributes, id, close) = parse_attributes_until_close(lexer, errors, true, open.span, ParseErrorKind::UnterminatedTag);
	let Some((close_kind, close_span)) = close else {
		return None;
	};

	let tag_marker = match close_kind {
		TagCloseMarker::TagClose => TagMarker::ElementOpen,
		TagCloseMarker::SelfTagClose => TagMarker::SelfClosing,
	};

	let element_kind = ElementKind::from_tag(tag);

	Some(FlatElement {
		id,
		tag,
		tag_span,
		tag_marker,
		attributes,
		element_kind,
		span: combined_span(open.span, close_span),
	})
}

fn parse_tag_close<'a>(lexer: &mut Lexer<'a>, open: Token<'a>, tokens: &mut Vec<FlatToken<'a>>, errors: &mut Vec<ParseError>) {
	let (tag, tag_span) = match lexer.next() {
		Some(Token { kind: TokenKind::Ident(tag), span }) => (tag, span),
		Some(other) => {
			errors.push(ParseError { span: other.span, kind: ParseErrorKind::MissingTagName });
			consume_until_close(lexer);
			return;
		}
		None => {
			errors.push(ParseError { span: open.span, kind: ParseErrorKind::UnterminatedTag });
			return;
		}
	};

	let (attributes, id, close) = parse_attributes_until_close(lexer, errors, false, open.span, ParseErrorKind::UnterminatedTag);
	let Some((_close_kind, close_span)) = close else {
		return;
	};

	let element_kind = ElementKind::from_tag(tag);

	tokens.push(FlatToken::Element(FlatElement {
		id,
		tag,
		tag_span,
		tag_marker: TagMarker::ElementClose,
		attributes,
		element_kind,
		span: combined_span(open.span, close_span),
	}));
}

fn parse_doctype<'a>(lexer: &mut Lexer<'a>, open: Token<'a>, tokens: &mut Vec<FlatToken<'a>>, errors: &mut Vec<ParseError>) {
	let (tag, tag_span) = match lexer.next() {
		Some(Token { kind: TokenKind::Ident(tag), span }) => (tag, span),
		Some(other) => {
			errors.push(ParseError { span: other.span, kind: ParseErrorKind::MissingTagName });
			consume_until_close(lexer);
			return;
		}
		None => {
			errors.push(ParseError { span: open.span, kind: ParseErrorKind::UnterminatedDoctype });
			return;
		}
	};

	let (attributes, id, close) = parse_attributes_until_close(lexer, errors, true, open.span, ParseErrorKind::UnterminatedDoctype);
	let Some((_close_kind, close_span)) = close else {
		return;
	};

	tokens.push(FlatToken::Element(FlatElement {
		id,
		tag,
		tag_span,
		tag_marker: TagMarker::Doctype,
		element_kind: ElementKind::Void,
		attributes,
		span: combined_span(open.span, close_span),
	}));
}

fn parse_pi<'a>(lexer: &mut Lexer<'a>, open: Token<'a>, tokens: &mut Vec<FlatToken<'a>>, errors: &mut Vec<ParseError>) {
	let (tag, tag_span) = match lexer.next() {
		Some(Token { kind: TokenKind::Ident(tag), span }) => (tag, span),
		Some(other) => {
			errors.push(ParseError { span: other.span, kind: ParseErrorKind::MissingTagName });
			consume_until_close(lexer);
			return;
		}
		None => {
			errors.push(ParseError { span: open.span, kind: ParseErrorKind::UnterminatedProcessingInstruction });
			return;
		}
	};

	let mut attributes = Vec::new();
	let mut id = None;
	let mut close_span = open.span;
	let mut saw_close = false;
	let mut pending: Option<Token<'a>> = None;

	loop {
		let token = match pending.take() {
			Some(token) => token,
			None => match lexer.next() {
				Some(token) => token,
				None => break,
			},
		};

		match token.kind {
			TokenKind::PIClose => {
				saw_close = true;
				close_span = token.span;
				break;
			}
			TokenKind::TagClose => {
				errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnterminatedProcessingInstruction });
				saw_close = true;
				close_span = token.span;
				break;
			}
			TokenKind::Ident(key) => {
				let key_span = token.span;
				let mut value = None;
				let mut attr_span = key_span;

				match lexer.next() {
					Some(Token { kind: TokenKind::Equals, .. }) => match lexer.next() {
						Some(Token { kind: TokenKind::Quoted(quoted), span }) => {
							value = Some(AttributeValue { value: quoted, span });
							attr_span = combined_span(key_span, span);
						}
						Some(Token { kind: TokenKind::Ident(unquoted), span }) => {
							value = Some(AttributeValue { value: unquoted, span });
							attr_span = combined_span(key_span, span);
						}
						Some(other) => {
							errors.push(ParseError { span: other.span, kind: ParseErrorKind::InvalidAttributeValue });
							pending = Some(other);
						}
						None => {
							errors.push(ParseError { span: key_span, kind: ParseErrorKind::UnterminatedProcessingInstruction });
							update_element_id(&mut id, key, value.as_ref());
							attributes.push(Attribute { key, value, key_span, span: attr_span });
							break;
						}
					},
					Some(next) => pending = Some(next),
					None => {
						errors.push(ParseError { span: key_span, kind: ParseErrorKind::UnterminatedProcessingInstruction });
						update_element_id(&mut id, key, value.as_ref());
						attributes.push(Attribute { key, value, key_span, span: attr_span });
						break;
					}
				}

				update_element_id(&mut id, key, value.as_ref());
				attributes.push(Attribute { key, value, key_span, span: attr_span });
			}
			TokenKind::Error(_) => {
				errors.push(ParseError { span: token.span, kind: ParseErrorKind::InvalidAttribute });
			}
			_ => {
				errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken });
			}
		}
	}

	if !saw_close {
		errors.push(ParseError { span: open.span, kind: ParseErrorKind::UnterminatedProcessingInstruction });
		close_span = open.span;
	}

	tokens.push(FlatToken::Element(FlatElement {
		id,
		tag,
		tag_span,
		tag_marker: TagMarker::ProcessingInstruction,
		element_kind: ElementKind::Void,
		attributes,
		span: combined_span(open.span, close_span),
	}));
}

fn parse_comment<'a>(lexer: &mut Lexer<'a>, open: Token<'a>, tokens: &mut Vec<FlatToken<'a>>, errors: &mut Vec<ParseError>) {
	let mut content = "";
	let mut span = open.span;

	match lexer.next() {
		Some(Token { kind: TokenKind::Text(text), span: text_span }) => {
			content = text;
			span = combined_span(open.span, text_span);

			match lexer.next() {
				Some(Token { kind: TokenKind::CommentClose, span: close_span }) => {
					span = combined_span(span, close_span);
				}
				Some(other) => {
					errors.push(ParseError { span: other.span, kind: ParseErrorKind::UnterminatedComment });
					span = combined_span(span, other.span);
				}
				None => {
					errors.push(ParseError { span: open.span, kind: ParseErrorKind::UnterminatedComment });
				}
			}
		}
		Some(Token { kind: TokenKind::CommentClose, span: close_span }) => {
			span = combined_span(open.span, close_span);
		}
		Some(other) => {
			errors.push(ParseError { span: other.span, kind: ParseErrorKind::UnexpectedToken });
			span = combined_span(open.span, other.span);
		}
		None => {
			errors.push(ParseError { span: open.span, kind: ParseErrorKind::UnterminatedComment });
		}
	}

	tokens.push(FlatToken::Comment(FlatComment { comment: content, span }));
}

fn parse_cdata<'a>(lexer: &mut Lexer<'a>, open: Token<'a>, tokens: &mut Vec<FlatToken<'a>>, errors: &mut Vec<ParseError>) {
	let mut data = "";
	let mut span = open.span;

	match lexer.next() {
		Some(Token { kind: TokenKind::Text(text), span: text_span }) => {
			data = text;
			span = combined_span(open.span, text_span);

			match lexer.next() {
				Some(Token { kind: TokenKind::CDataClose, span: close_span }) => {
					span = combined_span(span, close_span);
				}
				Some(other) => {
					errors.push(ParseError { span: other.span, kind: ParseErrorKind::UnterminatedCData });
					span = combined_span(span, other.span);
				}
				None => {
					errors.push(ParseError { span: open.span, kind: ParseErrorKind::UnterminatedCData });
				}
			}
		}
		Some(Token { kind: TokenKind::CDataClose, span: close_span }) => {
			span = combined_span(open.span, close_span);
		}
		Some(other) => {
			errors.push(ParseError { span: other.span, kind: ParseErrorKind::UnexpectedToken });
			span = combined_span(open.span, other.span);
		}
		None => {
			errors.push(ParseError { span: open.span, kind: ParseErrorKind::UnterminatedCData });
		}
	}

	tokens.push(FlatToken::CData(FlatCData { data, span }));
}

pub fn parse_flat<'a>(source: &'a str) -> FlatDocument<'a> {
	let mut lexer = Lexer::new(source);
	let mut tokens = Vec::new();
	let mut errors = Vec::new();

	while let Some(token) = lexer.next() {
		match token.kind {
			TokenKind::TagOpen => {
				if let Some(element) = parse_tag_open(&mut lexer, source, token, &mut tokens, &mut errors) {
					let raw_text_tag = (element.tag_marker == TagMarker::ElementOpen && element.element_kind.is_raw_text())
						.then_some(element.tag);
					tokens.push(FlatToken::Element(element));

					if let Some(tag) = raw_text_tag {
						if let Some((_text, span)) = lexer.next_raw_text_until_close_tag(tag) {
							push_text_token(&mut tokens, source, span);
						}
					}
				}
			}
			TokenKind::SelfTagOpen => parse_tag_close(&mut lexer, token, &mut tokens, &mut errors),
			TokenKind::DoctypeOpen => parse_doctype(&mut lexer, token, &mut tokens, &mut errors),
			TokenKind::PIOpen => parse_pi(&mut lexer, token, &mut tokens, &mut errors),
			TokenKind::CommentOpen => parse_comment(&mut lexer, token, &mut tokens, &mut errors),
			TokenKind::CDataOpen => parse_cdata(&mut lexer, token, &mut tokens, &mut errors),
			TokenKind::Text(_text) => push_text_token(&mut tokens, source, token.span),
			TokenKind::Error(_) => errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken }),
			_ => {
				push_source_span_text(&mut tokens, source, token.span);
				errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken });
			}
		}
	}

	FlatDocument { tokens, errors }
}
