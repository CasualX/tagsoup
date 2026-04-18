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

fn bench_host(bencher: &mut Bencher, host: &str) {
	let html = load_smoke_output(host);
	bencher.bytes = html.len() as u64;
	bencher.iter(|| {
		black_box(tagsoup::Document::parse(black_box(html.as_str())));
	});
}

#[bench]
fn spotify(bencher: &mut Bencher) {
	bench_host(bencher, "spotify");
}

#[ignore = "Amazon smoke output is only available when the ignored smoke test has been run."]
#[bench]
fn amazon(bencher: &mut Bencher) {
	bench_host(bencher, "amazon");
}

#[bench]
fn wikipedia(bencher: &mut Bencher) {
	bench_host(bencher, "wikipedia");
}

#[bench]
fn nytimes(bencher: &mut Bencher) {
	bench_host(bencher, "nytimes");
}

#[bench]
fn example(bencher: &mut Bencher) {
	bench_host(bencher, "example");
}
