use super::*;
use std::hash::{Hash, Hasher};

/// Element tag name, case-insensitive newtype for hash map lookups.
#[derive(Copy, Clone, Debug)]
pub struct ElementTag<'a>(pub &'a str);

impl<'a> From<&'a str> for ElementTag<'a> {
	#[inline]
	fn from(value: &'a str) -> Self {
		Self(value)
	}
}

impl PartialEq for ElementTag<'_> {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		self.0.eq_ignore_ascii_case(other.0)
	}
}

impl Eq for ElementTag<'_> {}

impl Hash for ElementTag<'_> {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		for byte in self.0.bytes() {
			state.write_u8(byte.to_ascii_lowercase());
		}
	}
}

/// Map of elements in the document.
///
/// Indexed by id, tag name, and class name.
#[derive(Clone, Debug, Default)]
pub struct Map<'a, 'dom> {
	pub by_id: HashMap<&'a str, &'dom Element<'a>>,
	pub by_tag: HashMap<ElementTag<'a>, Vec<&'dom Element<'a>>>,
	pub by_class: HashMap<&'a str, Vec<&'dom Element<'a>>>,
}

impl<'a, 'dom> Map<'a, 'dom> {
	/// Creates a new empty map.
	pub fn new() -> Self {
		Map { by_id: HashMap::new(), by_tag: HashMap::new(), by_class: HashMap::new() }
	}

	/// Builds a map from the given document.
	pub fn build(document: &'dom Document<'a>) -> Map<'a, 'dom> {
		let mut map = Map { by_id: HashMap::new(), by_tag: HashMap::new(), by_class: HashMap::new() };
		map.add_ids(&document.children);
		map.add_tags(&document.children);
		map.add_classes(&document.children);
		map
	}
}

impl<'a, 'dom> Map<'a, 'dom> {
	pub fn add_ids(&mut self, nodes: &'dom [Node<'a>]) {
		for node in nodes {
			if let Node::Element(element) = node {
				if let Some(id) = element.id {
					self.by_id.insert(id, element);
				}
				self.add_ids(&element.children);
			}
		}
	}
	pub fn add_tags(&mut self, nodes: &'dom [Node<'a>]) {
		for node in nodes {
			if let Node::Element(element) = node {
				self.by_tag.entry(element.tag.into()).or_default().push(element);
				self.add_tags(&element.children);
			}
		}
	}
	pub fn add_classes(&mut self, nodes: &'dom [Node<'a>]) {
		for node in nodes {
			if let Node::Element(element) = node {
				if let Some(class_attr) = element.get_attribute_value("class") {
					for class in class_attr.split_ascii_whitespace() {
						self.by_class.entry(class).or_default().push(element);
					}
				}
				self.add_classes(&element.children);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn groups_tags_case_insensitively() {
		let doc = Document::parse("<DIV></DIV><div></div><Span></Span>");
		let map = Map::build(&doc);

		assert_eq!(map.by_tag.get(&ElementTag("div")).map(Vec::len), Some(2));
		assert_eq!(map.by_tag.get(&ElementTag("DIV")).map(Vec::len), Some(2));
		assert_eq!(map.by_tag.get(&ElementTag("span")).map(Vec::len), Some(1));
	}

	#[test]
	fn preserves_original_tag_text_in_case_insensitive_keys() {
		let key = ElementTag("HeAd");

		assert_eq!(key.0, "HeAd");
		assert_eq!(key, ElementTag("head"));
	}
}
