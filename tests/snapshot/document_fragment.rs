use super::*;

#[test]
fn it_can_parse_single_div_as_fragment() {
	let html = "<div/>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_single_text_as_fragment() {
	let html = "hello";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_text_comment_element_as_fragment() {
	let html = "hello<!--world?--><div/>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}
