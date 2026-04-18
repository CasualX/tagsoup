use super::*;

/// Document parse error kinds.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub enum ParseErrorKind {
	InvalidAttribute,
	InvalidAttributeValue,
	InvalidAttributeTag,
	SelfClosingRawTextElement,
	UnterminatedComment,
	UnterminatedCData,
	UnterminatedDoctype,
	UnterminatedProcessingInstruction,
	UnclosedElement,
	UnterminatedTag,
	MissingTagName,
	SelfClosingEndTag,
	UnexpectedToken,
}

impl ParseErrorKind {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::InvalidAttribute => "invalid attribute",
			Self::InvalidAttributeValue => "invalid attribute value",
			Self::InvalidAttributeTag => "invalid attribute tag",
			Self::SelfClosingRawTextElement => "raw text elements cannot be self-closing",
			Self::UnterminatedComment => "unterminated comment",
			Self::UnterminatedCData => "unterminated CDATA section",
			Self::UnterminatedDoctype => "unterminated doctype",
			Self::UnterminatedProcessingInstruction => "unterminated processing instruction",
			Self::UnclosedElement => "unclosed element",
			Self::UnterminatedTag => "unterminated tag",
			Self::MissingTagName => "missing tag name",
			Self::SelfClosingEndTag => "end tags cannot be self-closing",
			Self::UnexpectedToken => "unexpected token",
		}
	}
}

/// Document parse error.
#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
	pub span: Span,
	pub kind: ParseErrorKind,
}

impl std::error::Error for ParseError {
	fn description(&self) -> &str {
		self.kind.as_str()
	}
}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}:{} {}", self.span.start, self.span.end, self.kind.as_str())
	}
}
