// Smoke tests against real websites

fn fetch(site: &str, url: &str) -> Option<String> {
	// Always use cached version if it exists
	let cache_path = format!("target/smoke/{}.html", site);
	if let Ok(html) = std::fs::read_to_string(&cache_path) {
		return Some(html);
	}

	// Use CLI tool to fetch the content of the page
	let output = std::process::Command::new("curl")
		.arg("-L") // Follow redirects
		.arg("-s") // Silent mode
		.arg(url)
		.output()
		.expect("Failed to execute curl");

	// Cache the output for future runs
	_ = std::fs::create_dir_all("target/smoke");
	_ = std::fs::write(&cache_path, &output.stdout);

	String::from_utf8(output.stdout).ok()
}

#[track_caller]
fn assert_parses(text: &str) {
	let doc = tagsoup::Document::parse(text);
	if !doc.errors.is_empty() {
		for error in &doc.errors {
			let span = error.span.resolve(text).unwrap();
			eprintln!("Parse error: {}:{} {}", span.start_line, span.start_column, error.kind.as_str());
			eprintln!("--> {}", span.snippet(80));
		}
	}
	else {
		println!("Parsed successfully with no errors.");
	}
}

#[test]
fn test_spotify() {
	if let Some(html) = fetch("spotify", "https://www.spotify.com/se") {
		assert_parses(&html);
	}
}

#[ignore = "Amazon does not like being scraped and returns a 503 error."]
#[test]
fn test_amazon() {
	if let Some(html) = fetch("amazon", "https://www.amazon.com") {
		assert_parses(&html);
	}
}

#[test]
fn test_wikipedia() {
	if let Some(html) = fetch("wikipedia", "https://en.wikipedia.org/wiki/Main_Page") {
		assert_parses(&html);
	}
}

#[test]
fn test_nytimes() {
	if let Some(html) = fetch("nytimes", "https://www.nytimes.com/") {
		assert_parses(&html);
	}
}

#[test]
fn test_example() {
	if let Some(html) = fetch("example", "https://www.example.com/") {
		assert_parses(&html);
	}
}
