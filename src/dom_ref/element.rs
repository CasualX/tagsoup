use super::*;

/// Element in the DOM tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Element<'a> {
	/// The id of the element.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
	pub id: Option<&'a str>,

	/// The tag of the element.
	pub tag: &'a str,

	/// Span of the element tag in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub tag_span: SourceSpan,

	/// The element kind.
	pub kind: ElementKind,

	/// All parsed attributes in source order.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
	pub attributes: Vec<Attribute<'a>>,

	/// All of the element's child nodes.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
	pub children: Vec<Node<'a>>,

	/// Span of the element in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub span: SourceSpan,
}

impl<'a> Element<'a> {
	/// Gets the attribute with the given key, if it exists.
	pub fn get_attribute(&self, key: &str) -> Option<&Attribute<'a>> {
		self.attributes.iter().find(|attr| attr.key == key)
	}

	/// Gets the value of the attribute with the given key, if it exists.
	pub fn get_attribute_value(&self, key: &str) -> Option<Cow<'a, str>> {
		self.get_attribute(key)?.value.as_ref().map(|value| value.value())
	}

	/// Gets the text content of the element and all of its children.
	///
	/// This decodes HTML entities (except `script` and `style` elements) but does not normalize whitespace.
	///
	/// Optionally run the output through [`normalize_whitespace`] to collapse runs of ASCII whitespace into a single space.
	#[inline]
	pub fn text_content(&self) -> String {
		let mut text = String::new();
		self.text_content_into(&mut text, matches!(self.kind, ElementKind::RawText));
		text
	}

	fn text_content_into(&self, text: &mut String, is_raw_text: bool) {
		for child in &self.children {
			match child {
				Node::Text(text_node) if is_raw_text => text.push_str(text_node.text),
				Node::Text(text_node) => entity::push_decoded_entities(text, text_node.text),
				Node::Element(element) => element.text_content_into(text, matches!(element.kind, ElementKind::RawText)),
				Node::Comment(_) => {}
				Node::Doctype(_) => {}
				Node::ProcessingInstruction(_) => {}
			}
		}
	}
}

impl<'a> Element<'a> {
	/// Visits all elements in the DOM tree depth-first.
	///
	/// The visitor function is called for each element in the tree.
	pub fn visit<'dom>(&'dom self, mut visitor: impl FnMut(&[&'dom Element<'a>], &'dom Element<'a>) -> VisitControl) {
		let mut parents = Vec::new();
		visit(&self.children, &mut parents, &mut visitor);
	}

	/// Queries the element for the first element matching the given CSS selector.
	///
	/// Selector queries do not descend into `<template>` contents.
	///
	/// Panics if the selector is invalid.
	pub fn query_selector(&self, selector: &str) -> Option<&Element<'a>> {
		let steps = selector::parser::Parser::parse(selector).expect(selector);
		let mut result = None;
		dfs::query(&self.children, &steps, &mut |element| {
			result = Some(element);
			false
		});
		result
	}

	/// Queries the element for all elements matching the given CSS selector.
	///
	/// Selector queries do not descend into `<template>` contents.
	///
	/// Panics if the selector is invalid.
	pub fn query_selector_all(&self, selector: &str) -> Vec<&Element<'a>> {
		let steps = selector::parser::Parser::parse(selector).expect(selector);
		let mut result = Vec::new();
		dfs::query(&self.children, &steps, &mut |element| {
			result.push(element);
			true
		});
		result
	}
}
