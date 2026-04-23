use std::fmt;
use std::io::Read;

const RESET: &str = "\x1b[0m";
const TREE_GUIDE: &str = "\x1b[38;5;244m";
const DOCTYPE_KEYWORD: &str = "\x1b[38;5;246m";
const DOCTYPE_ARGS: &str = "\x1b[38;5;80m";
const PI_TARGET: &str = "\x1b[38;5;246m";
const PI_ARGS: &str = "\x1b[38;5;80m";
const TEXT_LABEL: &str = "\x1b[38;5;242m";
const COMMENT_LABEL: &str = "\x1b[38;5;114m";
const TEXT_VALUE: &str = "\x1b[38;5;255m";
const TAG_NAME: &str = "\x1b[38;5;141m";
const ATTR_NAME: &str = "\x1b[38;5;255m";
const ATTR_VALUE: &str = "\x1b[38;5;111m";
const ATTR_PUNCT: &str = "\x1b[38;5;250m";
const ERROR_LABEL: &str = "\x1b[38;5;203m";
const ERROR_LOCATION: &str = "\x1b[38;5;250m";
const ERROR_KIND: &str = "\x1b[38;5;223m";
const ERROR_SNIPPET: &str = "\x1b[38;5;255m";
const QUERY_LABEL: &str = "\x1b[38;5;215m";
const QUERY_VALUE: &str = "\x1b[38;5;230m";
const MATCH_LABEL: &str = "\x1b[38;5;120m";
const MATCH_META: &str = "\x1b[38;5;249m";

const TREE_BRANCH: &str = "├─";
const TREE_LAST: &str = "└─";
const TREE_CONTINUE: &str = "│ ";
const TREE_SPACE: &str = "  ";

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum OutputFormat {
	Json,
	JsonPretty,
	Query,
	Tree,
}

impl OutputFormat {
	fn parse(value: &str) -> Self {
		match value {
			"json" => Self::Json,
			"json-pretty" => Self::JsonPretty,
			"query" => Self::Query,
			"tree" => Self::Tree,
			_ => unreachable!("clap validated the format argument"),
		}
	}
}

fn main() {
	let matches = clap::Command::new("tagsoup")
		.about("Parse HTML and render it")
		.version(clap::crate_version!())
		.arg(clap::Arg::new("format")
			.long("format")
			.short('f')
			.value_name("FORMAT")
			.value_parser(["json", "json-pretty", "query", "tree"])
			.help("Optional output format: json, json-pretty, query, or tree"))
		.arg(clap::Arg::new("input")
			.long("input")
			.short('i')
			.value_name("PATH")
			.help("Read HTML from a file instead of stdin"))
		.arg(clap::Arg::new("selector")
			.value_name("SELECTOR")
			.required_if_eq("format", "query")
			.help("CSS selector used with -fquery"))
		.arg(clap::Arg::new("trimmed")
			.long("trimmed")
			.help("Trim whitespace")
			.action(clap::ArgAction::SetTrue))
		.get_matches();

	let input = read_input(matches.get_one::<String>("input").map(String::as_str));

	let format = matches.get_one::<String>("format").map(|value| OutputFormat::parse(value.as_str()));
	let selector = matches.get_one::<String>("selector").map(String::as_str);
	let trimmed = matches.get_flag("trimmed");

	if selector.is_some() && format != Some(OutputFormat::Query) {
		eprintln!("selector positional argument requires -fquery");
		std::process::exit(2);
	}

	let mut document = tagsoup::Document::parse(&input);
	if trimmed {
		document = document.trimmed();
	}

	match format {
		Some(OutputFormat::Json) => println!("{}", serde_json::to_string(&document).unwrap()),
		Some(OutputFormat::JsonPretty) => println!("{}", serde_json::to_string_pretty(&document).unwrap()),
		Some(OutputFormat::Query) => print_query_matches(&document, selector.expect("clap should require selector for query format"), &input),
		Some(OutputFormat::Tree) => print_tree(&document),
		None => {}
	}

	if print_errors(&document, &input) {
		std::process::exit(1);
	}
}

fn read_input(path: Option<&str>) -> String {
	match path {
		Some(path) => match std::fs::read_to_string(path) {
			Ok(input) => input,
			Err(err) => {
				eprintln!("failed to read {path:?}: {err}");
				std::process::exit(1);
			}
		},
		None => {
			let mut input = String::new();
			if let Err(err) = std::io::stdin().read_to_string(&mut input) {
				eprintln!("failed to read stdin: {err}");
				std::process::exit(1);
			}
			input
		}
	}
}

fn print_tree(document: &tagsoup::Document<'_>) {
	let mut prefix = String::new();
	print_nodes(&document.children, &mut prefix);
}

fn print_query_matches(document: &tagsoup::Document<'_>, query: &str, input: &str) {
	let matches = document.query_selector_all(query);

	println!("{QUERY_LABEL}query{RESET}{ATTR_PUNCT}:{RESET} {QUERY_VALUE}{:?}{RESET}", query);
	println!("{QUERY_LABEL}matches{RESET}{ATTR_PUNCT}:{RESET} {QUERY_VALUE}{}{RESET}", matches.len());

	for (index, element) in matches.iter().enumerate() {
		print_query_match(index + 1, element, input);
	}
}

fn print_nodes(nodes: &[tagsoup::Node<'_>], prefix: &mut String) {
	for (index, node) in nodes.iter().enumerate() {
		let is_last = index + 1 == nodes.len();
		print_node(node, prefix, is_last);
	}
}

fn print_node(node: &tagsoup::Node<'_>, prefix: &mut String, is_last: bool) {
	let branch = if is_last { TREE_LAST } else { TREE_BRANCH };
	println!("{TREE_GUIDE}{prefix}{branch}{RESET}{}", format_node(node));

	if let tagsoup::Node::Element(element) = node {
		let prefix_len = prefix.len();
		prefix.push_str(if is_last { TREE_SPACE } else { TREE_CONTINUE });
		print_nodes(&element.children, prefix);
		prefix.truncate(prefix_len);
	}
}

fn format_node(node: &tagsoup::Node<'_>) -> impl fmt::Display {
	fmt::from_fn(|f| match node {
		tagsoup::Node::Text(text) => write!(f, "{TEXT_LABEL}#text{RESET}{ATTR_PUNCT}: {TEXT_VALUE}{:?}{RESET}", text.text),
		tagsoup::Node::Comment(comment) => write!(f, "{COMMENT_LABEL}#comment{RESET}{ATTR_PUNCT}: {TEXT_VALUE}{:?}{RESET}", comment.comment),
		tagsoup::Node::Doctype(doctype) => write!(f, "{}", format_doctype(doctype)),
		tagsoup::Node::ProcessingInstruction(pi) => write!(f, "{}", format_processing_instruction(pi)),
		tagsoup::Node::Element(element) => write!(f, "{}", format_element(element)),
	})
}

fn format_element(element: &tagsoup::Element<'_>) -> impl fmt::Display {
	fmt::from_fn(move |f| {
		write!(f, "{TAG_NAME}{}{RESET}", element.tag)?;
		write_attributes(f, &element.attributes, ATTR_NAME, ATTR_VALUE)
	})
}

fn print_query_match(index: usize, element: &tagsoup::Element<'_>, input: &str) {
	match element.tag_span.resolve(input) {
		Some(span) => println!("{MATCH_LABEL}match #{index}{RESET} {MATCH_META}@ {}:{}{RESET} {ATTR_PUNCT}|{RESET} {}", span.start_line, span.start_column + 1, format_element(element)),
		None => println!("{MATCH_LABEL}match #{index}{RESET} {ATTR_PUNCT}|{RESET} {}", format_element(element)),
	}
}

fn format_doctype(doctype: &tagsoup::DoctypeNode<'_>) -> impl fmt::Display {
	fmt::from_fn(move |f| {
		write!(f, "{DOCTYPE_KEYWORD}{}{RESET}", doctype.keyword)?;
		if !doctype.args.is_empty() {
			write!(f, "{ATTR_PUNCT}:{RESET}")?;
			write_doctype_args(f, &doctype.args, DOCTYPE_ARGS)?;
		}
		Ok(())
	})
}

fn format_processing_instruction(pi: &tagsoup::ProcessingInstructionNode<'_>) -> impl fmt::Display {
	fmt::from_fn(move |f| {
		write!(f, "{PI_TARGET}?{}{RESET}", pi.target)?;
		write_attributes(f, &pi.data, PI_ARGS, PI_ARGS)
	})
}

fn write_attributes(f: &mut fmt::Formatter<'_>, attributes: &[tagsoup::Attribute<'_>], key_color: &str, value_color: &str) -> fmt::Result {
	for attr in attributes {
		write!(f, " {key_color}{}{RESET}", attr.key)?;
		if let Some(value) = &attr.value {
			write!(f, "{ATTR_PUNCT}={RESET}{value_color}{:?}{RESET}", value.value)?;
		}
	}
	Ok(())
}

fn write_doctype_args(f: &mut fmt::Formatter<'_>, args: &[tagsoup::AttributeValue<'_>], value_color: &str) -> fmt::Result {
	for arg in args {
		write!(f, " {value_color}{:?}{RESET}", arg.value)?;
	}
	Ok(())
}

fn print_errors(document: &tagsoup::Document<'_>, input: &str) -> bool {
	for error in &document.errors {
		let span = error.span.resolve(input).expect("error span should be resolvable in the original input");
		let snippet = span.snippet(80);
		let snippet = if snippet.is_empty() { "<empty>" } else { snippet };
		eprintln!(
			"{ERROR_LABEL}error{RESET} {ERROR_LOCATION}[{}:{}]{RESET} {ERROR_KIND}{}{RESET} {ERROR_LOCATION}|{RESET} {ERROR_SNIPPET}{:?}{RESET}",
			span.start_line,
			span.start_column,
			error.kind.as_str(),
			snippet,
		);
	}

	!document.errors.is_empty()
}
