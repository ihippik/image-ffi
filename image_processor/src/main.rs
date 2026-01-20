use clap::Parser;
use image::{ImageBuffer, Rgba};
use std::ffi::CString;
use std::path::{Path, PathBuf};

use image_processor::error::AppError;
use image_processor::plugin_loader::Plugin;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "image_processor")]
struct Args {
    /// path to input PNG
    #[arg(long)]
    input: String,

    /// path to output PNG
    #[arg(long)]
    output: String,

    /// plugin name without extension (e.g. mirror_plugin or blur_plugin)
    #[arg(long)]
    plugin: String,

    /// path to params text file
    #[arg(long)]
    params: String,

    /// directory with plugins (default target/debug)
    #[arg(long, default_value = "target/debug")]
    plugin_path: String,
}

fn lib_filename(plugin_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{plugin_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{plugin_name}.dylib")
    } else {
        format!("lib{plugin_name}.so")
    }
}

fn main() -> Result<(), AppError> {
    init_tracing();

    let args = Args::parse();

    if !Path::new(&args.input).exists() {
        return Err(AppError::MissingInput(args.input));
    }
    if !Path::new(&args.params).exists() {
        return Err(AppError::MissingParams(args.params));
    }

    let params_bytes = std::fs::read(&args.params)?;
    let params_str =
        std::str::from_utf8(&params_bytes).map_err(|_| AppError::InvalidParamsUtf8)?;
    let params_c =
        CString::new(params_str).map_err(|_| AppError::InvalidParamsNul)?;

    let img = image::open(&args.input)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut data: Vec<u8> = rgba.into_raw();

    let mut plugin_path = PathBuf::from(&args.plugin_path);
    plugin_path.push(lib_filename(&args.plugin));

    if !plugin_path.exists() {
        return Err(AppError::MissingPlugin(plugin_path.display().to_string()));
    }

    tracing::info!(width, height,input_file=args.input,plugin=plugin_path.display().to_string(),"image processing..");

    // SAFETY:
    // - We only load from a path we constructed and checked exists.
    // - `Plugin::load` is unsafe because Rust can't verify at compile time that the loaded
    //   dynamic library exports the expected symbol with the expected ABI/signature.
    // - If the library is not compatible (wrong symbol, wrong signature, wrong ABI),
    //   calling through the obtained function pointer would be Undefined Behavior.
    let plugin = unsafe { Plugin::load(&plugin_path)? };
    let process = plugin.process_ptr();

    // SAFETY:
    // - `data` is a Vec<u8> of length exactly `width * height * 4` (RGBA8), produced by `into_raw()`.
    // - `data.as_mut_ptr()` is non-null for non-empty vectors; if empty, width/height would be 0
    //   and the plugin contract should handle it (or we can early-return).
    // - The pointer remains valid for the duration of the call because `data` is not reallocated
    //   or moved while the call is in progress.
    // - `params_c.as_ptr()` is a valid NUL-terminated C string that lives for the duration of the call.
    // - We assume the plugin follows the FFI contract: it will only read/write within the provided
    //   buffer bounds and will not store the pointers for later use.
    unsafe {
        process(width, height, data.as_mut_ptr(), params_c.as_ptr());
    }

    let out: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, data).expect("Invalid RGBA buffer length");
    out.save(&args.output)?;

    tracing::info!(output_file=args.output, "output file saved");

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();
}
