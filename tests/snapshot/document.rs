use super::*;

#[test]
fn it_can_parse_minimal_document() {
	let html = "<!DOCTYPE html><html></html>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_document_with_comments() {
	let html =
		r#"
		<!-- comment -->
		<!-- comment -->
		<!DOCTYPE html>
		<!-- comment -->
		<!-- comment -->
		<html>
		<!-- comment -->
		</html>
		<!-- comment -->
		<!-- comment -->
	"#;
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}
