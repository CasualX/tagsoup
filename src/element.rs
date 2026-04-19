use super::*;

/// Kind of element.
///
/// Determines how the element's content is parsed and whether it can have children.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum ElementKind {
	/// An element that cannot have any children and must not have an end tag.
	///
	/// Examples include `<br>`, `<img>`, and `<input>`.
	Void,
	/// An element that can have children but whose content is treated as unprocessed.
	///
	/// The `<template>` element.
	///
	/// Template contents are still parsed and preserved in the tree, but selector queries do not descend into them.
	Template,
	/// An element that contains raw text that should not be parsed as HTML and should not decode entities.
	///
	/// The `<script>` and `<style>` elements.
	RawText,
	/// An element that contains raw text that should not be parsed as HTML but should decode entities.
	///
	/// The `<textarea>` and `<title>` elements.
	EscapableRawText,
	/// A normal element that can have children and must be closed by a matching end tag.
	Normal,
}

#[inline(always)]
const fn to_lower_tag8(tag: &str) -> u64 {
	let mut lower_tag = [0u8; 8];
	let bytes = tag.as_bytes();
	let len = if bytes.len() > 8 { 8 } else { bytes.len() };
	let mut i = 0;
	while i < len {
		lower_tag[i] = bytes[i].to_ascii_lowercase();
		i += 1;
	}
	u64::from_le_bytes(lower_tag)
}

impl ElementKind {
	pub(crate) fn from_tag(tag: &str) -> ElementKind {
		if tag.len() > 8 {
			return ElementKind::Normal;
		}
		const AREA: u64 = to_lower_tag8("area");
		const BASE: u64 = to_lower_tag8("base");
		const BR: u64 = to_lower_tag8("br");
		const COL: u64 = to_lower_tag8("col");
		const EMBED: u64 = to_lower_tag8("embed");
		const HR: u64 = to_lower_tag8("hr");
		const IMG: u64 = to_lower_tag8("img");
		const INPUT: u64 = to_lower_tag8("input");
		const LINK: u64 = to_lower_tag8("link");
		const META: u64 = to_lower_tag8("meta");
		const PARAM: u64 = to_lower_tag8("param");
		const SOURCE: u64 = to_lower_tag8("source");
		const TRACK: u64 = to_lower_tag8("track");
		const WBR: u64 = to_lower_tag8("wbr");
		const TEMPLATE: u64 = to_lower_tag8("template");
		const SCRIPT: u64 = to_lower_tag8("script");
		const STYLE: u64 = to_lower_tag8("style");
		const TEXTAREA: u64 = to_lower_tag8("textarea");
		const TITLE: u64 = to_lower_tag8("title");
		match to_lower_tag8(tag) {
			AREA | BASE | BR | COL | EMBED | HR | IMG | INPUT | LINK | META | PARAM | SOURCE | TRACK | WBR => ElementKind::Void,
			TEMPLATE => ElementKind::Template,
			SCRIPT | STYLE => ElementKind::RawText,
			TEXTAREA | TITLE => ElementKind::EscapableRawText,
			_ => ElementKind::Normal,
		}
	}
	/// Returns true if the element kind is a raw text element.
	#[inline]
	pub fn is_raw_text(&self) -> bool {
		matches!(self, ElementKind::RawText | ElementKind::EscapableRawText)
	}
}

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
	pub tag_span: Span,

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
	pub span: Span,
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
	pub fn visit(&self, visitor: &mut dyn FnMut(&Element<'a>) -> VisitControl) -> VisitControl {
		match visitor(self) {
			VisitControl::Descend => {
				for child in &self.children {
					if let Node::Element(element) = child {
						if element.visit(visitor) == VisitControl::Stop {
							return VisitControl::Stop;
						}
					}
				}
				VisitControl::Continue
			}
			VisitControl::Continue => VisitControl::Continue,
			VisitControl::Stop => VisitControl::Stop,
		}
	}

	/// Queries the element for the first element matching the given CSS selector.
	///
	/// Selector queries do not descend into `<template>` contents.
	///
	/// Panics if the selector is invalid.
	pub fn query_selector(&self, selector: &str) -> Option<&Element<'a>> {
		let steps = selector::parser::Parser::parse(selector).expect(selector);
		let mut result = None;
		selector::dfs::query(&self.children, &steps, &mut |element| {
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
		selector::dfs::query(&self.children, &steps, &mut |element| {
			result.push(element);
			true
		});
		result
	}
}
