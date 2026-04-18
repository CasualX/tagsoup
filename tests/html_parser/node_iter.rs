
#[test]
fn it_can_iter_1() {
	let html = "
		<html>
			<head>
				<title>title</title>
			</head>
			<body>
				<ul>
					<li></li>
					<li></li>
					<li></li>
				</ul>
			</body>
		</html>
	";
	let dom = tagsoup::Document::parse(&html);
	let mut num_li = 0;
	dom.visit(&mut |element| {
		if element.tag == "li" {
			num_li += 1;
		}
		tagsoup::VisitControl::Descend
	});
	assert_eq!(num_li, 3);
}
