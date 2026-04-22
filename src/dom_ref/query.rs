use super::*;

// We keep the containing slice with each frame so sibling combinators can stay
// local to one parent boundary instead of reconstructing that context later.
#[derive(Copy, Clone)]
struct Frame<'a, 'dom> {
	element: &'dom Element<'a>,
	siblings: &'dom [Node<'a>],
}

impl<'a, 'dom> Frame<'a, 'dom> {
	fn with_element(self, element: &'dom Element<'a>) -> Self {
		Self { element, siblings: self.siblings }
	}
}

// Matching from the right lets us decide whether the current element is even a
// candidate before we pay to inspect ancestors or siblings.
fn split_last_compound<'step>(
	steps: &'step [selector::Step<'step>],
) -> Option<(
	&'step [selector::Step<'step>],
	Option<&'step selector::Combinator>,
	&'step [selector::Step<'step>],
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

pub fn query<'a, 'dom>(children: &'dom [Node<'a>], steps: &[selector::Step], result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool) {
	query_in(children, &mut Vec::new(), steps, result_fn);
}

fn query_in<'a, 'dom>(
	children: &'dom [Node<'a>],
	ancestors: &mut Vec<Frame<'a, 'dom>>,
	steps: &[selector::Step],
	result_fn: &mut dyn FnMut(&'dom Element<'a>) -> bool,
) -> bool {
	for child in children {
		let Node::Element(element) = child else { continue };
		let frame = Frame { element, siblings: children };

		if check_element(frame, ancestors, steps) && !result_fn(element) {
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

fn selectors_match<'a, 'dom>(element: &'dom Element<'a>, selectors: &[selector::Step]) -> bool {
	use selector::Step::*;
	use selector::Filter::*;
	selectors.iter().all(|step| match step {
		Combinator(_) => unreachable!(),
		&Filter(Tag(filter_tag)) => element.tag.eq_ignore_ascii_case(filter_tag),
		&Filter(Class(filter_class)) => element.get_attribute_value("class").map_or(false, |class_list| class_list.split_ascii_whitespace().any(|class| class == filter_class)),
		&Filter(Id(filter_id)) => element.id == Some(filter_id),
		&Filter(AttrExists(filter_name)) => element.get_attribute(filter_name).is_some(),
		&Filter(AttrEquals { name, value }) => element.get_attribute_value(name).map_or(false, |v| v == value),
		&Filter(AttrContains { name, value }) => element.get_attribute_value(name).map_or(false, |v| v.contains(value)),
		&Filter(AttrStartsWith { name, value }) => element.get_attribute_value(name).map_or(false, |v| v.starts_with(value)),
		&Filter(AttrEndsWith { name, value }) => element.get_attribute_value(name).map_or(false, |v| v.ends_with(value)),
		&Filter(AttrWhitespaceContains { name, value }) => element.get_attribute_value(name).map_or(false, |v| v.split_ascii_whitespace().any(|part| part == value)),
	})
}

fn previous_element_siblings<'a, 'dom>(frame: Frame<'a, 'dom>) -> impl Iterator<Item = &'dom Element<'a>> + 'dom {
	let index = frame.siblings.iter()
		// Pointer identity is enough here because traversal hands us references from
		// the exact sibling slice we are searching within.
		.position(|node| matches!(node, Node::Element(candidate) if std::ptr::eq(candidate, frame.element)))
		.expect("current element must be present in its sibling slice");

	frame.siblings[..index].iter().rev().filter_map(|node| node.element())
}

fn check_element<'a, 'dom>(frame: Frame<'a, 'dom>, ancestors: &[Frame<'a, 'dom>], steps: &[selector::Step]) -> bool {
	let Some((rest, combinator, selectors)) = split_last_compound(steps) else {
		return false;
	};

	if !selectors_match(frame.element, selectors) {
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
	ancestors.iter().enumerate().rev().any(|(index, &parent)| check_element(parent, &ancestors[..index], rest))
}

fn child<'a, 'dom>(ancestors: &[Frame<'a, 'dom>], rest: &[selector::Step<'_>]) -> bool {
	ancestors.split_last().is_some_and(|(&parent, rest_ancestors)| check_element(parent, rest_ancestors, rest))
}

fn next_sibling<'a, 'dom>(frame: Frame<'a, 'dom>, ancestors: &[Frame<'a, 'dom>], rest: &[selector::Step<'_>]) -> bool {
	// Adjacent-sibling matching stays under the same parent, so only the current
	// element changes while the ancestor stack and sibling slice stay fixed.
	previous_element_siblings(frame).next().is_some_and(|sibling| check_element(frame.with_element(sibling), ancestors, rest))
}

fn subsequent_sibling<'a, 'dom>(frame: Frame<'a, 'dom>, ancestors: &[Frame<'a, 'dom>], rest: &[selector::Step<'_>]) -> bool {
	previous_element_siblings(frame).any(|sibling| check_element(frame.with_element(sibling), ancestors, rest))
}
