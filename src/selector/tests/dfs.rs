use super::super::*;

#[test]
fn walks_descendants_depth_first_and_stops_when_callback_returns_false() {
	let doc = Document::parse(
		"<div><section><span class='target' id='a'></span><div><span class='target' id='b'></span></div></section><section><span class='target' id='c'></span></section></div>",
	);
	let steps = parser::Parser::parse("span.target").unwrap();
	let mut result = Vec::new();

	dfs::query(&doc.children, &steps, &mut |element| {
		result.push(element);
		result.len() < 2
	});

	assert_eq!(result.len(), 2);
	assert_eq!(result[0].id, Some("a"));
	assert_eq!(result[1].id, Some("b"));
}

#[test]
fn chains_descendant_and_child_groups() {
	let doc = Document::parse(
		"<div><section class='match' data-kind='alpha'><span class='target' id='direct'></span><div><span class='target' id='nested'></span></div></section><section class='other' data-kind='alpha'><span class='target' id='other'></span></section></div>",
	);
	let steps = parser::Parser::parse("section.match[data-kind=alpha] > span.target").unwrap();
	let mut result = Vec::new();

	dfs::query(&doc.children, &steps, &mut |element| {
		result.push(element);
		true
	});

	assert_eq!(result.len(), 1);
	assert_eq!(result[0].id, Some("direct"));
}

#[test]
fn parses_steps_and_queries_direct_children() {
	let doc = Document::parse(
		"<main><section class='card'><span class='title' id='a'></span><div><span class='title' id='b'></span></div></section><section><span class='title' id='c'></span></section></main>",
	);
	let steps = parser::Parser::parse("section.card > span.title").unwrap();
	let mut result = Vec::new();

	dfs::query(&doc.children, &steps, &mut |element| {
		result.push(element);
		true
	});

	assert_eq!(result.len(), 1);
	assert_eq!(result[0].id, Some("a"));
}

#[test]
fn matches_attribute_contains_filters() {
	let doc = Document::parse(
		"<div><section data-kind='hero-card' id='a'></section><section data-kind='card' id='b'></section><section data-kind='hero-banner' id='c'></section></div>",
	);
	let steps = parser::Parser::parse("section[data-kind*=hero]").unwrap();
	let mut result = Vec::new();

	dfs::query(&doc.children, &steps, &mut |element| {
		result.push(element);
		true
	});

	assert_eq!(result.len(), 2);
	assert_eq!(result[0].id, Some("a"));
	assert_eq!(result[1].id, Some("c"));
}

#[test]
fn query_selector_does_not_descend_into_template_contents() {
	let doc = Document::parse(
		"<div><template><span class='target' id='inside'></span></template><span class='target' id='outside'></span></div>",
	);

	assert_eq!(doc.query_selector("#inside"), None);
	assert_eq!(doc.query_selector_all("span.target").len(), 1);
	assert_eq!(doc.query_selector("#outside").map(|element| element.id), Some(Some("outside")));
	assert_eq!(doc.query_selector("template").map(|element| element.tag), Some("template"));
	assert_eq!(doc.children[0].element().unwrap().children[0].element().unwrap().tag, "template");
	assert_eq!(doc.children[0].element().unwrap().children[0].element().unwrap().children[0].element().unwrap().id, Some("inside"));
}
