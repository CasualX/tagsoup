use super::*;

#[test]
fn it_can_parse_empty_document() {
	let html = "";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}
