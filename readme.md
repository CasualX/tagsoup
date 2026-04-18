TagSoup
=======

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/tagsoup.svg)](https://crates.io/crates/tagsoup)
[![docs.rs](https://docs.rs/tagsoup/badge.svg)](https://docs.rs/tagsoup)
[![Build status](https://github.com/CasualX/tagsoup/workflows/Check/badge.svg)](https://github.com/CasualX/tagsoup/actions)

[TagSoup](https://en.wikipedia.org/wiki/Tag_soup) is a simple HTML parser written in Rust.

It is however, very fast.

It attempts to parse HTML tag soup as best as it can.
It is not a full HTML parser, but it should be able to handle most HTML documents.

Loosly based on [HTML Living Standard](https://html.spec.whatwg.org/multipage/syntax.html).

Usage
-----

```rust
// Parse an HTML fragment.
let doc = tagsoup::Document::parse("<div><p id=here>Hello, world!</p></div>");

// Check for parsing errors.
assert!(doc.errors.is_empty());

// Query the document for an element using a CSS selector.
let element = doc.query_selector("#here").unwrap();
assert_eq!(element.text_content(), "Hello, world!");
```

📜 License
----------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
