use super::*;

#[test]
fn it_can_parse_one_element() {
	let html = "<html></html>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_one_element_upper_case() {
	let html = "<HTML></HTML>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_one_element_mixed_case() {
	let html = "<Html></Html>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_one_element_mixed_case_numbers() {
	let html = "<Header1></Header1>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_one_element_mixed_case_numbers_symbols() {
	let html = "<Head_Er-1></Head_Er-1>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_multiple_elements() {
	let html = "<div/><div/>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_multiple_open_elements() {
	let html = "<div></div><div></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_nested_elements() {
	let html = r"
		<div>
			<div />
		</div>
	";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_nested_elements_mixed_children() {
	let html = r"
		<div>
			<!--comment-->
			<div/>
			Hello
			<div>
				World
			</div>
		</div>
	";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_deeply_nested() {
	let html = r#"
			<div class='1'>
				<div class='1'>
					<div class='1'>
						<div class='1'>
							<div class='1'>
								<div class='1'>
									<div class='1'>
										<div class='1'>
											<!--this is deep-->
											hello world
										</div>
									</div>
								</div>
							</div> 
						</div>
					</div>
				</div>
			</div>
		"#;
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_script_with_content() {
	let html = r#"
<script>
	const person_creator = ({ name, symtoms }) => {
		let person = {}
		person.name = name
		person.symtoms = {}
		for (symtom of symtoms) {
			person.symtoms[symtom] = true
		}
		return person
	}

	const main = () => {
		let name = 'mathias'
		let symtoms = ['Dunning-Kruger', 'ACDC', 'Slacker']

		setTimeout(() => {
			let person = person_creator({ name, symtoms })
			if (person.symtoms.hasOwnProperty('Dunning-Kruger')) {
				console.log('yeah buddy, that\'s right')
			}
		}, 1337)
	}

	main()
</script>"#;
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_style_with_content() {
	let html = r#"
<style>
	:root {
		--background-color: black;
		--text-color: white;
	}
	body {
		background: var(--background-color);
		color: var(--text-color);
	}
</style>"#;
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_skips_dangling_elements() {
	let html = "
		<div id='123'></div>
		</div>
		<div id='321'></div>
	";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_parse_broken_html() {
	let html = "<div></span><div></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_errors_when_multiple_nested_elements_dont_match() {
	let html = "<div><div><div><div></div></div_error></div></div>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}

#[test]
fn it_can_clone_node() {
	let html = "
		<div>one</div>
		<div>two</div>
	";
	let dom = tagsoup::Document::parse(html).trimmed();
	let one = dom.children[0].clone();
	assert_json_snapshot!(one);
}

#[test]
fn it_can_clone_dom() {
	let html = "
		<html>
			<head>
				<title>Title</title>
			</head>
			<body>
				<h1>Hello world</h1>
			</body>
		</html>
	";
	let dom = tagsoup::Document::parse(html).trimmed();
	let dom_clone = dom.clone();
	assert_eq!(dom, dom_clone);
}

#[test]
fn it_can_deal_with_weird_whitespaces() {
	let html = "
		<!-- Normal case -->
		<div> Text </div>

		<!-- Whitespaces in opening tag to the left -->
		< div> Text </div>

		<!-- Whitespaces in opening tag to the right -->
		<div > Text </div>

		<!-- Whitespaces in closing tag to the left (should not work) -->
		<div> Text < /div>

		<!-- Whitespaces in closing tag to the right -->
		<div> Text </div >

		<!-- Whitespaces everywhere (should not work) -->
		< div > Text < / div >
	";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}