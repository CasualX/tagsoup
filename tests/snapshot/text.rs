use super::*;

#[test]
fn it_can_parse_document_with_just_text() {
	let html = "hello world";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_document_with_text_and_line_breaks() {
	let html = r"
hello world
here's another line for you!
The end";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_document_with_multiple_text_elements() {
	let html = r"
hello world
here's another line for you!
<div/>
The end";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_text_with_chevron() {
	let html = r"hello <> world";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_text_in_paragraph_with_weird_formatting() {
	let html = r"
<p>
	This is a <b>para</b>gra<b>ph</b> with some<i> weird </i> formatting.
</p>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}
