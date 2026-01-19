#![deny(missing_docs)]

//! Image processing core library with dynamic plugin support.

/// Error types used by the image processor.
pub mod error;

/// Dynamic plugin loading and FFI bindings.
pub mod plugin_loader;
