use super::super::*;

#[test]
fn builds_nested_tree() {
	let doc = Document::parse("hello <div id=\"root\"><span>world</span><!-- ok --></div>!");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.children.len(), 3);

	let prefix = &doc.children[0].text().unwrap();
	assert_eq!(prefix.text, "hello ");

	let div = doc.children[1].element().unwrap();
	assert_eq!(div.tag, "div");
	assert_eq!(div.id, Some("root"));
	assert_eq!(div.children.len(), 2);

	let span = div.children[0].element().unwrap();
	assert_eq!(span.tag, "span");
	assert_eq!(span.children.len(), 1);
	assert_eq!(span.children[0].text().map(|text| text.text), Some("world"));

	let comment = div.children[1].comment().unwrap();
	assert_eq!(comment.comment, " ok ");

	let suffix = doc.children[2].text().unwrap();
	assert_eq!(suffix.text, "!");
}

#[test]
fn forwards_id_from_flat_element() {
	let doc = Document::parse("<div class=outer id=main><span id=inner></span></div>");
	assert_eq!(doc.errors, vec![]);

	let div = doc.children[0].element().unwrap();
	assert_eq!(div.id, Some("main"));

	let span = div.children[0].element().unwrap();
	assert_eq!(span.id, Some("inner"));
}

#[test]
fn keeps_void_elements_as_leaf_nodes() {
	let doc = Document::parse("<div><br>tail</div>");
	assert_eq!(doc.errors, vec![]);

	let div = doc.children[0].element().unwrap();
	assert_eq!(div.children.len(), 2);
	assert_eq!(div.children[0].element().map(|element| element.tag), Some("br"));
	assert_eq!(div.children[1].text().map(|text| text.text), Some("tail"));
}

#[test]
fn reports_self_closing_raw_text_elements() {
	let doc = Document::parse("<div><script/><style/></div>");
	assert_eq!(doc.errors.len(), 2);
	assert!(doc.errors.iter().all(|error| error.kind == ParseErrorKind::SelfClosingRawTextElement));

	let div = doc.children[0].element().unwrap();
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
			ParseError { span: SourceSpan::new(8, 14), kind: ParseErrorKind::UnclosedElement },
			ParseError { span: SourceSpan::new(5, 8), kind: ParseErrorKind::UnclosedElement },
		],
	);

	let div = doc.children[0].element().unwrap();
	assert_eq!(div.tag, "div");
	assert_eq!(div.span, SourceSpan::new(0, 20));
	assert_eq!(div.children.len(), 1);

	let paragraph = div.children[0].element().unwrap();
	assert_eq!(paragraph.tag, "p");
	assert_eq!(paragraph.span, SourceSpan::new(5, 14));
	assert_eq!(paragraph.children.len(), 1);

	let span = paragraph.children[0].element().unwrap();
	assert_eq!(span.tag, "span");
	assert_eq!(span.span, SourceSpan::new(8, 14));
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

	let div = doc.children[0].element().unwrap();
	assert_eq!(div.children.len(), 1);
	assert_eq!(div.children[0].element().map(|element| element.tag), Some("span"));
	assert_eq!(div.span, SourceSpan::new(0, 11));
	assert_eq!(div.children[0].element().map(|element| element.span), Some(SourceSpan::new(5, 11)));
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

	let pi = doc.children[0].processing_instruction().unwrap();
	assert_eq!(pi.target, "xml");
	assert_eq!(pi.data.len(), 1);
	assert_eq!(pi.data[0].key, "version");
	assert_eq!(pi.data[0].value.as_ref().map(|value| value.value_raw()), Some("1.0"));

	let doctype = doc.children[1].doctype().unwrap();
	assert_eq!(doctype.keyword, "DOCTYPE");
	assert_eq!(doctype.args.len(), 1);
	assert_eq!(doctype.args[0].value_raw(), "html");

	let div = doc.children[2].element().unwrap();
	assert_eq!(div.tag, "div");
}

#[test]
fn nests_internal_subset_doctypes_under_the_parent_doctype() {
	let doc = Document::parse("<!DOCTYPE html [<!ELEMENT test ANY><!ATTLIST test id ID #REQUIRED>]>");
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.children.len(), 1);

	let doctype = doc.children[0].doctype().unwrap();
	assert_eq!(doctype.keyword, "DOCTYPE");
	assert_eq!(doctype.args.len(), 1);
	assert_eq!(doctype.args[0].value_raw(), "html");
	assert_eq!(doctype.dtd.len(), 2);
	assert_eq!(doctype.span, SourceSpan::new(0, 68));

	let element = &doctype.dtd[0];
	assert_eq!(element.keyword, "ELEMENT");
	assert_eq!(element.args.len(), 2);
	assert_eq!(element.args[0].value_raw(), "test");
	assert_eq!(element.args[1].value_raw(), "ANY");
	assert_eq!(element.dtd.len(), 0);

	let attlist = &doctype.dtd[1];
	assert_eq!(attlist.keyword, "ATTLIST");
	assert_eq!(attlist.args.len(), 4);
	assert_eq!(attlist.args[0].value_raw(), "test");
	assert_eq!(attlist.args[1].value_raw(), "id");
	assert_eq!(attlist.args[2].value_raw(), "ID");
	assert_eq!(attlist.args[3].value_raw(), "#REQUIRED");
	assert_eq!(attlist.dtd.len(), 0);
	assert_eq!(attlist.span, SourceSpan::new(35, 66));
}

#[test]
fn keeps_non_element_markup_inside_open_elements() {
	let doc = Document::parse("<root><?pi mode=\"test\"?><!DOCTYPE html><child/></root>");
	assert_eq!(doc.errors, vec![]);

	let root = doc.children[0].element().unwrap();
	assert_eq!(root.children.len(), 3);

	let pi = root.children[0].processing_instruction().unwrap();
	assert_eq!(pi.target, "pi");
	assert_eq!(pi.data[0].key, "mode");
	assert_eq!(pi.data[0].value.as_ref().map(|value| value.value_raw()), Some("test"));

	let doctype = root.children[1].doctype().unwrap();
	assert_eq!(doctype.keyword, "DOCTYPE");
	assert_eq!(doctype.args[0].value_raw(), "html");

	assert_eq!(root.children[2].element().map(|element| element.tag), Some("child"));
	assert_eq!(root.span, SourceSpan::new(0, 54));
}

#[test]
fn keeps_script_contents_out_of_html_parsing() {
	let doc = Document::parse(r#"<script>if (n <= 1) { document.write(["<", "/script>"].join("")); }</script><div>ok</div>"#);
	assert_eq!(doc.errors, vec![]);
	assert_eq!(doc.children.len(), 2);

	let script = doc.children[0].element().unwrap();
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

	let div = doc.children[1].element().unwrap();
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

	let textarea = doc.children[0].element().unwrap();
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
			ParseError { span: SourceSpan::new(18, 20), kind: ParseErrorKind::SelfClosingEndTag },
			ParseError { span: SourceSpan::new(10, 20), kind: ParseErrorKind::UnexpectedToken },
			ParseError { span: SourceSpan::new(0, 8), kind: ParseErrorKind::UnclosedElement },
		],
	);

	let script = doc.children[0].element().unwrap();
	assert_eq!(script.children.len(), 2);
	assert_eq!(script.children[0].text().map(|text| text.text), Some("ok"));
	assert_eq!(script.children[1].text().map(|text| text.text), Some("tail"));
}

#[test]
fn element_text_decoding_varies_by_element_kind() {
	let doc = Document::parse(r#"
		<div>Tom &amp; Jerry &lt;3 &#33;</div>
		<script>const msg = "&amp;";</script>
		<style>.x::before { content: "&amp;"; }</style>
		<textarea>&lt;b&gt;ok&amp;go&lt;/b&gt;</textarea>
		<title>A &amp; B</title>"#,
	).trimmed();
	assert_eq!(doc.errors, []);

	let div = doc.children[0].element().unwrap();
	assert_eq!(div.tag, "div");
	assert_eq!(div.text_content(), "Tom & Jerry <3 !");

	let script = doc.children[1].element().unwrap();
	assert_eq!(script.tag, "script");
	assert_eq!(script.text_content(), "const msg = \"&amp;\";");

	let style = doc.children[2].element().unwrap();
	assert_eq!(style.tag, "style");
	assert_eq!(style.text_content(), ".x::before { content: \"&amp;\"; }");

	let textarea = doc.children[3].element().unwrap();
	assert_eq!(textarea.tag, "textarea");
	assert_eq!(textarea.text_content(), "<b>ok&go</b>");

	let title = doc.children[4].element().unwrap();
	assert_eq!(title.tag, "title");
	assert_eq!(title.text_content(), "A & B");
}

#[test]
fn trimmed_trims_comments_and_drops_empty_text_nodes() {
	let doc = Document::parse(" \n <!-- spaced --> \n <div> ok </div> \n ").trimmed();
	assert_eq!(doc.errors, []);
	assert_eq!(doc.children.len(), 2);

	let comment = doc.children[0].comment().unwrap();
	assert_eq!(comment.comment, "spaced");

	let div = doc.children[1].element().unwrap();
	assert_eq!(div.tag, "div");
	assert_eq!(div.children.len(), 1);
	assert_eq!(div.children[0].text().map(|text| text.text), Some("ok"));
}

#[test]
fn document_visit_and_parents_track_dom_ancestry() {
	let doc = Document::parse(r#"
		<div id="root">
			<section id="skip">
				<span id="skipped"></span>
			</section>
			<article id="stop">
				<span id="after-stop"></span>
			</article>
			<footer id="never"></footer>
		</div>
	"#).trimmed();

	let mut visited = Vec::new();
	doc.visit(|parents, element| {
		visited.push((parents.iter().filter_map(|parent| parent.id).collect(), element.id.unwrap()));
		match element.id {
			Some("skip") => VisitControl::Continue,
			Some("stop") => VisitControl::Stop,
			_ => VisitControl::Descend,
		}
	});

	assert_eq!(visited, vec![
		(vec![], "root"),
		(vec!["root"], "skip"),
		(vec!["root"], "stop"),
	]);

	let parents = doc.parents();
	let root = doc.children[0].element().unwrap();
	let skip_node = &root.children[0];
	let skipped_node = &root.children[0].element().unwrap().children[0];
	let stop_node = &root.children[1];
	let after_stop_node = &root.children[1].element().unwrap().children[0];

	assert_eq!(parents.get(&(skip_node as *const Node)).unwrap().id, Some("root"));
	assert_eq!(parents.get(&(skipped_node as *const Node)).unwrap().id, Some("skip"));
	assert_eq!(parents.get(&(stop_node as *const Node)).unwrap().id, Some("root"));
	assert_eq!(parents.get(&(after_stop_node as *const Node)).unwrap().id, Some("stop"));
}

#[test]
fn element_helpers_scope_queries_and_decode_values() {
	let doc = Document::parse(r#"
		<div id="root">
			<section id="scope">
				<a id="link" href="a&amp;b">one &amp; two<strong id="nested">!</strong></a>
				<script id="raw">&amp;</script>
				<p id="blank">  </p>
				<span class="target" id="inner"></span>
			</section>
			<span class="target" id="outside"></span>
		</div>
	"#).trimmed();

	let root = doc.children[0].element().unwrap();
	let section = root.query_selector("#scope").unwrap();
	let link = section.query_selector("a").unwrap();
	let raw = section.query_selector("#raw").unwrap();
	let blank = section.query_selector("#blank").unwrap();

	assert_eq!(link.get_attribute("href").map(|attribute| attribute.key), Some("href"));
	assert_eq!(link.get_attribute_value("href").as_deref(), Some("a&b"));
	assert_eq!(link.get_attribute_value("missing").as_deref(), None);
	assert_eq!(link.text_content(), "one & two!");
	assert_eq!(raw.text_content(), "&amp;");
	assert_eq!(blank.text_content(), "");
	assert_eq!(section.query_selector("#outside"), None);

	let matches = section.query_selector_all("span.target");
	assert_eq!(matches.len(), 1);
	assert_eq!(matches[0].id, Some("inner"));

	let mut visited = Vec::new();
	section.visit(|parents, element| {
		visited.push((parents.iter().filter_map(|parent| parent.id).collect::<Vec<_>>(), element.id));
		match element.id {
			Some("link") => VisitControl::Continue,
			_ => VisitControl::Descend,
		}
	});

	assert_eq!(visited, vec![
		(vec![], Some("link")),
		(vec![], Some("raw")),
		(vec![], Some("blank")),
		(vec![], Some("inner")),
	]);
}
