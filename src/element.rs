
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
