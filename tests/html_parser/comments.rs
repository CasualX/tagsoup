use super::*;

#[test]
fn it_can_parse_document_with_just_one_comment() {
	let html = "<!-- hello !\"#/()= -->";
	let ast = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(ast);
}
#[test]
fn it_can_parse_document_with_just_comments() {
	let html = "<!--x--><!--y--><!--z-->";
	let ast = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(ast);
}
