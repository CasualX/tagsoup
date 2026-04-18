use super::*;

#[test]
fn builds_nested_tree() {
	let doc = Document::parse("hello <div id=\"root\"><span>world</span><!-- ok --></div>!");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.children.len(), 3);

	let Node::Text(prefix) = &doc.children[0] else {
		panic!("expected leading text node");
	};
	assert_eq!(prefix.text, "hello ");

	let Node::Element(div) = &doc.children[1] else {
		panic!("expected div element");
	};
	assert_eq!(div.tag, "div");
	assert_eq!(div.id, Some("root"));
	assert_eq!(div.children.len(), 2);

	let Node::Element(span) = &div.children[0] else {
		panic!("expected nested span element");
	};
	assert_eq!(span.tag, "span");
	assert_eq!(span.children.len(), 1);
	assert_eq!(span.children[0].text().map(|text| text.text), Some("world"));

	let Node::Comment(comment) = &div.children[1] else {
		panic!("expected trailing comment");
	};
	assert_eq!(comment.comment, " ok ");

	let Node::Text(suffix) = &doc.children[2] else {
		panic!("expected trailing text node");
	};
	assert_eq!(suffix.text, "!");
}

#[test]
fn forwards_id_from_flat_element() {
	let doc = Document::parse("<div class=outer id=main><span id=inner></span></div>");
	assert_eq!(doc.errors, vec![]);

	let Some(div) = doc.children[0].element() else {
		panic!("expected root div element");
	};
	assert_eq!(div.id, Some("main"));

	let Some(span) = div.children[0].element() else {
		panic!("expected nested span element");
	};
	assert_eq!(span.id, Some("inner"));
}

#[test]
fn keeps_void_elements_as_leaf_nodes() {
	let doc = Document::parse("<div><br>tail</div>");
	assert_eq!(doc.errors, vec![]);

	let Some(div) = doc.children[0].element() else {
		panic!("expected root div element");
	};
	assert_eq!(div.children.len(), 2);
	assert_eq!(div.children[0].element().map(|element| element.tag), Some("br"));
	assert_eq!(div.children[1].text().map(|text| text.text), Some("tail"));
}

#[test]
fn reports_self_closing_raw_text_elements() {
	let doc = Document::parse("<div><script/><style/></div>");
	assert_eq!(doc.errors.len(), 2);
	assert!(doc.errors.iter().all(|error| error.kind == ParseErrorKind::SelfClosingRawTextElement));

	let Some(div) = doc.children[0].element() else {
		panic!("expected root div element");
	};
	assert_eq!(div.children.len(), 2);
	assert_eq!(div.children[0].element().map(|element| element.tag), Some("script"));
	assert_eq!(div.children[1].element().map(|element| element.tag), Some("style"));
}

#[test]
fn repairs_stack_when_closing_tag_matches_ancestor() {
	let doc = Document::parse("<div><p><span></div>");
	assert_eq!(
		doc.errors,
		vec![
			ParseError { span: Span::new(8, 14), kind: ParseErrorKind::UnclosedElement },
			ParseError { span: Span::new(5, 8), kind: ParseErrorKind::UnclosedElement },
		],
	);

	let Some(div) = doc.children[0].element() else {
		panic!("expected root div element");
	};
	assert_eq!(div.tag, "div");
	assert_eq!(div.span, Span::new(0, 20));
	assert_eq!(div.children.len(), 1);

	let Some(paragraph) = div.children[0].element() else {
		panic!("expected nested paragraph element");
	};
	assert_eq!(paragraph.tag, "p");
	assert_eq!(paragraph.span, Span::new(5, 14));
	assert_eq!(paragraph.children.len(), 1);

	let Some(span) = paragraph.children[0].element() else {
		panic!("expected nested span element");
	};
	assert_eq!(span.tag, "span");
	assert_eq!(span.span, Span::new(8, 14));
}

#[test]
fn reports_unmatched_closing_tag_when_no_ancestor_matches() {
	let doc = Document::parse("<div><span></p>");
	assert_eq!(doc.errors.len(), 3);
	assert!(doc.errors.iter().any(|error| error.kind == ParseErrorKind::UnexpectedToken));
	assert_eq!(
		doc.errors.iter().filter(|error| error.kind == ParseErrorKind::UnclosedElement).count(),
		2,
	);

	let Some(div) = doc.children[0].element() else {
		panic!("expected root div element");
	};
	assert_eq!(div.children.len(), 1);
	assert_eq!(div.children[0].element().map(|element| element.tag), Some("span"));
	assert_eq!(div.span, Span::new(0, 11));
	assert_eq!(div.children[0].element().map(|element| element.span), Some(Span::new(5, 11)));
}

#[test]
fn folds_cdata_into_text_nodes() {
	let doc = Document::parse("<![CDATA[x<y]]>");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.children.len(), 1);
	assert_eq!(doc.children[0].text().map(|text| text.text), Some("x<y"));
}

#[test]
fn keeps_doctypes_and_processing_instructions_in_document_order() {
	let doc = Document::parse("<?xml version=\"1.0\"?><!DOCTYPE html><div></div>");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.children.len(), 3);

	let Node::ProcessingInstruction(pi) = &doc.children[0] else {
		panic!("expected leading processing instruction");
	};
	assert_eq!(pi.target, "xml");
	assert_eq!(pi.data.len(), 1);
	assert_eq!(pi.data[0].key, "version");
	assert_eq!(pi.data[0].value.as_ref().map(|value| value.value), Some("1.0"));

	let Node::Doctype(doctype) = &doc.children[1] else {
		panic!("expected doctype after processing instruction");
	};
	assert_eq!(doctype.name, "DOCTYPE");
	assert_eq!(doctype.attributes.len(), 1);
	assert_eq!(doctype.attributes[0].key, "html");

	let Some(div) = doc.children[2].element() else {
		panic!("expected trailing div element");
	};
	assert_eq!(div.tag, "div");
}

#[test]
fn keeps_non_element_markup_inside_open_elements() {
	let doc = Document::parse("<root><?pi mode=\"test\"?><!DOCTYPE html><child/></root>");
	assert_eq!(doc.errors, vec![]);

	let Some(root) = doc.children[0].element() else {
		panic!("expected root element");
	};
	assert_eq!(root.children.len(), 3);

	let Node::ProcessingInstruction(pi) = &root.children[0] else {
		panic!("expected nested processing instruction");
	};
	assert_eq!(pi.target, "pi");
	assert_eq!(pi.data[0].key, "mode");
	assert_eq!(pi.data[0].value.as_ref().map(|value| value.value), Some("test"));

	let Node::Doctype(doctype) = &root.children[1] else {
		panic!("expected nested doctype");
	};
	assert_eq!(doctype.name, "DOCTYPE");
	assert_eq!(doctype.attributes[0].key, "html");

	assert_eq!(root.children[2].element().map(|element| element.tag), Some("child"));
	assert_eq!(root.span, Span::new(0, 54));
}

#[test]
fn keeps_script_contents_out_of_html_parsing() {
	let doc = Document::parse(r#"<script>if (n <= 1) { document.write(["<", "/script>"].join("")); }</script><div>ok</div>"#);
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.children.len(), 2);

	let Some(script) = doc.children[0].element() else {
		panic!("expected script element");
	};
	assert_eq!(script.tag, "script");
	assert_eq!(script.children.len(), 1);
	assert_eq!(script.children[0].text().map(|text| text.text), Some("if (n <= 1) { document.write([\"<\", \"/script>\"].join(\"\")); }"));

	assert_eq!(doc.children[1].element().map(|element| element.tag), Some("div"));
}

#[test]
fn preserves_whitespace_only_text_nodes() {
	let doc = Document::parse(" \n\t<div>\n\t<span>ok</span>\n\t</div> \n");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.children.len(), 3);
	assert_eq!(doc.children[0].text().map(|text| text.text), Some(" \n\t"));
	assert_eq!(doc.children[2].text().map(|text| text.text), Some(" \n"));

	let Some(div) = doc.children[1].element() else {
		panic!("expected root div element");
	};
	assert_eq!(div.tag, "div");
	assert_eq!(div.children.len(), 3);
	assert_eq!(div.children[0].text().map(|text| text.text), Some("\n\t"));
	assert_eq!(div.children[1].element().map(|element| element.tag), Some("span"));
	assert_eq!(div.children[1].element().and_then(|element| element.children[0].text()).map(|text| text.text), Some("ok"));
	assert_eq!(div.children[2].text().map(|text| text.text), Some("\n\t"));
}

#[test]
fn keeps_textarea_contents_out_of_html_parsing() {
	let doc = Document::parse("<textarea><b>not markup</b></textarea><div>ok</div>");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.children.len(), 2);

	let Some(textarea) = doc.children[0].element() else {
		panic!("expected textarea element");
	};
	assert_eq!(textarea.tag, "textarea");
	assert_eq!(textarea.children.len(), 1);
	assert_eq!(textarea.children[0].text().map(|text| text.text), Some("<b>not markup</b>"));

	assert_eq!(doc.children[1].element().map(|element| element.tag), Some("div"));
}

#[test]
fn currently_treats_mixed_case_raw_text_end_tags_as_unmatched() {
	let doc = Document::parse("<script>ok</SCRIPT/>tail");
	assert_eq!(
		doc.errors,
		vec![
			ParseError { span: Span::new(18, 20), kind: ParseErrorKind::SelfClosingEndTag },
			ParseError { span: Span::new(10, 20), kind: ParseErrorKind::UnexpectedToken },
			ParseError { span: Span::new(0, 8), kind: ParseErrorKind::UnclosedElement },
		],
	);

	let Some(script) = doc.children[0].element() else {
		panic!("expected script element");
	};
	assert_eq!(script.children.len(), 2);
	assert_eq!(script.children[0].text().map(|text| text.text), Some("ok"));
	assert_eq!(script.children[1].text().map(|text| text.text), Some("tail"));
}
