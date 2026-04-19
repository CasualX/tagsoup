use super::*;

pub fn query<'a, 'dom>(nodes: &'dom [Node<'a>], steps: &[Step], result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
	let Some((step, rest)) = steps.split_first() else { return true };
	match step {
		Step::Combinator(Combinator::Child) => {
			filter_nodes(nodes, rest, false, result_fn)
		}
		Step::Combinator(Combinator::Descendant) => {
			filter_nodes(nodes, rest, true, result_fn)
		}
		Step::Combinator(Combinator::NextSibling) => {
			filter_siblings(nodes, rest, true, result_fn)
		}
		Step::Combinator(Combinator::SubsequentSibling) => {
			filter_siblings(nodes, rest, false, result_fn)
		}
		Step::Filter(_) => true,
	}
}

fn filter_nodes<'a, 'dom>(nodes: &'dom [Node<'a>], steps: &[Step], descendant: bool, result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
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

fn filter_siblings<'a, 'dom>(nodes: &'dom [Node<'a>], steps: &[Step], adjacent_only: bool, result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
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

fn filter_node<'a, 'dom>(siblings: &'dom [Node<'a>], index: usize, node: &'dom Node<'a>, steps: &[Step], result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
	let Node::Element(element) = node else { return true };
	let Some(rest) = match_filters(node, steps) else { return true };

	if rest.is_empty() {
		return result_fn(element);
	}

	match rest[0] {
		Step::Combinator(Combinator::Child) | Step::Combinator(Combinator::Descendant) => {
			if matches!(element.kind, ElementKind::Template) {
				return true;
			}

			query(&element.children, rest, result_fn)
		}
		Step::Combinator(Combinator::NextSibling) | Step::Combinator(Combinator::SubsequentSibling) => {
			query(&siblings[index + 1..], rest, result_fn)
		}
		Step::Filter(_) => true,
	}
}

fn match_filters<'a, 'step>(node: &Node<'a>, mut steps: &'step [Step<'step>]) -> Option<&'step [Step<'step>]> {
	let mut matched = false;

	loop {
		let Some((step, rest)) = steps.split_first() else {
			return matched.then_some(steps);
		};

		match step {
			Step::Combinator(_) => return matched.then_some(steps),
			Step::Filter(filter) => {
				if !filter.matches(node) {
					return None;
				}
				matched = true;
				steps = rest;
			}
		}
	}
}
