use super::super::*;
use TokenKind::*;

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
	assert_tokens("\t\n   ", &[Text("\t\n   ")]);
	assert_tokens("  \n\t  <a>", &[Text("  \n\t  "), TagOpen, Ident("a"), TagClose]);
}

#[test]
fn plain_text_in_tagsoup() {
	assert_tokens("hello world", &[Text("hello world")]);
	assert_tokens("hello <b>world", &[Text("hello "), TagOpen, Ident("b"), TagClose, Text("world")]);
}

#[test]
fn processing_instructions() {
	assert_tokens("<?xml version=\"1.0\" encoding='utf-8'?>", &[PIOpen, Ident("xml"), Ident("version"), Equals, Quoted("1.0"), Ident("encoding"), Equals, Quoted("utf-8"), PIClose]);
	assert_tokens("<?pi  k = 'v'  ?>", &[PIOpen, Ident("pi"), Ident("k"), Equals, Quoted("v"), PIClose]);
	assert_tokens("<?pi x='unterminated", &[PIOpen, Ident("pi"), Ident("x"), Equals, Quoted("unterminated")]);

}

#[test]
fn pi_error() {
	assert_tokens("<?pi \0?>", &[PIOpen, Ident("pi"), Error("\0"), PIClose]);
	assert_tokens("<?pi \0?>tail", &[PIOpen, Ident("pi"), Error("\0"), PIClose, Text("tail")]);
	assert_tokens("<?pi \0>tail", &[PIOpen, Ident("pi"), Error("\0"), TagClose, Text("tail")]);
}

#[test]
fn lexes_regular_tag_attributes_and_self_close() {
	assert_tokens("<foo bar=\"baz\" @id='x'/>", &[TagOpen, Ident("foo"), Ident("bar"), Equals, Quoted("baz"), Ident("@id"), Equals, Quoted("x"), SelfTagClose]);
	assert_tokens("<script src=/link-fixup.js defer=''>", &[TagOpen, Ident("script"), Ident("src"), Equals, Ident("/link-fixup.js"), Ident("defer"), Equals, Quoted(""), TagClose]);
}

#[test]
fn lexes_tag_identifiers_with_allowed_special_chars() {
	assert_tokens(r#"<x foo._baz:qux-123_@$="ok">"#, &[TagOpen, Ident("x"), Ident("foo._baz:qux-123_@$"), Equals, Quoted("ok"), TagClose]);
}

#[test]
fn closing_tag() {
	assert_tokens("</root>", &[SelfTagOpen, Ident("root"), TagClose]);
	assert_tokens("</  root>", &[SelfTagOpen, Ident("root"), TagClose]);
}

#[test]
fn attr_unterminated() {
	assert_tokens("<a x=\"unterminated", &[TagOpen, Ident("a"), Ident("x"), Equals, Quoted("unterminated")]);
}

#[test]
fn attr_errors() {
	assert_tokens("<a \0>", &[TagOpen, Ident("a"), Error("\0"), TagClose]);
	assert_tokens("<a \0>tail", &[TagOpen, Ident("a"), Error("\0"), TagClose, Text("tail")]);
}

#[test]
fn doctype() {
	assert_tokens("<!DOCTYPE html>", &[DoctypeOpen, Ident("DOCTYPE"), Ident("html"), TagClose]);
	assert_tokens("<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\">", &[DoctypeOpen, Ident("DOCTYPE"), Ident("html"), Ident("PUBLIC"), Quoted("-//W3C//DTD XHTML 1.0 Transitional//EN"), TagClose]);
	assert_tokens("<!x \0>tail", &[DoctypeOpen, Ident("x"), Error("\0"), TagClose, Text("tail")]);
}

#[test]
fn comments() {
	assert_tokens("<!--   hello world   -->", &[CommentOpen, Text("   hello world   "), CommentClose]);
	assert_tokens("<!---->", &[CommentOpen, CommentClose]);
	assert_tokens("<!--   -->", &[CommentOpen, Text("   "), CommentClose]);
	assert_tokens("<!--ab-cd-->", &[CommentOpen, Text("ab-cd"), CommentClose]);
	assert_tokens("<!-- abc", &[CommentOpen, Text(" abc")]);
}

#[test]
fn cdata() {
	assert_tokens("<![CDATA[  x < y  ]]>", &[CDataOpen, Text("  x < y  "), CDataClose]);
	assert_tokens("<![CDATA[abc]def]]>", &[CDataOpen, Text("abc]def"), CDataClose]);
	assert_tokens("<![CDATA[abc", &[CDataOpen, Text("abc")]);
}
