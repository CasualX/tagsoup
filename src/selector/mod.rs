use super::*;

pub mod parser;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Combinator {
	Descendant,
	Child,
	NextSibling,
	SubsequentSibling,
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

#[cfg(test)]
mod tests;
