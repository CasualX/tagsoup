use super::*;

struct OpenElement<'a> {
	element: Element<'a>,
	open_span: Span,
}

fn node_span(node: &Node<'_>) -> Span {
	match node {
		Node::Text(text) => text.span,
		Node::Element(element) => element.span,
		Node::Comment(comment) => comment.span,
		Node::Doctype(doctype) => doctype.span,
		Node::ProcessingInstruction(pi) => pi.span,
	}
}

fn build_doctype<'a>(flat: &mut FlatElement<'a>) -> DoctypeNode<'a> {
	DoctypeNode {
		name: flat.tag,
		name_span: flat.tag_span,
		attributes: mem::take(&mut flat.attributes),
		span: flat.span,
	}
}

fn build_processing_instruction<'a>(flat: &mut FlatElement<'a>) -> ProcessingInstructionNode<'a> {
	ProcessingInstructionNode {
		target: flat.tag,
		target_span: flat.tag_span,
		data: mem::take(&mut flat.attributes),
		span: flat.span,
	}
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

fn build_element<'a>(flat: &mut FlatElement<'a>) -> Element<'a> {
	Element {
		id: flat.id,
		tag: flat.tag,
		tag_span: flat.tag_span,
		kind: flat.element_kind,
		attributes: mem::take(&mut flat.attributes),
		children: Vec::new(),
		span: flat.span,
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
	close_tag: &FlatElement<'a>,
) -> bool {
	let Some(match_index) = stack.iter().rposition(|open| open.element.tag == close_tag.tag) else {
		return false;
	};

	while stack.len() - 1 > match_index {
		if let Some(open) = stack.last() {
			errors.push(ParseError { span: open.open_span, kind: ParseErrorKind::UnclosedElement });
		}
		finish_open_element(stack, children);
	}

	if let Some(open) = stack.last_mut() {
		open.element.span = combined_span(open.element.span, close_tag.span);
	}
	finish_open_element(stack, children);
	true
}

fn parse_nodes<'a>(tokens: &mut [FlatToken<'a>], children: &mut Vec<Node<'a>>, errors: &mut Vec<ParseError>) {
	let mut stack = Vec::new();

	for token in tokens {
		match token {
			FlatToken::Text(text) => {
				push_node(
					stack.as_mut_slice(),
					children,
					Node::Text(TextNode { text: text.text, span: text.span }),
				);
			}
			FlatToken::Comment(comment) => {
				push_node(
					stack.as_mut_slice(),
					children,
					Node::Comment(CommentNode { comment: comment.comment, span: comment.span }),
				);
			}
			FlatToken::CData(cdata) => {
				push_node(
					stack.as_mut_slice(),
					children,
					Node::Text(TextNode { text: cdata.data, span: cdata.span }),
				);
			}
			FlatToken::Element(flat) => match flat.tag_marker {
				TagMarker::ElementOpen => {
					let element = build_element(flat);
					if matches!(element.kind, ElementKind::Void) {
						push_node(stack.as_mut_slice(), children, Node::Element(element));
					}
					else {
						stack.push(OpenElement { open_span: flat.span, element });
					}
				}
				TagMarker::SelfClosing => {
					let element = build_element(flat);
					if matches!(element.kind, ElementKind::RawText) {
						errors.push(ParseError { span: flat.span, kind: ParseErrorKind::SelfClosingRawTextElement });
					}

					push_node(stack.as_mut_slice(), children, Node::Element(element));
				}
				TagMarker::ElementClose => {
					if !recover_to_matching_close(&mut stack, children, errors, flat) {
						errors.push(ParseError { span: flat.span, kind: ParseErrorKind::UnexpectedToken });
					}
				}
				TagMarker::Doctype => {
					push_node(stack.as_mut_slice(), children, Node::Doctype(build_doctype(flat)));
				}
				TagMarker::ProcessingInstruction => {
					push_node(
						stack.as_mut_slice(),
						children,
						Node::ProcessingInstruction(build_processing_instruction(flat)),
					);
				}
			},
		}
	}

	while let Some(open) = stack.last() {
		errors.push(ParseError { span: open.open_span, kind: ParseErrorKind::UnclosedElement });
		finish_open_element(&mut stack, children);
	}
}

pub fn parse<'a>(input: &'a str) -> Document<'a> {
	let mut flat = parse_flat(input);
	let mut children = Vec::new();
	let mut errors = flat.errors;
	parse_nodes(&mut flat.tokens, &mut children, &mut errors);
	Document { children, errors }
}
