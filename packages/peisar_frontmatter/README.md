# peisar_frontmatter

A small, focused Rust crate for parsing optional YAML front matter from Markdown documents and deserializing it into any serde-enabled type.

This crate extracts a YAML front matter block delimited by `---` and returns a typed representation of the front matter (if present) together with the Markdown body with the front matter removed.

Highlights

- Parses YAML front matter and deserializes it using serde/serde_yaml
- Returns the Markdown body with the front matter removed
- Generic over any `T: serde::de::DeserializeOwned`
- Minimal API surface — one parsing function and a small result type

Quick example

```rust
use peisar_frontmatter::parse_markdown_frontmatter;

#[derive(serde::Deserialize, Debug)]
struct Frontmatter {
    title: String,
}

fn main() {
    let content = "---\ntitle: Hello\n---\n# Welcome\n";

    match parse_markdown_frontmatter::<Frontmatter>(content) {
        Ok(parsed) => {
            if let Some(fm) = parsed.yaml_data() {
                println!("title: {}", fm.title);
            }
            println!("markdown body:\n{}", parsed.pure_markdown_content());
        }
        Err(e) => eprintln!("parse error: {}", e),
    }
}
```

API

- parse_markdown_frontmatter<T>(content: &str) -> Result<ParseResult<T>, String>
  - T: `serde::de::DeserializeOwned`
  - Returns a `ParseResult<T>` on success or a `String` describing the parse error.

- ParseResult<T>
  - `yaml_data(&self) -> Option<&T>` — returns deserialized front matter if present
  - `pure_markdown_content(&self) -> &str` — returns the Markdown body with front matter removed

Behavior notes

- Leading whitespace is ignored before checking for front matter.
- A front matter block is recognized when the document (after trimming leading whitespace) begins with `---` (LF or CRLF). The end of the YAML block is the first occurrence of a newline followed by `---`.
- If no front matter is found, the original document is returned as the Markdown body and `yaml_data()` is `None`.
- YAML parsing errors return an Err(String) with a message starting with `Failed to parse YAML front matter:`.

Examples

Parsing a file on disk:

```rust
use std::fs;
use peisar_frontmatter::parse_markdown_frontmatter;

#[derive(serde::Deserialize, Debug)]
struct Frontmatter { title: String }

fn parse_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let parsed = parse_markdown_frontmatter::<Frontmatter>(&content)?;

    if let Some(fm) = parsed.yaml_data() {
        println!("title: {}", fm.title);
    }
    println!("body:\n{}", parsed.pure_markdown_content());
    Ok(())
}
```

Development and tests

- From the repository root (workspace):

```sh
cargo test -p peisar_frontmatter
```

- Or from the package directory:

```sh
cd packages/peisar_frontmatter
cargo test
```

Repository files

- [Cargo.toml](/home/ptm/Github/PEISAR/peisar-mono-repo/packages/peisar_frontmatter/Cargo.toml)
- [lib source](/home/ptm/Github/PEISAR/peisar-mono-repo/packages/peisar_frontmatter/src/lib.rs)
- [LICENSE](/home/ptm/Github/PEISAR/peisar-mono-repo/packages/peisar_frontmatter/LICENSE)

Contributing

Contributions are welcome. Please:

1. Open an issue to discuss larger changes or enhancements.
2. Send a small, focused pull request that includes tests for behavior changes.

License

See the [LICENSE](/home/ptm/Github/PEISAR/peisar-mono-repo/packages/peisar_frontmatter/LICENSE) file in this package.

