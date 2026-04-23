use super::*;

#[test]
fn test_valid_char() {
	assert!(is_valid_name_start_char("é".as_bytes()));
}

fn assert_tokens(input: &str, expected: &[(TokenKind, &str)]) {
	println!("\nInput: {:?}", input);
	let mut lexer = Lexer::new(input.as_bytes());
	for (i, &(expect_kind, expect_text)) in expected.iter().enumerate() {
		let token = lexer.next().unwrap_or_else(|| panic!("expected token {}: {:?}", i, expected));
		let text = &input[token.span.range()];
		println!("Token {}: kind={:?}, text={:?}", i, token.kind, text);
		assert_eq!(token.kind, expect_kind, "token {} kind mismatch", i);
		assert_eq!(text, expect_text, "token {} text mismatch", i);
	}
	let extra_token = lexer.next();
	assert!(extra_token.is_none(), "expected no more tokens, but got {:?}", extra_token);
}

#[test]
fn test_smoke() {
	let input = "hi<!--ok--><?pi a='1' b=two ?><!DOCTYPE html \"sys\"><![CDATA[x<y]]><div bare x=\"y\" z=w/></div><img src=foo />";
	assert_tokens(input, &[
		(TokenKind::Text, "hi"),
		(TokenKind::Comment, "<!--ok-->"),
		(TokenKind::PIOpen, "pi"),
		(TokenKind::AttrName, "a"),
		(TokenKind::AttrValue, "'1'"),
		(TokenKind::AttrName, "b"),
		(TokenKind::AttrValue, "two"),
		(TokenKind::PIClose, "?>"),
		(TokenKind::DocTypeOpen, "DOCTYPE"),
		(TokenKind::DocTypeValue, "html"),
		(TokenKind::DocTypeValue, "\"sys\""),
		(TokenKind::DocTypeClose, ">"),
		(TokenKind::CData, "<![CDATA[x<y]]>"),
		(TokenKind::TagOpen, "div"),
		(TokenKind::AttrName, "bare"),
		(TokenKind::AttrName, "x"),
		(TokenKind::AttrValue, "\"y\""),
		(TokenKind::AttrName, "z"),
		(TokenKind::AttrValue, "w/"),
		(TokenKind::TagClose, ">"),
		(TokenKind::EndTagOpen, "div"),
		(TokenKind::TagClose, ">"),
		(TokenKind::TagOpen, "img"),
		(TokenKind::AttrName, "src"),
		(TokenKind::AttrValue, "foo"),
		(TokenKind::TagSelfClose, "/>"),
	]);
}

#[test]
fn test_error_paths() {
	let input = "<div \0bad><?pi \0><!DOCTYPE \0>";
	assert_tokens(input, &[
		(TokenKind::TagOpen, "div"),
		(TokenKind::Error, "\0bad"),
		(TokenKind::TagClose, ">"),
		(TokenKind::PIOpen, "pi"),
		(TokenKind::Error, "\0"),
		(TokenKind::TagClose, ">"),
		(TokenKind::DocTypeOpen, "DOCTYPE"),
		(TokenKind::Error, "\0"),
		(TokenKind::DocTypeClose, ">"),
	]);
}

#[test]
fn test_whitespace() {
	assert_tokens("", &[]);
	assert_tokens("\t\n   ", &[(TokenKind::Text, "\t\n   ")]);
	assert_tokens("  \n\t  <a>", &[
		(TokenKind::Text, "  \n\t  "),
		(TokenKind::TagOpen, "a"),
		(TokenKind::TagClose, ">"),
	]);
}

#[test]
fn test_plain_text_in_tagsoup() {
	assert_tokens("hello world", &[(TokenKind::Text, "hello world")]);
	assert_tokens("hello <b>world", &[
		(TokenKind::Text, "hello "),
		(TokenKind::TagOpen, "b"),
		(TokenKind::TagClose, ">"),
		(TokenKind::Text, "world"),
	]);
}

#[test]
fn test_processing_instructions() {
	assert_tokens("<?xml version=\"1.0\" encoding='utf-8'?>", &[
		(TokenKind::PIOpen, "xml"),
		(TokenKind::AttrName, "version"),
		(TokenKind::AttrValue, "\"1.0\""),
		(TokenKind::AttrName, "encoding"),
		(TokenKind::AttrValue, "'utf-8'"),
		(TokenKind::PIClose, "?>"),
	]);
	assert_tokens("<?pi  k = 'v'  ?>", &[
		(TokenKind::PIOpen, "pi"),
		(TokenKind::AttrName, "k"),
		(TokenKind::AttrValue, "'v'"),
		(TokenKind::PIClose, "?>"),
	]);
	assert_tokens("<?pi x='unterminated", &[
		(TokenKind::PIOpen, "pi"),
		(TokenKind::AttrName, "x"),
		(TokenKind::AttrValue, "'unterminated"),
	]);
}

#[test]
fn test_process_instructions_args() {
	assert_tokens("<?pi1?><?pi2 v?><?pi3 v=1?><?pi4 v='1'?>", &[
		(TokenKind::PIOpen, "pi1"),
		(TokenKind::PIClose, "?>"),
		(TokenKind::PIOpen, "pi2"),
		(TokenKind::AttrName, "v"),
		(TokenKind::PIClose, "?>"),
		(TokenKind::PIOpen, "pi3"),
		(TokenKind::AttrName, "v"),
		(TokenKind::AttrValue, "1?"),
		(TokenKind::TagClose, ">"),
		(TokenKind::PIOpen, "pi4"),
		(TokenKind::AttrName, "v"),
		(TokenKind::AttrValue, "'1'"),
		(TokenKind::PIClose, "?>"),
	]);
}

#[test]
fn test_processing_instruction_errors() {
	assert_tokens("<?pi \0?>", &[
		(TokenKind::PIOpen, "pi"),
		(TokenKind::Error, "\0?"),
		(TokenKind::TagClose, ">"),
	]);
	assert_tokens("<?pi \0?>tail", &[
		(TokenKind::PIOpen, "pi"),
		(TokenKind::Error, "\0?"),
		(TokenKind::TagClose, ">"),
		(TokenKind::Text, "tail"),
	]);
	assert_tokens("<?pi \0>tail", &[
		(TokenKind::PIOpen, "pi"),
		(TokenKind::Error, "\0"),
		(TokenKind::TagClose, ">"),
		(TokenKind::Text, "tail"),
	]);
}

#[test]
fn test_smoke_tag_and_attributes() {
	assert_tokens("<foo bar=\"baz\" @id='x'/>", &[
		(TokenKind::TagOpen, "foo"),
		(TokenKind::AttrName, "bar"),
		(TokenKind::AttrValue, "\"baz\""),
		(TokenKind::AttrName, "@id"),
		(TokenKind::AttrValue, "'x'"),
		(TokenKind::TagSelfClose, "/>"),
	]);
	assert_tokens("<script src=/link-fixup.js defer=''>", &[
		(TokenKind::TagOpen, "script"),
		(TokenKind::AttrName, "src"),
		(TokenKind::AttrValue, "/link-fixup.js"),
		(TokenKind::AttrName, "defer"),
		(TokenKind::AttrValue, "''"),
		(TokenKind::TagClose, ">"),
	]);
}

#[test]
fn test_tag_identifiers_allowed_special_chars() {
	assert_tokens(r#"<x foo._baz:qux-123_@$="ok">"#, &[
		(TokenKind::TagOpen, "x"),
		(TokenKind::AttrName, "foo._baz:qux-123_@$"),
		(TokenKind::AttrValue, "\"ok\""),
		(TokenKind::TagClose, ">"),
	]);
}

#[test]
fn test_closing_tag_inputs() {
	assert_tokens("</root>", &[
		(TokenKind::EndTagOpen, "root"),
		(TokenKind::TagClose, ">"),
	]);
	assert_tokens("</root>  ", &[
		(TokenKind::EndTagOpen, "root"),
		(TokenKind::TagClose, ">"),
		(TokenKind::Text, "  "),
	]);
	assert_tokens("</  root>", &[
		(TokenKind::Text, "</"),
		(TokenKind::Text, "  root>"),
	]);
	assert_tokens("</  root>  ", &[
		(TokenKind::Text, "</"),
		(TokenKind::Text, "  root>  "),
	]);
}

#[test]
fn test_attr_unterminated_errors() {
	assert_tokens("<a x=\"unterminated", &[
		(TokenKind::TagOpen, "a"),
		(TokenKind::AttrName, "x"),
		(TokenKind::AttrValue, "\"unterminated"),
	]);
	assert_tokens("<a \0>", &[
		(TokenKind::TagOpen, "a"),
		(TokenKind::Error, "\0"),
		(TokenKind::TagClose, ">"),
	]);
	assert_tokens("<a \0>tail", &[
		(TokenKind::TagOpen, "a"),
		(TokenKind::Error, "\0"),
		(TokenKind::TagClose, ">"),
		(TokenKind::Text, "tail"),
	]);
}

#[test]
fn test_doctype_inputs() {
	assert_tokens("<!DOCTYPE html>", &[
		(TokenKind::DocTypeOpen, "DOCTYPE"),
		(TokenKind::DocTypeValue, "html"),
		(TokenKind::DocTypeClose, ">"),
	]);
	assert_tokens("<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\">", &[
		(TokenKind::DocTypeOpen, "DOCTYPE"),
		(TokenKind::DocTypeValue, "html"),
		(TokenKind::DocTypeValue, "PUBLIC"),
		(TokenKind::DocTypeValue, "\"-//W3C//DTD XHTML 1.0 Transitional//EN\""),
		(TokenKind::DocTypeClose, ">"),
	]);
	assert_tokens("<!x \0>tail", &[
		(TokenKind::DocTypeOpen, "x"),
		(TokenKind::Error, "\0"),
		(TokenKind::DocTypeClose, ">"),
		(TokenKind::Text, "tail"),
	]);
}

#[test]
fn test_doctype_dtd() {
	assert_tokens("<!DOCTYPE html [<!ELEMENT test ANY>]>", &[
		(TokenKind::DocTypeOpen, "DOCTYPE"),
		(TokenKind::DocTypeValue, "html"),
		(TokenKind::DocTypeSubsetOpen, "["),
		(TokenKind::DocTypeOpen, "ELEMENT"),
		(TokenKind::DocTypeValue, "test"),
		(TokenKind::DocTypeValue, "ANY"),
		(TokenKind::DocTypeClose, ">"),
		(TokenKind::DocTypeSubsetClose, "]"),
		(TokenKind::DocTypeClose, ">"),
	]);
}

#[test]
fn test_comments() {
	assert_tokens("<!--   hello world   -->", &[(TokenKind::Comment, "<!--   hello world   -->")]);
	assert_tokens("<!---->", &[(TokenKind::Comment, "<!---->")]);
	assert_tokens("<!--->", &[(TokenKind::Comment, "<!--->")]);
	assert_tokens("<!-->", &[(TokenKind::Comment, "<!-->")]);
	assert_tokens("<!--   -->", &[(TokenKind::Comment, "<!--   -->")]);
	assert_tokens("<!--ab-cd-->", &[(TokenKind::Comment, "<!--ab-cd-->")]);
	assert_tokens("<!-- abc", &[(TokenKind::Comment, "<!-- abc")]);
}

#[test]
fn test_cdata() {
	assert_tokens("<![CDATA[  x < y  ]]>", &[(TokenKind::CData, "<![CDATA[  x < y  ]]>")]);
	assert_tokens("<![CDATA[abc]def]]>", &[(TokenKind::CData, "<![CDATA[abc]def]]>")]);
	assert_tokens("<![CDATA[abc", &[(TokenKind::CData, "<![CDATA[abc")]);
}

#[test]
fn test_text_with_bare_less_than() {
	assert_tokens("1 < 2", &[
		(TokenKind::Text, "1 "),
		(TokenKind::Text, "<"),
		(TokenKind::Text, " 2"),
	]);
	assert_tokens("a << b", &[
		(TokenKind::Text, "a "),
		(TokenKind::Text, "<"),
		(TokenKind::Text, "<"),
		(TokenKind::Text, " b"),
	]);
}

#[test]
fn test_unquoted_attr_value() {
	assert_tokens("<img src=x/>", &[
		(TokenKind::TagOpen, "img"),
		(TokenKind::AttrName, "src"),
		(TokenKind::AttrValue, "x/"),
		(TokenKind::TagClose, ">"),
	]);
	assert_tokens("<img src=x?>", &[
		(TokenKind::TagOpen, "img"),
		(TokenKind::AttrName, "src"),
		(TokenKind::AttrValue, "x?"),
		(TokenKind::TagClose, ">"),
	]);
	assert_tokens("<img src=http://example.test/logo.svg/>", &[
		(TokenKind::TagOpen, "img"),
		(TokenKind::AttrName, "src"),
		(TokenKind::AttrValue, "http://example.test/logo.svg/"),
		(TokenKind::TagClose, ">"),
	]);
}

#[test]
fn test_unicode_xml_names() {
	assert_tokens("<é attr=1>", &[
		(TokenKind::TagOpen, "é"),
		(TokenKind::AttrName, "attr"),
		(TokenKind::AttrValue, "1"),
		(TokenKind::TagClose, ">"),
	]);
	assert_tokens("</é>", &[
		(TokenKind::EndTagOpen, "é"),
		(TokenKind::TagClose, ">"),
	]);
}

#[test]
fn test_unterminated_constructs() {
	assert_tokens("<!-- unterminated", &[(TokenKind::Comment, "<!-- unterminated")]);
	assert_tokens("<![CDATA[unterminated", &[(TokenKind::CData, "<![CDATA[unterminated")]);
	assert_tokens("<a x='unterminated", &[
		(TokenKind::TagOpen, "a"),
		(TokenKind::AttrName, "x"),
		(TokenKind::AttrValue, "'unterminated"),
	]);
}

#[test]
fn test_raw_text_without_helper() {
	// You're expected to use the raw_text helper for <script> and <style> tags
	assert_tokens("<script>if (a < b) x = 1;</script>", &[
		(TokenKind::TagOpen, "script"),
		(TokenKind::TagClose, ">"),
		(TokenKind::Text, "if (a "),
		(TokenKind::Text, "<"),
		(TokenKind::Text, " b) x = 1;"),
		(TokenKind::EndTagOpen, "script"),
		(TokenKind::TagClose, ">"),
	]);
}

#[test]
fn test_raw_text_helper() {
	let input = "if (a < b) x = 1;</ScRiPt >tail";
	let mut lexer = Lexer::new(input.as_bytes());
	let span = lexer.raw_text(b"script");
	assert_eq!(&input[span.range()], "if (a < b) x = 1;");
	assert_eq!(lexer.position, "if (a < b) x = 1;".len());

	let input = "body</scriptx>tail";
	let mut lexer = Lexer::new(input.as_bytes());
	let span = lexer.raw_text(b"script");
	assert_eq!(&input[span.range()], "body</scriptx>tail");
	assert_eq!(lexer.position, input.len());
}
