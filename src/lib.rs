/*!
TagSoup is a small, fast, fairly forgiving HTML-ish parser with zero required dependencies.

It is built for the boringly useful jobs:

- Parse real-world markup without immediately fainting.
- Walk the resulting tree.
- Query it with a compact CSS-style selector API.
- Pull out text, attributes, and spans.

It is not trying to impersonate a browser engine. It just wants to turn messy markup into something workable, quickly.

Loosely based on the [HTML Living Standard](https://html.spec.whatwg.org/multipage/syntax.html).

# Highlights

- Optional `serde` support, enabled by default.
- Preserves source spans for nodes and parse errors.
- Handles raw-text elements like `script` and `style` sensibly.
- Supports `query_selector` and `query_selector_all`.
- Supports tree walking with a small visitor API.
- Tries to recover from malformed markup instead of giving up immediately.

# Examples

```
// Parse an HTML tag soup.
let doc = tagsoup::Document::parse("<div><p id=here>Hello, world!</p></div>");

// Check for parsing errors.
assert!(doc.errors.is_empty());

// Query the document for an element using a CSS selector.
let element = doc.query_selector("#here").unwrap();
assert_eq!(element.text_content(), "Hello, world!");
```

# Querying The Tree

```
let doc = tagsoup::Document::parse(r#"
	<article id="main">
		<p class="lead">Hello</p>
		<p data-kind="feature card">world</p>
	</article>
"#);

assert_eq!(doc.query_selector("#main .lead").unwrap().text_content(), "Hello");
assert_eq!(doc.query_selector_all("[data-kind*=feature]").len(), 1);
```

# Notes

- Whitespace is preserved by default.
- Call [`Document::trimmed`] if you want leading and trailing ASCII whitespace removed from text nodes.
- [`Element::text_content`] decodes HTML entities, except inside raw-text elements like `script` and `style`.
- Invalid selectors currently panic in [`Document::query_selector`] and [`Document::query_selector_all`].

This is not a full WHATWG-compliant HTML parser. It is a pragmatic parser for documents that are mostly HTML, occasionally cursed, and still need to be dealt with.
*/

use std::collections::HashMap;
use std::borrow::Cow;
use std::{error, fmt, iter, mem, str};

#[macro_use]
mod known;

mod attribute;
mod tag;

pub use attribute::*;
pub use tag::*;


mod element;
mod entity;
mod errors;
mod selector;
mod span;
mod utils;
mod visit;

pub use element::*;
pub use errors::*;
pub use span::*;
pub use utils::*;
pub use visit::*;

pub mod lexer;
mod dom_ref;
pub use dom_ref::*;

#[cfg(debug_assertions)]
#[inline]
fn unsafe_as_str(bytes: &[u8]) -> &str {
	str::from_utf8(bytes).unwrap()
}
#[cfg(not(debug_assertions))]
#[inline]
fn unsafe_as_str(bytes: &[u8]) -> &str {
	unsafe { str::from_utf8_unchecked(bytes) }
}
