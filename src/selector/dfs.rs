use super::*;

#[inline]
fn descends_into_children(element: &Element<'_>) -> bool {
	!matches!(element.kind, ElementKind::Template)
}

pub fn query<'a, 'dom>(nodes: &'dom [Node<'a>], steps: &[Step], result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
	let Some((step, rest)) = steps.split_first() else { return true };
	match step {
		Step::Combinator(Combinator::Child) => {
			filter_nodes(nodes, rest, false, result_fn)
		}
		Step::Combinator(Combinator::Descendant) => {
			filter_nodes(nodes, rest, true, result_fn)
		}
		Step::Filter(_) => true,
	}
}

fn filter_nodes<'a, 'dom>(nodes: &'dom [Node<'a>], steps: &[Step], descendant: bool, result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
	for node in nodes {
		if !filter_node(node, steps, result_fn) {
			return false;
		}

		if descendant && let Node::Element(element) = node && descends_into_children(element) {
			if !filter_nodes(&element.children, steps, true, result_fn) {
				return false;
			}
		}
	}

	true
}

fn filter_node<'a, 'dom>(node: &'dom Node<'a>, steps: &[Step], result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) -> bool {
	let Node::Element(element) = node else { return true };
	let Some(rest) = match_filters(node, steps) else { return true };

	if rest.is_empty() {
		return result_fn(element);
	}

	if !descends_into_children(element) {
		return true;
	}

	query(&element.children, rest, result_fn)
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
