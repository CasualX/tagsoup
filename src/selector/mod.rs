use super::*;

pub mod parser;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Combinator {
	Descendant,
	Child,
	NextSibling,
	SubsequentSibling,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Step<'a> {
	SelectorList,
	Combinator(Combinator),
	Filter(Filter<'a>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NthExpr {
	Formula { a: i32, b: i32 },
	Even,
	Odd,
}
impl NthExpr {
	/// Evaluates the expression for a given child index (starting from 1).
	pub fn eval(&self, child_index: i32) -> bool {
		let (a, b) = match self {
			&NthExpr::Formula { a, b } => (a, b),
			&NthExpr::Even => (2, 0),
			&NthExpr::Odd => (2, 1),
		};
		if a == 0 {
			child_index == b
		}
		else if a > 0 {
			child_index >= b && (child_index - b) % a == 0
		}
		else {
			child_index <= b && (child_index - b) % a == 0
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter<'a> {
	/// `*`
	///
	/// Matches all elements.
	Universal,

	/// `tag`
	///
	/// Matches elements with the given tag name (case-insensitive).
	Tag(&'a str),

	/// `.class`
	///
	/// Matches elements with the given class in their `class` attribute.
	Class(&'a str),

	/// `#id`
	///
	/// Matches elements with the given id in their `id` attribute.
	Id(&'a str),

	/// `[attr]`
	///
	/// Matches elements with the given attribute.
	AttrExists(&'a str),

	/// `[attr=value]`
	///
	/// Matches elements with the given attribute and value.
	AttrEquals { name: &'a str, value: &'a str },

	/// `[attr~=value]`
	///
	/// Matches elements with the given attribute containing the specified word.
	AttrWord { name: &'a str, value: &'a str },

	/// `[attr|=value]`
	///
	/// Matches elements with the given attribute starting with the specified value followed by a hyphen.
	AttrHyphen { name: &'a str, value: &'a str },

	/// `[attr^=value]`
	///
	/// Matches elements with the given attribute starting with the specified value.
	AttrStartsWith { name: &'a str, value: &'a str },

	/// `[attr$=value]`
	///
	/// Matches elements with the given attribute ending with the specified value.
	AttrEndsWith { name: &'a str, value: &'a str },

	/// `[attr*=value]`
	///
	/// Matches elements with the given attribute containing the specified value.
	AttrContains { name: &'a str, value: &'a str },

	/// `:empty`
	///
	/// Matches elements that have no children.
	Empty,

	/// `:first-child`
	///
	/// Matches elements that are the first child of their parent.
	FirstChild,

	/// `:last-child`
	///
	/// Matches elements that are the last child of their parent.
	LastChild,

	/// `:only-child`
	///
	/// Matches elements that are the only child of their parent.
	OnlyChild,

	/// `:nth-child(<An+B> | even | odd)`
	///
	/// Matches elements that are the nth child of their parent, counting from the first child.
	/// `An+B` expressions are supported, where `A` and `B` are integers and `n` is a variable representing the index of the child (starting from 1).
	/// The keywords `even` and `odd` can also be used as shortcuts for `2n` and `2n+1`, respectively.
	NthChild(NthExpr),

	/// `:nth-last-child(<An+B> | even | odd)`
	///
	/// Matches elements that are the nth child of their parent, counting from the last child.
	/// `An+B` expressions are supported, where `A` and `B` are integers and `n` is a variable representing the index of the child (starting from 1).
	/// The keywords `even` and `odd` can also be used as shortcuts for `2n` and `2n+1`, respectively.
	NthLastChild(NthExpr),
}

impl Filter<'_> {
	#[inline]
	pub fn tag_equals(value: &str, tag: &str) -> bool {
		value.eq_ignore_ascii_case(tag)
	}
	#[inline]
	pub fn attr_word(value: Option<&str>, expected: &str) -> bool {
		value.map_or(false, |value| value.split_ascii_whitespace().any(|part| part == expected))
	}
	#[inline]
	pub fn attr_equals(value: Option<&str>, expected: &str) -> bool {
		value.map_or(false, |value| value == expected)
	}
	#[inline]
	pub fn attr_hyphen(value: Option<&str>, expected: &str) -> bool {
		value.map_or(false, |v| v.strip_prefix(expected).map_or(false, |suffix| matches!(suffix.bytes().next(), Some(b'-') | None)))
	}
	#[inline]
	pub fn attr_starts_with(value: Option<&str>, expected: &str) -> bool {
		value.map_or(false, |v| v.starts_with(expected))
	}
	#[inline]
	pub fn attr_ends_with(value: Option<&str>, expected: &str) -> bool {
		value.map_or(false, |v| v.ends_with(expected))
	}
	#[inline]
	pub fn attr_contains(value: Option<&str>, expected: &str) -> bool {
		value.map_or(false, |v| v.contains(expected))
	}
}

#[cfg(test)]
mod tests;
