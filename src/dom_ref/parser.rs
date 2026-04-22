use super::*;

struct OpenElement<'a> {
	element: Element<'a>,
	open_span: SourceSpan,
}

#[derive(Copy, Clone)]
enum TagEnd {
	Close(SourceSpan),
	SelfClose(SourceSpan),
}

struct PendingAttribute<'a> {
	key: &'a str,
	key_span: SourceSpan,
}

struct Parser<'a> {
	input: &'a str,
	lexer: lexer::Lexer<'a>,
	stack: Vec<OpenElement<'a>>,
	children: Vec<Node<'a>>,
	errors: Vec<ParseError>,
	scratch_attributes: Vec<Attribute<'a>>,
	current_id: Option<&'a str>,
	pending_attribute: Option<PendingAttribute<'a>>,
}

fn node_span(node: &Node<'_>) -> SourceSpan {
	match node {
		Node::Text(text) => text.span,
		Node::Element(element) => element.span,
		Node::Comment(comment) => comment.span,
		Node::Doctype(doctype) => doctype.span,
		Node::ProcessingInstruction(pi) => pi.span,
	}
}

fn update_element_id<'a>(id: &mut Option<&'a str>, key: &'a str, value: Option<&AttributeValue<'a>>) {
	if key == "id" && id.is_none() {
		*id = value.map(|value| value.value);
	}
}

fn strip_quotes(value: &str) -> &str {
	let bytes = value.as_bytes();
	if bytes.len() >= 2 {
		let first = bytes[0];
		let last = bytes[bytes.len() - 1];
		if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
			return unsafe_as_str(&bytes[1..bytes.len() - 1]);
		}
	}

	value
}

fn build_doctype<'a>(
	keyword: &'a str,
	keyword_span: SourceSpan,
	args: Vec<AttributeValue<'a>>,
	dtd: Vec<DoctypeNode<'a>>,
	span: SourceSpan,
) -> DoctypeNode<'a> {
	DoctypeNode { keyword, keyword_span, args, dtd, span }
}

fn build_processing_instruction<'a>(
	target: &'a str,
	target_span: SourceSpan,
	data: Vec<Attribute<'a>>,
	span: SourceSpan,
) -> ProcessingInstructionNode<'a> {
	ProcessingInstructionNode { target, target_span, data, span }
}

fn push_node<'a>(stack: &mut [OpenElement<'a>], children: &mut Vec<Node<'a>>, node: Node<'a>) {
	let span = node_span(&node);
	if let Some(parent) = stack.last_mut() {
		parent.element.span = combined_span(parent.element.span, span);
		parent.element.children.push(node);
	}
	else {
		children.push(node);
	}
}

fn push_text_node<'a>(stack: &mut [OpenElement<'a>], children: &mut Vec<Node<'a>>, input: &'a str, span: SourceSpan) {
	let target = if let Some(parent) = stack.last_mut() {
		parent.element.span = combined_span(parent.element.span, span);
		&mut parent.element.children
	}
	else {
		children
	};

	if let Some(Node::Text(previous)) = target.last_mut()
		&& previous.span.end == span.start
	{
		previous.span = combined_span(previous.span, span);
		previous.text = &input[previous.span.range()];
		return;
	}

	target.push(Node::Text(TextNode { text: &input[span.range()], span }));
}

fn build_element<'a>(
	id: Option<&'a str>,
	tag: &'a str,
	tag_span: SourceSpan,
	attributes: Vec<Attribute<'a>>,
	span: SourceSpan,
) -> Element<'a> {
	Element {
		id,
		tag,
		tag_span,
		kind: ElementKind::from_tag(tag),
		attributes,
		children: Vec::new(),
		span,
	}
}

fn finish_open_element<'a>(stack: &mut Vec<OpenElement<'a>>, children: &mut Vec<Node<'a>>) {
	if let Some(open) = stack.pop() {
		push_node(stack.as_mut_slice(), children, Node::Element(open.element));
	}
}

fn recover_to_matching_close<'a>(
	stack: &mut Vec<OpenElement<'a>>,
	children: &mut Vec<Node<'a>>,
	errors: &mut Vec<ParseError>,
	close_tag: &'a str,
	close_span: SourceSpan,
) -> bool {
	let Some(match_index) = stack.iter().rposition(|open| open.element.tag == close_tag) else {
		return false;
	};

	while stack.len() - 1 > match_index {
		if let Some(open) = stack.last() {
			errors.push(ParseError { span: open.open_span, kind: ParseErrorKind::UnclosedElement });
		}
		finish_open_element(stack, children);
	}

	if let Some(open) = stack.last_mut() {
		open.element.span = combined_span(open.element.span, close_span);
	}
	finish_open_element(stack, children);
	true
}

impl<'a> Parser<'a> {
	fn new(input: &'a str) -> Self {
		Self {
			input,
			lexer: lexer::Lexer::new(input.as_bytes()),
			stack: Vec::new(),
			children: Vec::new(),
			errors: Vec::new(),
			scratch_attributes: Vec::new(),
			current_id: None,
			pending_attribute: None,
		}
	}

	fn text_for_span(&self, span: SourceSpan) -> &'a str {
		self.input.get(span.range()).unwrap_or("")
	}

	fn tag_open_span(tag_span: SourceSpan) -> SourceSpan {
		SourceSpan { start: tag_span.start.saturating_sub(1), end: tag_span.end }
	}

	fn end_tag_open_span(tag_span: SourceSpan) -> SourceSpan {
		SourceSpan { start: tag_span.start.saturating_sub(2), end: tag_span.end }
	}

	fn markup_open_span(tag_span: SourceSpan) -> SourceSpan {
		SourceSpan { start: tag_span.start.saturating_sub(2), end: tag_span.end }
	}

	fn clear_attribute_state(&mut self) {
		self.scratch_attributes.clear();
		self.current_id = None;
		self.pending_attribute = None;
	}

	fn push_pending_attribute_without_value(&mut self) {
		if let Some(attribute) = self.pending_attribute.take() {
			update_element_id(&mut self.current_id, attribute.key, None);
			self.scratch_attributes.push(Attribute {
				key: attribute.key,
				value: None,
				key_span: attribute.key_span,
				span: attribute.key_span,
			});
		}
	}

	fn push_pending_attribute_with_value(&mut self, value_span: SourceSpan) {
		let Some(attribute) = self.pending_attribute.take() else {
			self.errors.push(ParseError { span: value_span, kind: ParseErrorKind::InvalidAttributeValue });
			return;
		};

		let value = AttributeValue {
			value: strip_quotes(self.text_for_span(value_span)),
			span: value_span,
		};
		update_element_id(&mut self.current_id, attribute.key, Some(&value));
		self.scratch_attributes.push(Attribute {
			key: attribute.key,
			value: Some(value),
			key_span: attribute.key_span,
			span: combined_span(attribute.key_span, value_span),
		});
	}

	fn take_attributes(&mut self) -> (Vec<Attribute<'a>>, Option<&'a str>) {
		self.push_pending_attribute_without_value();
		(mem::take(&mut self.scratch_attributes), self.current_id.take())
	}

	fn parse_tag_attributes(&mut self, allow_self_close: bool, missing_close_error_kind: ParseErrorKind) -> Option<TagEnd> {
		self.clear_attribute_state();

		loop {
			let Some(token) = self.lexer.next() else {
				self.push_pending_attribute_without_value();
				self.errors.push(ParseError { span: SourceSpan::new(self.input.len(), self.input.len()), kind: missing_close_error_kind });
				return None;
			};

			match token.kind {
				lexer::TokenKind::AttrName => {
					self.push_pending_attribute_without_value();
					self.pending_attribute = Some(PendingAttribute {
						key: self.text_for_span(token.span),
						key_span: token.span,
					});
				}
				lexer::TokenKind::AttrValue => self.push_pending_attribute_with_value(token.span),
				lexer::TokenKind::TagClose => {
					self.push_pending_attribute_without_value();
					return Some(TagEnd::Close(token.span));
				}
				lexer::TokenKind::TagSelfClose => {
					self.push_pending_attribute_without_value();
					if !allow_self_close {
						self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::SelfClosingEndTag });
					}
					return Some(TagEnd::SelfClose(token.span));
				}
				lexer::TokenKind::Error => {
					self.push_pending_attribute_without_value();
					self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::InvalidAttribute });
				}
				_ => {
					self.push_pending_attribute_without_value();
					self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken });
				}
			}
		}
	}

	fn push_text(&mut self, span: SourceSpan) {
		push_text_node(self.stack.as_mut_slice(), &mut self.children, self.input, span);
	}

	fn parse_comment(&mut self, span: SourceSpan) {
		let text = self.text_for_span(span);
		let comment = if let Some(inner) = text.strip_prefix("<!--").and_then(|value| value.strip_suffix("-->")) {
			inner
		}
		else {
			self.errors.push(ParseError { span, kind: ParseErrorKind::UnterminatedComment });
			text.strip_prefix("<!--").unwrap_or(text)
		};

		push_node(
			self.stack.as_mut_slice(),
			&mut self.children,
			Node::Comment(CommentNode { comment, span }),
		);
	}

	fn parse_cdata(&mut self, span: SourceSpan) {
		let text = self.text_for_span(span);
		let cdata = if let Some(inner) = text.strip_prefix("<![CDATA[").and_then(|value| value.strip_suffix("]]>") ) {
			inner
		}
		else {
			self.errors.push(ParseError { span, kind: ParseErrorKind::UnterminatedCData });
			text.strip_prefix("<![CDATA[").unwrap_or(text)
		};

		push_node(
			self.stack.as_mut_slice(),
			&mut self.children,
			Node::Text(TextNode { text: cdata, span }),
		);
	}

	fn parse_doctype_node(&mut self, keyword_span: SourceSpan) -> DoctypeNode<'a> {
		let keyword = self.text_for_span(keyword_span);
		let open_span = Self::markup_open_span(keyword_span);
		let mut args = Vec::new();
		let mut dtd = Vec::new();
		let mut span = open_span;
		let mut subset_depth = 0usize;

		loop {
			let Some(token) = self.lexer.next() else {
				self.errors.push(ParseError { span: open_span, kind: ParseErrorKind::UnterminatedDoctype });
				return build_doctype(keyword, keyword_span, args, dtd, span);
			};

			match token.kind {
				lexer::TokenKind::DocTypeValue => {
					let value_span = token.span;
					args.push(AttributeValue { value: strip_quotes(self.text_for_span(value_span)), span: value_span });
					span = combined_span(span, value_span);
				}
				lexer::TokenKind::DocTypeOpen => {
					let child = self.parse_doctype_node(token.span);
					span = combined_span(span, child.span);
					dtd.push(child);
				}
				lexer::TokenKind::DocTypeSubsetOpen => {
					subset_depth += 1;
					span = combined_span(span, token.span);
				}
				lexer::TokenKind::DocTypeSubsetClose => {
					if subset_depth == 0 {
						self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken });
					}
					else {
						subset_depth -= 1;
					}
					span = combined_span(span, token.span);
				}
				lexer::TokenKind::DocTypeClose => {
					return build_doctype(keyword, keyword_span, args, dtd, combined_span(span, token.span));
				}
				lexer::TokenKind::Error => {
					self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::InvalidAttribute });
					span = combined_span(span, token.span);
				}
				_ => {
					self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken });
					span = combined_span(span, token.span);
				}
			}
		}
	}

	fn parse_doctype(&mut self, tag_span: SourceSpan) {
		let doctype = self.parse_doctype_node(tag_span);
		push_node(self.stack.as_mut_slice(), &mut self.children, Node::Doctype(doctype));
	}

	fn parse_processing_instruction(&mut self, target_span: SourceSpan) {
		let target = self.text_for_span(target_span);
		let open_span = Self::markup_open_span(target_span);
		self.clear_attribute_state();

		loop {
			let Some(token) = self.lexer.next() else {
				let (attributes, _) = self.take_attributes();
				self.errors.push(ParseError { span: open_span, kind: ParseErrorKind::UnterminatedProcessingInstruction });
				push_node(
					self.stack.as_mut_slice(),
					&mut self.children,
					Node::ProcessingInstruction(build_processing_instruction(target, target_span, attributes, open_span)),
				);
				return;
			};

			match token.kind {
				lexer::TokenKind::AttrName => {
					self.push_pending_attribute_without_value();
					self.pending_attribute = Some(PendingAttribute {
						key: self.text_for_span(token.span),
						key_span: token.span,
					});
				}
				lexer::TokenKind::AttrValue => self.push_pending_attribute_with_value(token.span),
				lexer::TokenKind::PIClose => {
					let (attributes, _) = self.take_attributes();
					push_node(
						self.stack.as_mut_slice(),
						&mut self.children,
						Node::ProcessingInstruction(build_processing_instruction(
							target,
							target_span,
							attributes,
							combined_span(open_span, token.span),
						)),
					);
					return;
				}
				lexer::TokenKind::TagClose => {
					let (attributes, _) = self.take_attributes();
					self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnterminatedProcessingInstruction });
					push_node(
						self.stack.as_mut_slice(),
						&mut self.children,
						Node::ProcessingInstruction(build_processing_instruction(
							target,
							target_span,
							attributes,
							combined_span(open_span, token.span),
						)),
					);
					return;
				}
				lexer::TokenKind::Error => {
					self.push_pending_attribute_without_value();
					self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::InvalidAttribute });
				}
				_ => {
					self.push_pending_attribute_without_value();
					self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken });
				}
			}
		}
	}

	fn parse_open_tag(&mut self, tag_span: SourceSpan) {
		let tag = self.text_for_span(tag_span);
		let open_span = Self::tag_open_span(tag_span);
		let Some(tag_end) = self.parse_tag_attributes(true, ParseErrorKind::UnterminatedTag) else {
			return;
		};

		let (attributes, id) = self.take_attributes();
		let element_span = match tag_end {
			TagEnd::Close(close_span) | TagEnd::SelfClose(close_span) => combined_span(open_span, close_span),
		};
		let element = build_element(id, tag, tag_span, attributes, element_span);

		match tag_end {
			TagEnd::Close(_) if matches!(element.kind, ElementKind::Void) => {
				push_node(self.stack.as_mut_slice(), &mut self.children, Node::Element(element));
			}
			TagEnd::Close(_) => {
				let raw_text_tag = element.kind.is_raw_text().then_some(element.tag);
				self.stack.push(OpenElement { open_span: element_span, element });
				if let Some(raw_text_tag) = raw_text_tag {
					let span = self.lexer.raw_text(raw_text_tag.as_bytes());
					self.push_text(span);
				}
			}
			TagEnd::SelfClose(_) => {
				if matches!(element.kind, ElementKind::RawText) {
					self.errors.push(ParseError { span: element_span, kind: ParseErrorKind::SelfClosingRawTextElement });
				}

				push_node(self.stack.as_mut_slice(), &mut self.children, Node::Element(element));
			}
		}
	}

	fn parse_close_tag(&mut self, tag_span: SourceSpan) {
		let tag = self.text_for_span(tag_span);
		let open_span = Self::end_tag_open_span(tag_span);
		let Some(tag_end) = self.parse_tag_attributes(false, ParseErrorKind::UnterminatedTag) else {
			return;
		};

		let close_span = match tag_end {
			TagEnd::Close(close_span) | TagEnd::SelfClose(close_span) => combined_span(open_span, close_span),
		};
		let _ = self.take_attributes();

		if !recover_to_matching_close(&mut self.stack, &mut self.children, &mut self.errors, tag, close_span) {
			self.errors.push(ParseError { span: close_span, kind: ParseErrorKind::UnexpectedToken });
		}
	}

	fn parse(mut self) -> Document<'a> {
		while let Some(token) = self.lexer.next() {
			match token.kind {
				lexer::TokenKind::Text => self.push_text(token.span),
				lexer::TokenKind::Comment => self.parse_comment(token.span),
				lexer::TokenKind::CData => self.parse_cdata(token.span),
				lexer::TokenKind::DocTypeOpen => self.parse_doctype(token.span),
				lexer::TokenKind::PIOpen => self.parse_processing_instruction(token.span),
				lexer::TokenKind::TagOpen => self.parse_open_tag(token.span),
				lexer::TokenKind::EndTagOpen => self.parse_close_tag(token.span),
				lexer::TokenKind::Error => self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken }),
				_ => self.errors.push(ParseError { span: token.span, kind: ParseErrorKind::UnexpectedToken }),
			}
		}

		while let Some(open) = self.stack.last() {
			self.errors.push(ParseError { span: open.open_span, kind: ParseErrorKind::UnclosedElement });
			finish_open_element(&mut self.stack, &mut self.children);
		}

		Document { children: self.children, errors: self.errors }
	}
}

pub fn parse<'a>(input: &'a str) -> Document<'a> {
	Parser::new(input).parse()
}
