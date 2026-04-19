use super::*;

#[test]
fn it_can_parse_double_quote() {
	let html = "<div id=\"one\"></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_single_quote() {
	let html = "<div id='one'></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_no_quote() {
	let html = "<div id=one></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_attribute_key_mixed_case_symbols() {
	let html = "<div data-cat='morris'></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_vue_style_attribute_shorthand() {
	let html = "<vue-style @attributes=\"or\" :bindings=\"are_not_supported\"></vue-style>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_multiple_attributes_single_quote() {
	let html = "<div cat='mjau' dog='woff' ape=oh></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_multiple_attributes_where_whitespace_does_not_matter_for_keys() {
	let html = "<div	cat   =  \"mjau\" dog ='  woff  'ape = oh ></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_multiple_attributes_double_quote() {
	let html = "<div cat=\"mjau\" dog=\"woff\" ape=\"oh\"></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_multiple_attributes_no_quote() {
	let html = "<div cat=mjau dog=woff ape=oh></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_attribute_multiple_values_single_quote() {
	let html = "<div cat='mjau mjau' />";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_attribute_multiple_values_double_quote() {
	let html = "<div cat=\"mjau mjau\" />";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_attribute_with_empty_value() {
	let html = "<img hidden/>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_id() {
	let html = "<img id=a>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_classes() {
	let html = "<img class='a b c'/>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_keeps_spaces_for_non_classes() {
	let html = "<img attr=' a b	 \n\t'/>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}
