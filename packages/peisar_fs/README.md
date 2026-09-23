# peisar_fs

A small Rust crate for performing common filesystem operations relative to a
configured root directory.

## Highlights

- Create a `PeisarFs` helper rooted at a chosen directory.
- Check whether files or directories exist.
- Create nested directories.
- Write and read UTF-8 text files.
- Recursively find files, optionally filtered by extension.

## Quick example

```rust
use std::path::PathBuf;
use peisar_fs::PeisarFs;

fn main() {
    let filesystem = PeisarFs::new(Some(PathBuf::from("workspace")));

    filesystem.write_file("notes/today.txt", "Remember to ship it.");
    println!("{}", filesystem.read_file("notes/today.txt"));

    let text_files = filesystem.read_dir(".", Some("txt"));
    println!("text files: {text_files:?}");
}
```

Pass `None` to `PeisarFs::new` to use the process's current working directory:

```rust
use peisar_fs::PeisarFs;

let filesystem = PeisarFs::new(None);
filesystem.mkdir("build/output");
```

## API

### `PeisarFs::new(root)`

Creates a filesystem helper rooted at `root`. If `root` is `None`, the
process's current directory is used.

### `exixts(path)`

Returns `true` when `path` exists below the configured root. The method name is
`exixts` for compatibility with the current public API.

```rust
use peisar_fs::PeisarFs;

let filesystem = PeisarFs::new(None);
if filesystem.exixts("Cargo.toml") {
    println!("Cargo.toml exists");
}
```

### `mkdir(path)`

Creates `path` and any missing parent directories. If the directory already
exists, no operation is performed.

### `write_file(path, content)`

Writes bytes or text to `path`, creating missing parent directories first.

### `read_file(path)`

Reads `path` as a UTF-8 `String`. Missing or unreadable files are reported
through `peisar_log` and terminate the process with an error status.

### `read_dir(path, extension)`

Recursively returns files below `path` as `Vec<PathBuf>`.

- `Some("rs")` returns only files whose extension is exactly `rs`.
- `None` returns all files.

Returned paths include the configured root directory.

## Development and tests

Run the package tests from the repository root:

```sh
cargo test -p peisar_fs
```

Build the package documentation with warnings treated as errors:

```sh
RUSTDOCFLAGS='-D warnings' cargo doc -p peisar_fs --no-deps
```

## Repository files

- [Cargo.toml](./Cargo.toml)
- [Library source](./src/lib.rs)

## License

See the repository's license information.