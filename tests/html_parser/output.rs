use super::*;

#[test]
fn it_can_output_json() {
	assert!(serde_json::to_string(&tagsoup::Document::parse("<div/>")).is_ok());
}

#[test]
fn it_can_output_json_pretty() {
	assert!(serde_json::to_string_pretty(&tagsoup::Document::parse("<div/>")).is_ok());
}

#[test]
fn it_can_output_complex_html_as_json() {
	let html = 
		"<html lang=\"sv\">
		<head>
			<title>Här kan man va</title>
		</head>
			<body>
				<h1>Tjena världen!</h1>
				<p>Tänkte bara informera om att Sverige är bättre än Finland i ishockey.</p>
			</body>
		</html>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_json_snapshot!(dom);
}
