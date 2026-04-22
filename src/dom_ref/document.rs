use super::*;

/// Document represents the entire parsed HTML document fragment.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Document<'a> {
	/// The root nodes of the document.
	pub children: Vec<Node<'a>>,

	/// A list of parse errors encountered while parsing the document.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub errors: Vec<ParseError>,
}

impl<'a> Document<'a> {
	/// Parses the input HTML and returns the document and any parse errors.
	///
	/// ```
	/// // Parse an HTML fragment.
	/// let doc = tagsoup::Document::parse("<div><p id=here>Hello, world!</p></div>");
	///
	/// // Check for parsing errors.
	/// assert!(doc.errors.is_empty());
	///
	/// // Query the document for an element using a CSS selector.
	/// let element = doc.query_selector("#here").unwrap();
	/// assert_eq!(element.text_content(), "Hello, world!");
	/// ```
	#[inline]
	pub fn parse(input: &'a str) -> Document<'a> {
		parser::parse(input)
	}
}

fn trim_span<'a>(span: &mut SourceSpan, text: &mut &'a str) {
	// Trim leading whitespace
	let s = text.trim_ascii_start();
	span.start += (text.len() - s.len()) as u32;
	// Trim trailing whitespace
	let s = s.trim_ascii_end();
	span.end -= (text.len() - s.len()) as u32;
	*text = s;
}

fn trim_nodes<'a>(nodes: &mut Vec<Node<'a>>) {
	nodes.retain_mut(|node| {
		match node {
			Node::Text(text) => {
				trim_span(&mut text.span, &mut text.text);
				!text.text.is_empty()
			}
			Node::Element(element) => {
				trim_nodes(&mut element.children);
				true
			}
			Node::Comment(comment) => {
				// Do not update comment.span as it spans the entire comment
				comment.comment = comment.comment.trim_ascii();
				true
			}
			Node::Doctype(_) => true,
			Node::ProcessingInstruction(_) => true,
		}
	})
}

impl<'a> Document<'a> {
	/// Recursively trims all ascii whitespace from the document's text.
	///
	/// By default, the parser preserves all whitespace in the input.
	/// This method trims all leading and trailing ASCII whitespace and removes any empty text nodes.
	///
	/// ```
	/// let doc = tagsoup::Document::parse("  <div>  Hello, world!  </div>  ").trimmed();
	/// assert_eq!(doc.children.len(), 1);
	///
	/// let element = doc.children[0].element().unwrap();
	/// assert_eq!(element.tag, "div");
	/// assert_eq!(element.children.len(), 1);
	///
	/// let text = element.children[0].text().unwrap();
	/// assert_eq!(text.text, "Hello, world!");
	/// ```
	#[inline]
	pub fn trimmed(mut self) -> Self {
		trim_nodes(&mut self.children);
		self
	}
}

impl<'a> Document<'a> {
	/// Visits all elements in the DOM tree depth-first.
	///
	/// The visitor function is called for each element in the tree.
	///
	/// The visitor is passed a slice of parent elements (ordered from root to immediate parent) and the current element.
	///
	/// ```
	/// let html = r#"
	/// 	<ul>
	/// 		<li><a href="/one">One</a></li>
	/// 		<li><a href="/two">Two</a></li>
	/// 	</ul>
	/// "#;
	///
	/// let doc = tagsoup::Document::parse(html);
	/// let mut hrefs = Vec::new();
	///
	/// doc.visit(|_parents, element| {
	/// 	if element.tag.eq_ignore_ascii_case("a") {
	/// 		if let Some(href) = element.get_attribute_value("href") {
	/// 			hrefs.push(href);
	/// 		}
	/// 	}
	///
	/// 	tagsoup::VisitControl::Descend
	/// });
	///
	/// assert_eq!(hrefs, vec!["/one", "/two"]);
	/// ```
	pub fn visit<'dom>(&'dom self, mut visitor: impl FnMut(&[&'dom Element<'a>], &'dom Element<'a>) -> VisitControl) {
		let mut parents = Vec::new();
		visit(&self.children, &mut parents, &mut visitor);
	}

	/// Returns a map of all nodes in the document to their parent element.
	pub fn parents<'dom>(&'dom self) -> HashMap<*const Node<'a>, &'dom Element<'a>> {
		let mut map = HashMap::new();
		for child in &self.children {
			if let Node::Element(element) = child {
				parents(element, &element.children, &mut map);
			}
		}
		map
	}

	/// Queries the document for the first element matching the given CSS selector.
	///
	/// Selector queries do not descend into `<template>` contents.
	///
	/// Panics if the selector is invalid.
	pub fn query_selector(&self, selector: &str) -> Option<&Element<'a>> {
		let steps = selector::parser::Parser::parse(selector).expect(selector);
		let mut result = None;
		dfs::query(&self.children, &steps, &mut |element| {
			result = Some(element);
			false
		});
		result
	}

	/// Queries the document for all elements matching the given CSS selector.
	///
	/// Selector queries do not descend into `<template>` contents.
	///
	/// Panics if the selector is invalid.
	pub fn query_selector_all(&self, selector: &str) -> Vec<&Element<'a>> {
		let steps = selector::parser::Parser::parse(selector).expect(selector);
		let mut result = Vec::new();
		dfs::query(&self.children, &steps, &mut |element| {
			result.push(element);
			true
		});
		result
	}
}

fn parents<'a, 'dom>(parent: &'dom Element<'a>, nodes: &'dom [Node<'a>], map: &mut HashMap<*const Node<'a>, &'dom Element<'a>>) {
	for node in nodes {
		let ptr = node as *const Node<'a>;
		map.insert(ptr, parent);
		if let Node::Element(element) = node {
			parents(element, &element.children, map);
		}
	}
}
