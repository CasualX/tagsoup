TagSoup
=======

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/tagsoup.svg)](https://crates.io/crates/tagsoup)
[![docs.rs](https://docs.rs/tagsoup/badge.svg)](https://docs.rs/tagsoup)
[![Build status](https://github.com/CasualX/tagsoup/workflows/Check/badge.svg)](https://github.com/CasualX/tagsoup/actions)

[TagSoup](https://en.wikipedia.org/wiki/Tag_soup) is a small, fast, fairly forgiving HTML-ish parser written in Rust.

It is built for the boringly useful jobs:

- Parse real-world markup without immediately fainting.
- Walk the resulting tree.
- Query it with a compact CSS-style selector API.
- Pull out text, attributes, and spans.

It is not trying to impersonate a browser engine. It just wants to turn messy markup into something workable, quickly.

Loosely based on the [HTML Living Standard](https://html.spec.whatwg.org/multipage/syntax.html).

Features
--------

- Zero required dependencies.
- Optional `serde` support, enabled by default.
- Preserves source spans for nodes and parse errors.
- Handles raw-text elements like `script` and `style` sensibly.
- Supports `query_selector` and `query_selector_all`.
- Supports tree walking with a small visitor API.
- Tries to recover from malformed markup instead of giving up immediately.

Installation
------------

```toml
[dependencies]
tagsoup = "0.1.1"
```

If you want to keep it dependency-free all the way down:

```toml
[dependencies]
tagsoup = { version = "0.1.1", default-features = false }
```

Usage
-----

```rust
// Parse an HTML tag soup.
let doc = tagsoup::Document::parse("<div><p id=here>Hello, world!</p></div>");

// Check for parsing errors.
assert!(doc.errors.is_empty());

// Query the document for an element using a CSS selector.
let element = doc.query_selector("#here").unwrap();
assert_eq!(element.text_content(), "Hello, world!");
```

If you want to collect data, use the query API:

```rust
let doc = tagsoup::Document::parse(r#"
	<article id="main">
		<p class="lead">Hello</p>
		<p data-kind="feature card">world</p>
	</article>
"#);

assert_eq!(doc.query_selector("#main .lead").unwrap().text_content(), "Hello");
assert_eq!(doc.query_selector_all("[data-kind*=feature]").len(), 1);
```

Selector Support
----------------

The selector engine is intentionally compact, but it covers the selectors you usually want for scraping and document inspection:

- Tag selectors: `div`
- ID selectors: `#main`
- Class selectors: `.hero`
- Attribute presence: `[href]`
- Attribute equality: `[lang=en]`
- Attribute contains: `[data-kind*=feature]`
- Attribute prefix and suffix: `[src^=http]`, `[src$=.png]`
- Whitespace-separated attribute matching: `[rel~=nofollow]`
- Descendant, child, and sibling combinators: `article .lead`, `ul > li`, `h2 + p`, `h2 ~ p`

Invalid selectors currently panic in `query_selector` and `query_selector_all`, so if the selector is user input, validate or sanitize it first.

Parsing Notes
-------------

- Whitespace is preserved by default.
- Call `trimmed()` if you want leading and trailing ASCII whitespace removed from text nodes.
- `text_content()` decodes HTML entities, except inside raw-text elements like `script` and `style`.
- Parse errors are collected in `document.errors` instead of stopping the parse.
- Errors include source spans, so reporting decent diagnostics is straightforward.

There is also a tiny example CLI that reads HTML from stdin and dumps JSON:

```bash
cargo run --example tagsoup -- --pretty < page.html
```

What This Is Not
----------------

- Not a browser DOM implementation.
- Not a full WHATWG-compliant HTML parser.
- Not trying to perfectly reproduce browser error recovery in every bizarre corner case.

It is a pragmatic parser for documents that are mostly HTML, occasionally cursed, and still need to be dealt with.

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
