#![feature(test)] // Run with `cargo +nightly bench` and ignore the error feature not allowed on stable channel

extern crate test;

use std::fs;
use std::path::PathBuf;

use test::{black_box, Bencher};

fn load_smoke_output(host: &str) -> String {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("smoke").join(format!("{host}.html"));

	fs::read_to_string(&path).unwrap_or_else(|error| {
		panic!("missing smoke output for {host} at {} ({error}); run `cargo test --test smoke` first", path.display());
	})
}

fn bench_parse(bencher: &mut Bencher, host: &str) {
	let html = load_smoke_output(host);
	bencher.bytes = html.len() as u64;
	bencher.iter(|| {
		black_box(tagsoup::Document::parse(black_box(html.as_str())));
	});
}

#[bench]
fn parse_spotify(bencher: &mut Bencher) {
	bench_parse(bencher, "spotify");
}

#[ignore = "Amazon smoke output is only available when the ignored smoke test has been run."]
#[bench]
fn parse_amazon(bencher: &mut Bencher) {
	bench_parse(bencher, "amazon");
}

#[bench]
fn parse_wikipedia(bencher: &mut Bencher) {
	bench_parse(bencher, "wikipedia");
}

#[bench]
fn parse_nytimes(bencher: &mut Bencher) {
	bench_parse(bencher, "nytimes");
}

#[bench]
fn parse_example(bencher: &mut Bencher) {
	bench_parse(bencher, "example");
}

fn bench_lexer(bencher: &mut Bencher, host: &str) {
	let html = load_smoke_output(host);
	bencher.bytes = html.len() as u64;
	bencher.iter(|| {
		black_box(tagsoup::Lexer::new(black_box(html.as_str().as_bytes())).count());
	});
}

#[bench]
fn lexer_spotify(bencher: &mut Bencher) {
	bench_lexer(bencher, "spotify");
}

#[ignore = "Amazon smoke output is only available when the ignored smoke test has been run."]
#[bench]
fn lexer_amazon(bencher: &mut Bencher) {
	bench_lexer(bencher, "amazon");
}

#[bench]
fn lexer_wikipedia(bencher: &mut Bencher) {
	bench_lexer(bencher, "wikipedia");
}

#[bench]
fn lexer_nytimes(bencher: &mut Bencher) {
	bench_lexer(bencher, "nytimes");
}

#[bench]
fn lexer_example(bencher: &mut Bencher) {
	bench_lexer(bencher, "example");
}
