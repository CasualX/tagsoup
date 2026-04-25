use super::super::*;

#[test]
fn walks_descendants_depth_first_and_stops_when_callback_returns_false() {
	let doc = Document::parse(r#"
		<div>
			<section>
				<span class="target" id="a"></span>
				<div>
					<span class="target" id="b"></span>
				</div>
			</section>
			<section>
				<span class="target" id="c"></span>
			</section>
		</div>
	"#);
	let steps = selector::parser::Parser::parse("span.target").unwrap();
	let mut result = Vec::new();

	query(&doc.children, &steps, &mut |element| {
		result.push(element);
		result.len() < 2
	});

	assert_eq!(result.len(), 2);
	assert_eq!(result[0].id, Some("a"));
	assert_eq!(result[1].id, Some("b"));
}

#[test]
fn chains_descendant_and_child_groups() {
	let doc = Document::parse(r#"
		<div>
			<section class="match" data-kind="alpha">
				<span class="target" id="direct"></span>
				<div>
					<span class="target" id="nested"></span>
				</div>
			</section>
			<section class="other" data-kind="alpha">
				<span class="target" id="other"></span>
			</section>
		</div>
	"#);

	let result = doc.query_selector("section.match[data-kind=alpha] > span.target");
	assert_eq!(result.unwrap().id, Some("direct"));
}

#[test]
fn parses_steps_and_queries_direct_children() {
	let doc = Document::parse(r#"
		<main>
			<section class="card">
				<span class="title" id="a"></span>
				<div>
					<span class="title" id="b"></span>
				</div>
			</section>
			<section>
				<span class="title" id="c"></span>
			</section>
		</main>
	"#);

	let result = doc.query_selector("section.card > span.title");
	assert_eq!(result.unwrap().id, Some("a"));
}

#[test]
fn matches_attribute_contains_filters() {
	let doc = Document::parse(r#"
		<div>
			<section data-kind="hero-card" id="a"></section>
			<section data-kind="card" id="b"></section>
			<section data-kind="hero-banner" id="c"></section>
		</div>
	"#);

	let result = doc.query_selector_all("section[data-kind*=hero]");
	assert_eq!(result.len(), 2);
	assert_eq!(result[0].id, Some("a"));
	assert_eq!(result[1].id, Some("c"));
}

#[test]
fn matches_attribute_hyphen_filters() {
	let doc = Document::parse(r#"
		<div>
			<section lang="en" id="a"></section>
			<section lang="en-US" id="b"></section>
			<section lang="english" id="c"></section>
			<section lang="fr" id="d"></section>
		</div>
	"#);

	let result = doc.query_selector_all("section[lang|=en]");
	assert_eq!(result.len(), 2);
	assert_eq!(result[0].id, Some("a"));
	assert_eq!(result[1].id, Some("b"));
}

#[test]
fn matches_universal_empty_and_first_child_filters() {
	let doc = Document::parse(r#"
		<div>
			<section id="a"></section>
			<section id="b"> text </section>
			<section id="c"><span></span></section>
		</div>
	"#);

	assert_eq!(doc.query_selector("*").unwrap().tag, "div");
	assert_eq!(doc.query_selector("section:first-child").unwrap().id, Some("a"));

	let result = doc.query_selector_all("section:empty");
	assert_eq!(result.len(), 1);
	assert_eq!(result[0].id, Some("a"));
}

#[test]
fn matches_last_child_and_only_child_filters() {
	let doc = Document::parse(r#"
		<div>
			<div>
				<section id="a"></section>
				<section id="b"></section>
				<section id="c"></section>
			</div>
			<article>
				<p id="only"></p>
			</article>
		</div>
	"#);

	assert_eq!(doc.query_selector("section:last-child").unwrap().id, Some("c"));
	assert_eq!(doc.query_selector("p:only-child").unwrap().id, Some("only"));
	assert_eq!(doc.query_selector_all("section:only-child").len(), 0);
}

#[test]
fn matches_nth_child_filters() {
	let doc = Document::parse(r#"
		<ul>
			<li id="a"></li>
			<li id="b"></li>
			<li id="c"></li>
			<li id="d"></li>
			<li id="e"></li>
		</ul>
	"#);

	let even = doc.query_selector_all("li:nth-child(even)");
	assert_eq!(even.len(), 2);
	assert_eq!(even[0].id, Some("b"));
	assert_eq!(even[1].id, Some("d"));

	let odd = doc.query_selector_all("li:nth-child(odd)");
	assert_eq!(odd.len(), 3);
	assert_eq!(odd[0].id, Some("a"));
	assert_eq!(odd[1].id, Some("c"));
	assert_eq!(odd[2].id, Some("e"));

	let formula = doc.query_selector_all("li:nth-child(2n+1)");
	assert_eq!(formula.len(), 3);
	assert_eq!(formula[0].id, Some("a"));
	assert_eq!(formula[1].id, Some("c"));
	assert_eq!(formula[2].id, Some("e"));

	assert_eq!(doc.query_selector("li:nth-child(3)").unwrap().id, Some("c"));
	assert_eq!(doc.query_selector("li:nth-child(-n+3)").unwrap().id, Some("a"));
	assert_eq!(doc.query_selector_all("li:nth-child(-n+3)").len(), 3);

	let nth_last = doc.query_selector_all("li:nth-last-child(2n+1)");
	assert_eq!(nth_last.len(), 3);
	assert_eq!(nth_last[0].id, Some("a"));
	assert_eq!(nth_last[1].id, Some("c"));
	assert_eq!(nth_last[2].id, Some("e"));

	assert_eq!(doc.query_selector("li:nth-last-child(2)").unwrap().id, Some("d"));
	assert_eq!(doc.query_selector_all("li:nth-last-child(-n+3)").len(), 3);
}

#[test]
fn query_selector_does_not_descend_into_template_contents() {
	let doc = Document::parse(r#"
		<div>
			<template>
				<span class="target" id="inside"></span>
			</template>
			<span class="target" id="outside"></span>
		</div>
	"#);

	assert_eq!(doc.query_selector("#inside"), None);
	assert_eq!(doc.query_selector_all("span.target").len(), 1);
	assert_eq!(doc.query_selector("#outside").map(|element| element.id), Some(Some("outside")));
	assert_eq!(doc.query_selector("template").map(|element| element.tag), Some("template"));
	assert_eq!(doc.children[1].element().unwrap().children[1].element().unwrap().tag, "template");
	assert_eq!(doc.children[1].element().unwrap().children[1].element().unwrap().children[1].element().unwrap().id, Some("inside"));
}

#[test]
fn matches_next_sibling_ignoring_non_element_nodes() {
	let doc = Document::parse(r#"
		<div>
			<h2 id="heading"></h2>
			<!-- break -->
			<p class="target" id="match"></p>
			<p class="target" id="later"></p>
		</div>
	"#);

	let result = doc.query_selector("h2 + p.target");
	assert_eq!(result.unwrap().id, Some("match"));
}

#[test]
fn matches_subsequent_siblings_only_with_same_parent() {
	let doc = Document::parse(r#"
		<div>
			<h2 id="heading"></h2>
			<section>
				<p class="target" id="nested"></p>
			</section>
			<p class="target" id="first"></p>
			<p class="target" id="second"></p>
		</div>
	"#);

	let result = doc.query_selector_all("h2 ~ p.target");
	assert_eq!(result.len(), 2);
	assert_eq!(result[0].id, Some("first"));
	assert_eq!(result[1].id, Some("second"));
}

#[test]
fn matches_descendants_with_attribute_presence_prefix_suffix_and_word_filters() {
	let doc = Document::parse(r#"
		<div id="root">
			<article id="outer">
				<section>
					<a id="match" href="https://example.com/card" rel="nofollow noopener" data-kind="hero-card"></a>
				</section>
			</article>
			<article>
				<a id="missing-href" rel="nofollow noopener" data-kind="hero-card"></a>
				<a id="wrong-prefix" href="ftp://example.com/card" rel="nofollow noopener" data-kind="hero-card"></a>
				<a id="wrong-suffix" href="https://example.com/feed" rel="nofollow noopener" data-kind="hero-card"></a>
				<a id="wrong-word" href="https://example.com/card" rel="follow" data-kind="hero-card"></a>
				<a id="wrong-kind" href="https://example.com/card" rel="nofollow noopener" data-kind="card-hero"></a>
			</article>
		</div>
	"#);

	let result = doc.query_selector_all("div article a[href][href^=https][rel~=nofollow][data-kind^=hero][href$=card]");
	assert_eq!(result.len(), 1);
	assert_eq!(result[0].id, Some("match"));
}
