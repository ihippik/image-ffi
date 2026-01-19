use libloading::{Library, Symbol};
use std::path::Path;

/// FFI function signature exported by image processing plugins.
///
/// The function processes an RGBA8 image buffer in place.
pub type ProcessFn = unsafe extern "C" fn(u32, u32, *mut u8, *const std::os::raw::c_char);

/// Dynamically loaded image processing plugin.
pub struct Plugin {
    _lib: Library,
    process: Symbol<'static, ProcessFn>,
}

impl Plugin {
    /// Loads a plugin dynamic library and resolves the `process_image` symbol.
    ///
    /// # SAFETY
    /// The caller must ensure that the library at `path`:
    /// - exports a `process_image` symbol with the exact `ProcessFn` ABI and signature,
    /// - follows the FFI contract for the function (buffer size, lifetimes, no aliasing),
    /// - remains compatible for the lifetime of the returned `Plugin`.
    pub unsafe fn load(path: &Path) -> Result<Self, libloading::Error> {
        let lib = Library::new(path)?;
        let sym: Symbol<ProcessFn> = lib.get(b"process_image")?;
        let process: Symbol<'static, ProcessFn> = std::mem::transmute(sym);

        Ok(Self { _lib: lib, process })
    }

    /// Returns a reference to the plugin's image processing function pointer.
    pub fn process_ptr(&self) -> &Symbol<'static, ProcessFn> {
        &self.process
    }
}
