use super::*;

/// Value of an attribute.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AttributeValue<'a> {
	/// The value of the attribute, including quotes if they exist.
	pub value: &'a str,

	/// Span of the attribute value in the parsed source, including quotes if they exist.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub span: SourceSpan,
}

impl<'a> AttributeValue<'a> {
	/// Gets the raw value of the attribute, removing just the outer quotes if they exist.
	pub fn value_raw(&self) -> &'a str {
		// Unquote the value if it's quoted
		if self.value.len() < 2 {
			return self.value;
		}
		let bytes = self.value.as_bytes();
		let first = bytes[0];
		let last = bytes[bytes.len() - 1];
		if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
			unsafe_as_str(&bytes[1..bytes.len() - 1])
		}
		else {
			self.value
		}
	}
	/// Gets the decoded value of the attribute, decoding HTML entities.
	pub fn value(&self) -> Cow<'a, str> {
		let unquoted = self.value_raw();
		if unquoted.contains('&') {
			Cow::Owned(entity::decode_entities(unquoted))
		}
		else {
			Cow::Borrowed(unquoted)
		}
	}
}

/// Attribute of an element.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Attribute<'a> {
	/// The key of the attribute.
	pub key: &'a str,

	/// The value of the attribute, if it exists.
	#[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_attr_value"))]
	pub value: Option<AttributeValue<'a>>,

	/// Span of the attribute key in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub key_span: SourceSpan,

	/// Span of the attribute in the parsed source.
	#[cfg_attr(feature = "serde", serde(skip))]
	pub span: SourceSpan,
}

#[cfg(feature = "serde")]
fn serialize_attr_value<'a, S: serde::Serializer>(
	value: &Option<AttributeValue<'a>>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	match value {
		Some(attr_value) => serializer.serialize_some(&attr_value.value_raw()),
		None => serializer.serialize_none(),
	}
}
