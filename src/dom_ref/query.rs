use super::*;

// We keep the containing slice with each frame so sibling combinators can stay
// local to one parent boundary instead of reconstructing that context later.
#[derive(Copy, Clone)]
struct Frame<'a, 'dom> {
	node_index: usize,
	element_index: usize,
	element: &'dom Element<'a>,
	siblings: &'dom [Node<'a>],
}

pub fn query<'a, 'dom>(children: &'dom [Node<'a>], steps: &[selector::Step], result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) {
	// Query each selector list independently, this may cause duplicates but keeps the implementation simple...
	for steps in steps.split(|step| matches!(step, selector::Step::SelectorList)) {
		if !query_in(children, &mut Vec::new(), steps, result_fn) {
			return;
		}
	}
}

fn query_in<'a, 'dom>(
	children: &'dom [Node<'a>],
	ancestors: &mut Vec<Frame<'a, 'dom>>,
	steps: &[selector::Step<'_>],
	result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool,
) -> bool {
	let mut element_index = 0;
	for (node_index, child) in children.iter().enumerate() {
		let Node::Element(element) = child else { continue };
		let frame = Frame { node_index, element_index, element, siblings: children };
		element_index += 1;

		if check_element(&frame, ancestors, steps) && !result_fn(element) {
			return false;
		}

		if matches!(element.kind, ElementKind::Template) {
			continue;
		}

		ancestors.push(frame);
		let keep_going = query_in(&element.children, ancestors, steps, result_fn);
		ancestors.pop();

		if !keep_going {
			return false;
		}
	}

	true
}

// Matching from the right lets us decide whether the current element is even a
// candidate before we pay to inspect ancestors or siblings.
fn split_last_compound<'a, 'step>(steps: &'step [selector::Step<'a>]) -> Option<(
	&'step [selector::Step<'a>],
	Option<&'step selector::Combinator>,
	&'step [selector::Step<'a>],
)> {
	match steps.iter().rposition(|step| matches!(step, selector::Step::Combinator(_))) {
		Some(index) => {
			let selector::Step::Combinator(combinator) = &steps[index] else { unreachable!() };
			Some((&steps[..index], Some(combinator), &steps[index + 1..]))
		}
		None if !steps.is_empty() => Some((&[], None, steps)),
		None => None,
	}
}

fn is_last_child(frame: &Frame) -> bool {
	let Some(tail) = frame.siblings.get(frame.node_index + 1..) else { return true };
	tail.iter().all(|node| node.element().is_none())
}

fn is_only_child(frame: &Frame) -> bool {
	frame.element_index == 0 && is_last_child(frame)
}

fn nth_expr_matches(frame: &Frame, expr: &selector::NthExpr) -> bool {
	let child_index = frame.element_index as i32 + 1;
	expr.eval(child_index)
}

fn nth_last_expr_matches(frame: &Frame, expr: &selector::NthExpr) -> bool {
	// TODO: Inefficient to count all siblings for every element
	let total_children = frame.siblings.iter().filter(|node| node.element().is_some()).count() as i32;
	let child_index = total_children - frame.element_index as i32;
	expr.eval(child_index)
}

fn filter_matches<'a, 'dom>(frame: &Frame<'a, 'dom>, filter: &selector::Filter) -> bool {
	use selector::Filter::{self, *};
	let element = frame.element;
	match filter {
		&Universal => true,
		&Tag(tag) => Filter::tag_equals(element.tag, tag),
		&Class(value) => Filter::attr_word(element.get_attribute_value("class").as_deref(), value),
		&Id(id) => element.id == Some(id),
		&AttrExists(name) => element.get_attribute(name).is_some(),
		&AttrEquals { name, value } => Filter::attr_equals(element.get_attribute_value(name).as_deref(), value),
		&AttrWord { name, value } => Filter::attr_word(element.get_attribute_value(name).as_deref(), value),
		&AttrHyphen { name, value } => Filter::attr_hyphen(element.get_attribute_value(name).as_deref(), value),
		&AttrStartsWith { name, value } => Filter::attr_starts_with(element.get_attribute_value(name).as_deref(), value),
		&AttrEndsWith { name, value } => Filter::attr_ends_with(element.get_attribute_value(name).as_deref(), value),
		&AttrContains { name, value } => Filter::attr_contains(element.get_attribute_value(name).as_deref(), value),
		&Empty => element.is_empty(),
		&FirstChild => frame.element_index == 0,
		&LastChild => is_last_child(frame),
		&OnlyChild => is_only_child(frame),
		&NthChild(ref expr) => nth_expr_matches(frame, expr),
		&NthLastChild(ref expr) => nth_last_expr_matches(frame, expr),
	}
}

fn selectors_match<'a, 'dom>(frame: &Frame<'a, 'dom>, selectors: &[selector::Step]) -> bool {
	use selector::Step::*;
	selectors.iter().all(|step| match step {
		#[cfg(debug_assertions)]
		SelectorList => unreachable!(),
		#[cfg(not(debug_assertions))]
		SelectorList => true,
		#[cfg(debug_assertions)]
		Combinator(_) => unreachable!(),
		#[cfg(not(debug_assertions))]
		Combinator(_) => true,
		Filter(filter) => filter_matches(frame, filter),
	})
}

fn check_element<'a, 'dom>(frame: &Frame<'a, 'dom>, ancestors: &[Frame<'a, 'dom>], steps: &[selector::Step]) -> bool {
	let Some((rest, combinator, selectors)) = split_last_compound(steps) else {
		return false;
	};

	if !selectors_match(frame, selectors) {
		return false;
	}

	match combinator {
		None => true,
		Some(selector::Combinator::Descendant) => descendant(ancestors, rest),
		Some(selector::Combinator::Child) => child(ancestors, rest),
		Some(selector::Combinator::NextSibling) => next_sibling(frame, ancestors, rest),
		Some(selector::Combinator::SubsequentSibling) => subsequent_sibling(frame, ancestors, rest),
	}
}

fn descendant<'a, 'dom>(ancestors: &[Frame<'a, 'dom>], rest: &[selector::Step<'_>]) -> bool {
	ancestors.iter().enumerate().rev().any(|(index, &parent)|
		check_element(&parent, &ancestors[..index], rest))
}

fn child<'a, 'dom>(ancestors: &[Frame<'a, 'dom>], rest: &[selector::Step<'_>]) -> bool {
	ancestors.split_last().is_some_and(|(&parent, rest_ancestors)|
		check_element(&parent, rest_ancestors, rest))
}

fn next_sibling<'a, 'dom>(frame: &Frame<'a, 'dom>, ancestors: &[Frame<'a, 'dom>], rest: &[selector::Step<'_>]) -> bool {
	previous_elements(frame).next().is_some_and(|sibling|
		check_element(&sibling, ancestors, rest))
}

fn subsequent_sibling<'a, 'dom>(frame: &Frame<'a, 'dom>, ancestors: &[Frame<'a, 'dom>], rest: &[selector::Step<'_>]) -> bool {
	previous_elements(frame).any(|sibling|
		check_element(&sibling, ancestors, rest))
}

/// Iterates over the previous sibling elements of the given frame, starting with the immediately preceding sibling.
fn previous_elements<'a, 'dom>(frame: &Frame<'a, 'dom>) -> impl Iterator<Item = Frame<'a, 'dom>> {
	let mut node_index = frame.node_index;
	let mut element_index = frame.element_index;

	iter::from_fn(move || {
		// Should never be out of bounds if the frame is valid
		#[cfg(not(debug_assertions))]
		if node_index >= frame.siblings.len() {
			return None;
		}

		while node_index > 0 {
			node_index -= 1;
			let Some(element) = frame.siblings[node_index].element() else { continue };
			element_index -= 1;
			return Some(Frame { node_index, element_index, element, siblings: frame.siblings });
		}

		None
	})
}
