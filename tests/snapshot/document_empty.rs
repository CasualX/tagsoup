use super::*;

#[test]
fn it_can_parse_empty_document() {
	let html = "";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert!(dom.is_empty());
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_document_with_no_elements() {
	let html = r#"
		<?xml version="1.0"?>
		<!DOCTYPE html>
		<!-- comment -->
	"#;
	let dom = tagsoup::Document::parse(html);
	assert!(dom.is_empty());
	assert_json_snapshot!(dom);
}
