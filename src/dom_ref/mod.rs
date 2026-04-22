use super::*;

mod attributes;
// mod dfs;
mod doctype;
mod document;
mod element;
mod nodes;
mod parser;
mod query;

pub use attributes::*;
pub use doctype::*;
pub use document::*;
pub use element::*;
pub use nodes::*;
use query::*;

fn visit<'a, 'dom>(
	children: &'dom [Node<'a>],
	parents: &mut Vec<&'dom Element<'a>>,
	visitor: &mut dyn FnMut(&[&'dom Element<'a>], &'dom Element<'a>) -> VisitControl,
) -> bool {
	for child in children {
		if let Node::Element(element) = child {
			match visitor(parents, element) {
				VisitControl::Descend => {}
				VisitControl::Continue => continue,
				VisitControl::Stop => return false,
			}

			parents.push(element);
			let result = visit(&element.children, parents, visitor);
			parents.pop();

			if !result {
				return false;
			}
		}
	}
	return true;
}

#[cfg(test)]
mod tests;
