use super::*;

impl selector::Filter<'_> {
	fn matches(&self, node: &Node) -> bool {
		let Node::Element(element) = node else { return false };
		match self {
			&selector::Filter::Tag(filter_tag) => element.tag.eq_ignore_ascii_case(filter_tag),
			&selector::Filter::Class(filter_class) => element.get_attribute_value("class").map_or(false, |class_list| class_list.split_ascii_whitespace().any(|class| class == filter_class)),
			&selector::Filter::Id(filter_id) => element.id == Some(filter_id),
			&selector::Filter::AttrExists(filter_name) => element.get_attribute(filter_name).is_some(),
			&selector::Filter::AttrEquals { name, value } => element.get_attribute_value(name).map_or(false, |v| v == value),
			&selector::Filter::AttrContains { name, value } => element.get_attribute_value(name).map_or(false, |v| v.contains(value)),
			&selector::Filter::AttrStartsWith { name, value } => element.get_attribute_value(name).map_or(false, |v| v.starts_with(value)),
			&selector::Filter::AttrEndsWith { name, value } => element.get_attribute_value(name).map_or(false, |v| v.ends_with(value)),
			&selector::Filter::AttrWhitespaceContains { name, value } => element.get_attribute_value(name).map_or(false, |v| v.split_ascii_whitespace().any(|part| part == value)),
		}
	}
}

pub fn query<'a, 'dom>(nodes: &'dom [Node<'a>], steps: &[selector::Step], result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
	let Some((step, rest)) = steps.split_first() else { return true };
	match step {
		selector::Step::Combinator(selector::Combinator::Child) => {
			filter_nodes(nodes, rest, false, result_fn)
		}
		selector::Step::Combinator(selector::Combinator::Descendant) => {
			filter_nodes(nodes, rest, true, result_fn)
		}
		selector::Step::Combinator(selector::Combinator::NextSibling) => {
			filter_siblings(nodes, rest, true, result_fn)
		}
		selector::Step::Combinator(selector::Combinator::SubsequentSibling) => {
			filter_siblings(nodes, rest, false, result_fn)
		}
		selector::Step::Filter(_) => true,
	}
}

fn filter_nodes<'a, 'dom>(nodes: &'dom [Node<'a>], steps: &[selector::Step], descendant: bool, result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
	for (index, node) in nodes.iter().enumerate() {
		if !filter_node(nodes, index, node, steps, result_fn) {
			return false;
		}

		if descendant && let Node::Element(element) = node && !matches!(element.kind, ElementKind::Template) {
			if !filter_nodes(&element.children, steps, true, result_fn) {
				return false;
			}
		}
	}

	true
}

fn filter_siblings<'a, 'dom>(nodes: &'dom [Node<'a>], steps: &[selector::Step], adjacent_only: bool, result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
	for (index, node) in nodes.iter().enumerate() {
		if node.element().is_none() {
			continue;
		}

		if !filter_node(nodes, index, node, steps, result_fn) {
			return false;
		}

		if adjacent_only {
			break;
		}
	}

	true
}

fn filter_node<'a, 'dom>(siblings: &'dom [Node<'a>], index: usize, node: &'dom Node<'a>, steps: &[selector::Step], result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
	let Node::Element(element) = node else { return true };
	let Some(rest) = match_filters(node, steps) else { return true };

	if rest.is_empty() {
		return result_fn(element);
	}

	match rest[0] {
		selector::Step::Combinator(selector::Combinator::Child) | selector::Step::Combinator(selector::Combinator::Descendant) => {
			if matches!(element.kind, ElementKind::Template) {
				return true;
			}

			query(&element.children, rest, result_fn)
		}
		selector::Step::Combinator(selector::Combinator::NextSibling) | selector::Step::Combinator(selector::Combinator::SubsequentSibling) => {
			query(&siblings[index + 1..], rest, result_fn)
		}
		selector::Step::Filter(_) => true,
	}
}

fn match_filters<'a, 'step>(node: &Node<'a>, mut steps: &'step [selector::Step<'step>]) -> Option<&'step [selector::Step<'step>]> {
	let mut matched = false;

	loop {
		let Some((step, rest)) = steps.split_first() else {
			return matched.then_some(steps);
		};

		match step {
			selector::Step::Combinator(_) => return matched.then_some(steps),
			selector::Step::Filter(filter) => {
				if !filter.matches(node) {
					return None;
				}
				matched = true;
				steps = rest;
			}
		}
	}
}
