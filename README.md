# Image FFI

Image FFI is a Rust workspace that provides an image processing CLI and a core library with support for dynamically loaded plugins via FFI. The project focuses on correctness, explicit safety contracts, and minimal use of `unsafe`, serving both as a practical tool and a reference implementation for safe Rust FFI design.

## Features

The project supports loading PNG images, converting them to RGBA8 format, and applying transformations implemented in external dynamic plugins. Plugins are loaded at runtime and operate directly on image buffers, allowing flexible extension without recompiling the main application.

## Project Structure

The workspace is split into a library crate that defines error handling and plugin loading, a binary crate that implements the CLI, and one or more plugin crates compiled as dynamic libraries. This separation keeps unsafe FFI boundaries isolated and the public API clean.

## Command-Line Usage

The CLI accepts an input image, an output path, a plugin name, a parameters file, and a plugin directory. At runtime, it loads the requested plugin, passes the image buffer to it, and writes the processed result back to disk.

## Plugin Interface

Each plugin must export a `process_image` function with a C-compatible ABI. The function receives image dimensions, a mutable pointer to an RGBA8 buffer, and an optional NUL-terminated UTF-8 parameters string. Plugins are required to follow a strict safety contract regarding buffer size, lifetimes, and aliasing.

## Unsafe Code Policy

Unsafe code is restricted to FFI boundaries and dynamic symbol loading. Every unsafe operation is accompanied by a `// SAFETY:` comment that explains the required invariants, and the project enables compiler lints to prevent unchecked unsafe operations.

## Error Handling

All errors are represented by a typed `AppError` enum using `thiserror`. I/O, image decoding, plugin loading, and parameter parsing errors are explicitly modeled and propagated without panics in normal execution paths.

## Logging and Observability

The project uses `tracing` for structured logging. Log levels and filters can be configured via environment variables, and logs include relevant context such as image dimensions and plugin names.

## Example Run

The following command applies the `blur_plugin` to an input PNG image using parameters from a text file and writes the result to the specified output path:

```bash
cargo run -p app -- \
  --input examples/input.png \
  --output examples/output.png \
  --plugin blur_plugin \
  --params examples/params.txt \
  --plugin-path target/debug
```

### Example contents of params.txt:
```text
radius=3
iterations=2
horizontal=true
vertical=false
```
