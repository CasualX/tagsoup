// This example illustrates how to use the library to get all of the anchor-hrefs from a document.

fn main() {
	let html = include_str!("./index.html");
	let dom = tagsoup::Document::parse(html);
	let mut hrefs = Vec::new();

	dom.visit(|_parents, element| {
		if element.tag.eq_ignore_ascii_case("a") {
			if let Some(href) = element.get_attribute_value("href") {
				hrefs.push(href);
			}
		}
		tagsoup::VisitControl::Descend
	});

	println!("\nThe following links where found:");
	for (index, href) in hrefs.into_iter().enumerate() {
		println!("{}: {}", index + 1, href)
	}
}
