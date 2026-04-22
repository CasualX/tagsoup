use super::*;

/// Doctype node in the DOM tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DoctypeNode<'a> {
	/// The keyword of the doctype.
	///
	/// This is `DOCTYPE` in `<!DOCTYPE html>`.
	pub keyword: &'a str,

	/// Span of the doctype keyword in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub keyword_span: SourceSpan,

	/// Arguments of the doctype.
	///
	/// This is `html` in `<!DOCTYPE html>`.
	pub args: Vec<AttributeValue<'a>>,

	/// Doctype declarations inside the doctype.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
	pub dtd: Vec<DoctypeNode<'a>>,

	/// Span of the doctype in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub span: SourceSpan,
}
