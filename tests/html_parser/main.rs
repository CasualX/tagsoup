//! Tests kindly borrowed from https://github.com/mathiversen/html-parser

#![allow(non_snake_case)]

use std::{any, fs, path};

fn assert_snapshot(fn_name: &str, mut value: String) {
	let file_name = fn_name.replace(":", "_");
	let snapshot_path = path::PathBuf::from(format!("tests/html_parser/snapshots/{file_name}.snap"));

	value.push_str("\n");

	if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
		if let Some(parent) = snapshot_path.parent() {
			fs::create_dir_all(parent).unwrap();
		}
		fs::write(&snapshot_path, value).unwrap();
		return;
	}

	let expected = fs::read_to_string(&snapshot_path).unwrap_or_else(|err| {
		panic!("snapshot file not found for {fn_name} ({err}). Run with UPDATE_SNAPSHOTS=1 to create it.")
	});

	let a = expected.find("---").map_or(0, |i| i + 3);
	let b = a + expected[a..].find("---").map_or(0, |i| i + 4);
	let expected = &expected[b..];

	if expected != value {
		panic!("snapshot mismatch for {fn_name}");
	}
}

macro_rules! function_name {
	() => {{
		#[allow(dead_code)]
		fn f() {}
		let fn_name = any::type_name_of_val(&f);
		let fn_name = fn_name.strip_suffix("::f").unwrap();
		fn_name.split_once("::").map(|(_, name)| name).unwrap_or(fn_name)
	}};
}

macro_rules! assert_json_snapshot {
	($value:expr) => {
		assert_snapshot(function_name!(), serde_json::to_string_pretty(&$value).unwrap());
	}
}

#[allow(unused_macros)]
macro_rules! assert_debug_snapshot {
	($value:expr) => {
		assert_snapshot(function_name!(), format!("{:#?}", $value));
	}
}

mod comments;
mod document;
mod document_empty;
mod document_fragment;
mod element;
mod element_attributes;
mod node_iter;
mod output;
mod source_span;
mod svg;
mod text;
