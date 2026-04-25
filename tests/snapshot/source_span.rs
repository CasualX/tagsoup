use super::*;

#[test]
fn it_can_generate_source_span() {
	let html = "\
<template>
	<h1>Header</h1>
	<p>Paragraph</p>
</template>";
	let dom = tagsoup::Document::parse(html).trimmed();
	assert_debug_snapshot!(dom);
}
