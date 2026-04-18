use super::*;

fn assert_tokens(input: &str, expected: &[TokenKind<'_>]) {
	let mut lexer = Lexer::new(input);
	for (i, &expected_kind) in expected.iter().enumerate() {
		let token = lexer.next().expect("Expected more tokens, but lexer returned None");
		println!("Token {}: {:?}", i, token);
		assert_eq!(token.kind, expected_kind, "Token {i}: expected {:?}, got {:?}", expected_kind, token.kind);
	}
	let extra_token = lexer.next();
	assert!(extra_token.is_none(), "Expected no more tokens, but lexer returned Some({:?})", extra_token);
}

#[test]
fn whitespace() {
	assert_tokens("", &[]);
	assert_tokens("\t\n   ", &[TokenKind::Text("\t\n   ")]);
	assert_tokens("  \n\t  <a>", &[TokenKind::Text("  \n\t  "), TokenKind::TagOpen, TokenKind::Ident("a"), TokenKind::TagClose]);
}

#[test]
fn plain_text_in_tagsoup() {
	assert_tokens("hello world", &[TokenKind::Text("hello world")]);
	assert_tokens("hello <b>world", &[TokenKind::Text("hello "), TokenKind::TagOpen, TokenKind::Ident("b"), TokenKind::TagClose, TokenKind::Text("world")]);
}

#[test]
fn processing_instructions() {
	assert_tokens("<?xml version=\"1.0\" encoding='utf-8'?>", &[
		TokenKind::PIOpen,
		TokenKind::Ident("xml"),
		TokenKind::Ident("version"),
		TokenKind::Equals,
		TokenKind::Quoted("1.0"),
		TokenKind::Ident("encoding"),
		TokenKind::Equals,
		TokenKind::Quoted("utf-8"),
		TokenKind::PIClose,
	]);
	assert_tokens("<?pi  k = 'v'  ?>", &[TokenKind::PIOpen, TokenKind::Ident("pi"), TokenKind::Ident("k"), TokenKind::Equals, TokenKind::Quoted("v"), TokenKind::PIClose]);
	assert_tokens("<?pi x='unterminated", &[TokenKind::PIOpen, TokenKind::Ident("pi"), TokenKind::Ident("x"), TokenKind::Equals, TokenKind::Quoted("unterminated")]);

}

#[test]
fn pi_error() {
	assert_tokens("<?pi \0?>", &[TokenKind::PIOpen, TokenKind::Ident("pi"), TokenKind::Error("\0"), TokenKind::PIClose]);
	assert_tokens("<?pi \0?>tail", &[TokenKind::PIOpen, TokenKind::Ident("pi"), TokenKind::Error("\0"), TokenKind::PIClose, TokenKind::Text("tail")]);
	assert_tokens("<?pi \0>tail", &[TokenKind::PIOpen, TokenKind::Ident("pi"), TokenKind::Error("\0"), TokenKind::TagClose, TokenKind::Text("tail")]);
}

#[test]
fn lexes_regular_tag_attributes_and_self_close() {
	assert_tokens("<foo bar=\"baz\" @id='x'/>", &[
		TokenKind::TagOpen,
		TokenKind::Ident("foo"),
		TokenKind::Ident("bar"),
		TokenKind::Equals,
		TokenKind::Quoted("baz"),
		TokenKind::Ident("@id"),
		TokenKind::Equals,
		TokenKind::Quoted("x"),
		TokenKind::SelfTagClose,
	]);
	assert_tokens("<script src=/link-fixup.js defer=''>", &[
		TokenKind::TagOpen,
		TokenKind::Ident("script"),
		TokenKind::Ident("src"),
		TokenKind::Equals,
		TokenKind::Ident("/link-fixup.js"),
		TokenKind::Ident("defer"),
		TokenKind::Equals,
		TokenKind::Quoted(""),
		TokenKind::TagClose,
	]);
}

#[test]
fn lexes_tag_identifiers_with_allowed_special_chars() {
	assert_tokens(r#"<x foo._baz:qux-123_@$="ok">"#, &[
		TokenKind::TagOpen, TokenKind::Ident("x"),
		TokenKind::Ident("foo._baz:qux-123_@$"),
		TokenKind::Equals, TokenKind::Quoted("ok"),
		TokenKind::TagClose,
	]);
}

#[test]
fn closing_tag() {
	assert_tokens("</root>", &[TokenKind::SelfTagOpen, TokenKind::Ident("root"), TokenKind::TagClose]);
	assert_tokens("</  root>", &[TokenKind::SelfTagOpen, TokenKind::Ident("root"), TokenKind::TagClose]);
}

#[test]
fn attr_unterminated() {
	assert_tokens("<a x=\"unterminated", &[TokenKind::TagOpen, TokenKind::Ident("a"), TokenKind::Ident("x"), TokenKind::Equals, TokenKind::Quoted("unterminated")]);
}

#[test]
fn attr_errors() {
	assert_tokens("<a \0>", &[TokenKind::TagOpen, TokenKind::Ident("a"), TokenKind::Error("\0"), TokenKind::TagClose]);
	assert_tokens("<a \0>tail", &[TokenKind::TagOpen, TokenKind::Ident("a"), TokenKind::Error("\0"), TokenKind::TagClose, TokenKind::Text("tail")]);
}

#[test]
fn doctype() {
	assert_tokens("<!DOCTYPE html>", &[TokenKind::DoctypeOpen, TokenKind::Ident("DOCTYPE"), TokenKind::Ident("html"), TokenKind::TagClose]);
	assert_tokens("<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\">", &[
		TokenKind::DoctypeOpen,
		TokenKind::Ident("DOCTYPE"),
		TokenKind::Ident("html"),
		TokenKind::Ident("PUBLIC"),
		TokenKind::Quoted("-//W3C//DTD XHTML 1.0 Transitional//EN"),
		TokenKind::TagClose,
	]);
	assert_tokens("<!x \0>tail", &[TokenKind::DoctypeOpen, TokenKind::Ident("x"), TokenKind::Error("\0"), TokenKind::TagClose, TokenKind::Text("tail")]);
}

#[test]
fn comments() {
	assert_tokens("<!--   hello world   -->", &[TokenKind::CommentOpen, TokenKind::Text("   hello world   "), TokenKind::CommentClose]);
	assert_tokens("<!---->", &[TokenKind::CommentOpen, TokenKind::CommentClose]);
	assert_tokens("<!--   -->", &[TokenKind::CommentOpen, TokenKind::Text("   "), TokenKind::CommentClose]);
	assert_tokens("<!--ab-cd-->", &[TokenKind::CommentOpen, TokenKind::Text("ab-cd"), TokenKind::CommentClose]);
	assert_tokens("<!-- abc", &[TokenKind::CommentOpen, TokenKind::Text(" abc")]);
}

#[test]
fn cdata() {
	assert_tokens("<![CDATA[  x < y  ]]>", &[TokenKind::CDataOpen, TokenKind::Text("  x < y  "), TokenKind::CDataClose]);
	assert_tokens("<![CDATA[abc]def]]>", &[TokenKind::CDataOpen, TokenKind::Text("abc]def"), TokenKind::CDataClose]);
	assert_tokens("<![CDATA[abc", &[TokenKind::CDataOpen, TokenKind::Text("abc")]);
}
