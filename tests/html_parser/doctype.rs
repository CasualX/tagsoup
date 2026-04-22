use super::*;

#[test]
fn valid_xhtml() {
	let text = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">"#;
	let doc = tagsoup::Document::parse(text);
	assert_json_snapshot!(doc);
}

#[test]
fn doctype_internal() {
	let text = r#"
		<?xml version="1.0" standalone="yes"?>
		<!DOCTYPE tutorials [
			<!ELEMENT tutorials (tutorial)+>
			<!ELEMENT tutorial (name,url)>
			<!ELEMENT name (#PCDATA)>
			<!ELEMENT url (#PCDATA)>
			<!ATTLIST tutorials type CDATA #REQUIRED>
		]>
		<tutorials type="web">
		<tutorial>
			<name>XML Tutorial</name>
			<url>https://www.quackit.com/xml/tutorial</url>
		</tutorial>
		<tutorial>
			<name>HTML Tutorial</name>
			<url>https://www.quackit.com/html/tutorial</url>
		</tutorial>
		</tutorials>
	"#;
	let doc = tagsoup::Document::parse(text);
	assert_json_snapshot!(doc);
}
