use super::*;

/// Text node in the DOM tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TextNode<'a> {
	/// The text content of the text node.
	pub text: &'a str,

	/// Span of the text node in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub span: SourceSpan,
}

/// Comment node in the DOM tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CommentNode<'a> {
	/// The text content of the comment node.
	pub comment: &'a str,

	/// Span of the comment node in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub span: SourceSpan,
}

/// Doctype node in the DOM tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DoctypeNode<'a> {
	/// The name of the doctype.
	///
	/// This is `DOCTYPE` in `<!DOCTYPE html>`.
	pub name: &'a str,

	/// Span of the doctype name in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub name_span: SourceSpan,

	/// Attributes of the doctype.
	///
	/// This is `html` in `<!DOCTYPE html>`.
	pub attributes: Vec<Attribute<'a>>,

	/// Span of the doctype in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub span: SourceSpan,
}

/// Processing instruction node in the DOM tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ProcessingInstructionNode<'a> {
	/// The target of the processing instruction.
	///
	/// This is `xml` in `<?xml version="1.0"?>`.
	pub target: &'a str,

	/// Span of the processing instruction target in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub target_span: SourceSpan,

	/// The data of the processing instruction.
	///
	/// This is `version="1.0"` in `<?xml version="1.0"?>`.
	pub data: Vec<Attribute<'a>>,

	/// Span of the processing instruction in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub span: SourceSpan,
}

/// Node in the DOM tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Node<'a> {
	Text(TextNode<'a>),
	Element(Element<'a>),
	Comment(CommentNode<'a>),
	Doctype(DoctypeNode<'a>),
	ProcessingInstruction(ProcessingInstructionNode<'a>),
}

impl<'a> Node<'a> {
	#[inline]
	pub fn text(&self) -> Option<&TextNode<'a>> {
		match self {
			Node::Text(t) => Some(t),
			_ => None,
		}
	}

	#[inline]
	pub fn element(&self) -> Option<&Element<'a>> {
		match self {
			Node::Element(e) => Some(e),
			_ => None,
		}
	}

	#[inline]
	pub fn comment(&self) -> Option<&CommentNode<'a>> {
		match self {
			Node::Comment(t) => Some(t),
			_ => None,
		}
	}

	#[inline]
	pub fn doctype(&self) -> Option<&DoctypeNode<'a>> {
		match self {
			Node::Doctype(d) => Some(d),
			_ => None,
		}
	}

	#[inline]
	pub fn processing_instruction(&self) -> Option<&ProcessingInstructionNode<'a>> {
		match self {
			Node::ProcessingInstruction(pi) => Some(pi),
			_ => None,
		}
	}
}
