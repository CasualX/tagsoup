use super::*;

pub mod dfs;
pub mod parser;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Combinator {
	Descendant,
	Child,
}

#[derive(Debug, PartialEq)]
pub enum Step<'step> {
	Combinator(Combinator),
	Filter(Filter<'step>),
}

#[derive(Debug, PartialEq)]
pub enum Filter<'step> {
	Tag(&'step str),
	Class(&'step str),
	Id(&'step str),
	AttrExists(&'step str),
	AttrEquals { name: &'step str, value: &'step str },
	AttrContains { name: &'step str, value: &'step str },
	AttrStartsWith { name: &'step str, value: &'step str },
	AttrEndsWith { name: &'step str, value: &'step str },
	AttrWhitespaceContains { name: &'step str, value: &'step str },
}

impl Filter<'_> {
	fn matches(&self, node: &Node) -> bool {
		let Node::Element(element) = node else { return false };
		match self {
			&Filter::Tag(filter_tag) => element.tag.eq_ignore_ascii_case(filter_tag),
			&Filter::Class(filter_class) => element.get_attribute_value("class").map_or(false, |class_list| class_list.split_ascii_whitespace().any(|class| class == filter_class)),
			&Filter::Id(filter_id) => element.id == Some(filter_id),
			&Filter::AttrExists(filter_name) => element.get_attribute(filter_name).is_some(),
			&Filter::AttrEquals { name, value } => element.get_attribute_value(name).map_or(false, |v| v == value),
			&Filter::AttrContains { name, value } => element.get_attribute_value(name).map_or(false, |v| v.contains(value)),
			&Filter::AttrStartsWith { name, value } => element.get_attribute_value(name).map_or(false, |v| v.starts_with(value)),
			&Filter::AttrEndsWith { name, value } => element.get_attribute_value(name).map_or(false, |v| v.ends_with(value)),
			&Filter::AttrWhitespaceContains { name, value } => element.get_attribute_value(name).map_or(false, |v| v.split_ascii_whitespace().any(|part| part == value)),
		}
	}
}

#[cfg(test)]
mod tests;
