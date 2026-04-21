use super::super::*;

#[test]
fn parses_flat_structure_with_attributes() {
	let doc = parse_flat("hello <a href=\"x\" checked>world</a> tail");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.tokens.len(), 5);

	let FlatToken::Text(prefix) = &doc.tokens[0] else {
		panic!("expected first token to be text");
	};
	assert_eq!(prefix.text, "hello ");

	let FlatToken::Element(open) = &doc.tokens[1] else {
		panic!("expected second token to be an element");
	};
	assert_eq!(open.tag_marker, TagMarker::ElementOpen);
	assert_eq!(open.id, None);
	assert_eq!(open.tag, "a");
	assert_eq!(open.attributes.len(), 2);
	assert_eq!(open.attributes[0].key, "href");
	assert_eq!(open.attributes[0].value.as_ref().map(|v| v.value), Some("x"));
	assert_eq!(open.attributes[1].key, "checked");
	assert_eq!(open.attributes[1].value, None);

	let FlatToken::Text(inner_text) = &doc.tokens[2] else {
		panic!("expected third token to be text");
	};
	assert_eq!(inner_text.text, "world");

	let FlatToken::Element(close) = &doc.tokens[3] else {
		panic!("expected fourth token to be an element");
	};
	assert_eq!(close.tag_marker, TagMarker::ElementClose);
	assert_eq!(close.tag, "a");

	let FlatToken::Text(suffix) = &doc.tokens[4] else {
		panic!("expected fifth token to be text");
	};
	assert_eq!(suffix.text, " tail");
}

#[test]
fn rejects_self_closing_end_tag_but_keeps_going() {
	let doc = parse_flat("</sandwhich/><x/>");
	assert_eq!(doc.tokens.len(), 2);
	assert!(doc.errors.iter().any(|error| error.kind == ParseErrorKind::SelfClosingEndTag));

	let FlatToken::Element(close) = &doc.tokens[0] else {
		panic!("expected first token to be a closing element");
	};
	assert_eq!(close.tag_marker, TagMarker::ElementClose);
	assert_eq!(close.tag, "sandwhich");

	let FlatToken::Element(self_closing) = &doc.tokens[1] else {
		panic!("expected second token to be an element");
	};
	assert_eq!(self_closing.tag_marker, TagMarker::SelfClosing);
	assert_eq!(self_closing.tag, "x");
}

#[test]
fn recovers_processing_instruction_with_gt_close() {
	let doc = parse_flat("<?pi \0>tail<x>");
	assert_eq!(doc.tokens.len(), 3);
	assert!(doc.errors.iter().any(|error| error.kind == ParseErrorKind::InvalidAttribute));
	assert!(doc.errors.iter().any(|error| error.kind == ParseErrorKind::UnterminatedProcessingInstruction));

	let FlatToken::Element(pi) = &doc.tokens[0] else {
		panic!("expected first token to be PI element");
	};
	assert_eq!(pi.tag_marker, TagMarker::ProcessingInstruction);
	assert_eq!(pi.tag, "pi");

	let FlatToken::Text(text) = &doc.tokens[1] else {
		panic!("expected second token to be text");
	};
	assert_eq!(text.text, "tail");

	let FlatToken::Element(element) = &doc.tokens[2] else {
		panic!("expected third token to be normal element");
	};
	assert_eq!(element.tag_marker, TagMarker::ElementOpen);
	assert_eq!(element.tag, "x");
}

#[test]
fn parses_comments_doctype_and_cdata() {
	let doc = parse_flat("<!DOCTYPE html><!-- ok --><![CDATA[x<y]]>");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.tokens.len(), 3);

	let FlatToken::Element(doctype) = &doc.tokens[0] else {
		panic!("expected doctype as first token");
	};
	assert_eq!(doctype.tag_marker, TagMarker::Doctype);
	assert_eq!(doctype.tag, "DOCTYPE");
	assert_eq!(doctype.attributes.len(), 1);
	assert_eq!(doctype.attributes[0].key, "html");

	let FlatToken::Comment(comment) = &doc.tokens[1] else {
		panic!("expected comment as second token");
	};
	assert_eq!(comment.comment, " ok ");

	let FlatToken::CData(cdata) = &doc.tokens[2] else {
		panic!("expected cdata as third token");
	};
	assert_eq!(cdata.data, "x<y");
}

#[test]
fn parses_unquoted_attribute_value() {
	let doc = parse_flat("<div class=note>");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.tokens.len(), 1);

	let FlatToken::Element(element) = &doc.tokens[0] else {
		panic!("expected element token");
	};
	assert_eq!(element.tag_marker, TagMarker::ElementOpen);
	assert_eq!(element.tag, "div");
	assert_eq!(element.attributes.len(), 1);
	assert_eq!(element.attributes[0].key, "class");
	assert_eq!(element.attributes[0].value.as_ref().map(|v| v.value), Some("note"));
}

#[test]
fn captures_first_id_attribute_value() {
	let doc = parse_flat("<div class=note id=main id=override></div>");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.tokens.len(), 2);

	let FlatToken::Element(open) = &doc.tokens[0] else {
		panic!("expected opening element token");
	};
	assert_eq!(open.id, Some("main"));

	let FlatToken::Element(close) = &doc.tokens[1] else {
		panic!("expected closing element token");
	};
	assert_eq!(close.id, None);
}

#[test]
fn treats_script_contents_as_raw_text() {
	let doc = parse_flat(r#"<script>if (n <= 1) { document.write(["<", "/script>"].join("")); }</script>"#);
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.tokens.len(), 3);

	let FlatToken::Element(open) = &doc.tokens[0] else {
		panic!("expected opening script element");
	};
	assert_eq!(open.tag_marker, TagMarker::ElementOpen);
	assert_eq!(open.tag, "script");

	let FlatToken::Text(text) = &doc.tokens[1] else {
		panic!("expected script contents as text");
	};
	assert_eq!(text.text, "if (n <= 1) { document.write([\"<\", \"/script>\"].join(\"\")); }");

	let FlatToken::Element(close) = &doc.tokens[2] else {
		panic!("expected closing script element");
	};
	assert_eq!(close.tag_marker, TagMarker::ElementClose);
	assert_eq!(close.tag, "script");
}

#[test]
fn does_not_treat_self_closing_raw_text_tags_as_open_raw_text() {
	let doc = parse_flat("<div><script/><style/></div>");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.tokens.len(), 4);

	let FlatToken::Element(div) = &doc.tokens[0] else {
		panic!("expected opening div element");
	};
	assert_eq!(div.tag_marker, TagMarker::ElementOpen);
	assert_eq!(div.tag, "div");

	let FlatToken::Element(script) = &doc.tokens[1] else {
		panic!("expected self-closing script element");
	};
	assert_eq!(script.tag_marker, TagMarker::SelfClosing);
	assert_eq!(script.tag, "script");

	let FlatToken::Element(style) = &doc.tokens[2] else {
		panic!("expected self-closing style element");
	};
	assert_eq!(style.tag_marker, TagMarker::SelfClosing);
	assert_eq!(style.tag, "style");

	let FlatToken::Element(close) = &doc.tokens[3] else {
		panic!("expected closing div element");
	};
	assert_eq!(close.tag_marker, TagMarker::ElementClose);
	assert_eq!(close.tag, "div");
}

#[test]
fn preserves_whitespace_only_text_tokens() {
	let doc = parse_flat(" \n\t<div>\n\t</div> ");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.tokens.len(), 5);

	let FlatToken::Text(prefix) = &doc.tokens[0] else {
		panic!("expected leading text token");
	};
	assert_eq!(prefix.text, " \n\t");

	let FlatToken::Element(open) = &doc.tokens[1] else {
		panic!("expected opening div element");
	};
	assert_eq!(open.tag_marker, TagMarker::ElementOpen);
	assert_eq!(open.tag, "div");

	let FlatToken::Element(close) = &doc.tokens[3] else {
		panic!("expected closing div element");
	};
	assert_eq!(close.tag_marker, TagMarker::ElementClose);
	assert_eq!(close.tag, "div");
}

#[test]
fn preserves_text_after_empty_angle_brackets() {
	let doc = parse_flat("hello <> world");
	assert!(doc.errors.iter().any(|error| error.kind == ParseErrorKind::MissingTagName));
	assert_eq!(
		doc.tokens,
		vec![FlatToken::Text(FlatText { text: "hello <> world", span: SourceSpan::new(0, 14) })],
	);
}
