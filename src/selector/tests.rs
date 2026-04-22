use super::*;

#[test]
fn parses_compound_selector_without_synthetic_leading_combinator() {
	let steps = parser::Parser::parse("div#main.hero[data-kind][lang=en]").unwrap();

	assert_eq!(steps, vec![
		Step::Filter(Filter::Tag("div")),
		Step::Filter(Filter::Id("main")),
		Step::Filter(Filter::Class("hero")),
		Step::Filter(Filter::AttrExists("data-kind")),
		Step::Filter(Filter::AttrEquals { name: "lang", value: "en" }),
	]);
}

#[test]
fn parses_child_and_descendant_combinators() {
	let steps = parser::Parser::parse("section.card > span.title").unwrap();

	assert_eq!(steps, vec![
		Step::Filter(Filter::Tag("section")),
		Step::Filter(Filter::Class("card")),
		Step::Combinator(Combinator::Child),
		Step::Filter(Filter::Tag("span")),
		Step::Filter(Filter::Class("title")),
	]);
}

#[test]
fn parses_sibling_combinators() {
	let steps = parser::Parser::parse("h2 + p.note ~ a[href]").unwrap();

	assert_eq!(steps, vec![
		Step::Filter(Filter::Tag("h2")),
		Step::Combinator(Combinator::NextSibling),
		Step::Filter(Filter::Tag("p")),
		Step::Filter(Filter::Class("note")),
		Step::Combinator(Combinator::SubsequentSibling),
		Step::Filter(Filter::Tag("a")),
		Step::Filter(Filter::AttrExists("href")),
	]);
}

#[test]
fn parses_whitespace_as_descendant_between_groups() {
	let steps = parser::Parser::parse("article .lead [data-kind*='feature']").unwrap();

	assert_eq!(steps, vec![
		Step::Filter(Filter::Tag("article")),
		Step::Combinator(Combinator::Descendant),
		Step::Filter(Filter::Class("lead")),
		Step::Combinator(Combinator::Descendant),
		Step::Filter(Filter::AttrContains { name: "data-kind", value: "feature" }),
	]);
}

#[test]
fn parses_attribute_contains_operator() {
	let steps = parser::Parser::parse("div[data-kind*=hero]").unwrap();

	assert_eq!(steps, vec![
		Step::Filter(Filter::Tag("div")),
		Step::Filter(Filter::AttrContains { name: "data-kind", value: "hero" }),
	]);
}

#[test]
fn parses_attribute_prefix_suffix_and_whitespace_operators() {
	let steps = parser::Parser::parse("div[data-kind^=hero][data-kind$=card][rel~=nofollow]").unwrap();

	assert_eq!(steps, vec![
		Step::Filter(Filter::Tag("div")),
		Step::Filter(Filter::AttrStartsWith { name: "data-kind", value: "hero" }),
		Step::Filter(Filter::AttrEndsWith { name: "data-kind", value: "card" }),
		Step::Filter(Filter::AttrWhitespaceContains { name: "rel", value: "nofollow" }),
	]);
}

#[test]
fn rejects_trailing_combinator() {
	let error = parser::Parser::parse("div >").unwrap_err();

	assert_eq!(error.kind, parser::ParseSelectorErrorKind::InvalidSelector);
}

#[test]
fn rejects_duplicate_sibling_combinators() {
	let error = parser::Parser::parse("div + + span").unwrap_err();

	assert_eq!(error.kind, parser::ParseSelectorErrorKind::InvalidSelector);
}

#[test]
fn rejects_leading_sibling_combinators() {
	let error = parser::Parser::parse("+ span").unwrap_err();

	assert_eq!(error.kind, parser::ParseSelectorErrorKind::InvalidSelector);
}

#[test]
fn rejects_leading_child_combinator() {
	let error = parser::Parser::parse("> span").unwrap_err();

	assert_eq!(error.kind, parser::ParseSelectorErrorKind::InvalidSelector);
}
