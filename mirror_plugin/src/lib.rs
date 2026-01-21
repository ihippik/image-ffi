use std::ffi::CStr;
use std::os::raw::c_char;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Params {
    horizontal: bool,
    vertical: bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) -> u32 {
    if rgba_data.is_null() {
        return 1;
    }

    // SAFETY:
    // - We checked `params` is not NULL.
    // - FFI contract requires `params` to be a valid NUL-terminated C string
    //   that remains valid for the duration of this call.
    // - `CStr::from_ptr` will read memory until the first NUL byte; if the pointer
    //   is invalid or not NUL-terminated, that would be UB, hence the contract.
    let params_str = unsafe {
        if params.is_null() {
            ""
        } else {
            CStr::from_ptr(params).to_str().unwrap_or("")
        }
    };

    let params: Params = match toml::from_str(&params_str) {
        Ok(config) => config,
        Err(_) => {
            return 1;
        }
    };


    let w = width as usize;
    let h = height as usize;
    let len = w.checked_mul(h).and_then(|wh| wh.checked_mul(4));

    let Some(total_len) = len else {
        return 1;
    };

    // SAFETY:
    // - We checked `rgba_data` is not NULL above.
    // - FFI contract requires `rgba_data` to point to at least `len` writable bytes.
    // - The pointed-to memory is assumed valid for the duration of this call.
    // - `u8` alignment is 1, so `rgba_data` is always sufficiently aligned.
    // - Caller must ensure there are no competing mutable borrows/aliases of the same buffer
    //   while this function runs (no aliasing / no data races).
    let buf = unsafe { std::slice::from_raw_parts_mut(rgba_data, total_len) };

    if params.horizontal {
        flip_top_bottom_in_place(w, h, buf);
    }

    if params.vertical {
        mirror_left_right_in_place(w, h, buf);
    }

    0
}

fn flip_top_bottom_in_place(width: usize, height: usize, buf: &mut [u8]) {
    let row_bytes = width * 4;
    if row_bytes == 0 || height == 0 {
        return;
    }

    let mut tmp = vec![0u8; row_bytes];

    for y in 0..(height / 2) {
        let top = y * row_bytes;
        let bottom = (height - 1 - y) * row_bytes;

        let (head, tail) = buf.split_at_mut(bottom);
        let top_row = &mut head[top..top + row_bytes];
        let bottom_row = &mut tail[..row_bytes];

        tmp.copy_from_slice(top_row);
        top_row.copy_from_slice(bottom_row);
        bottom_row.copy_from_slice(&tmp);
    }
}

fn mirror_left_right_in_place(width: usize, height: usize, buf: &mut [u8]) {
    let row_bytes = width * 4;
    if width == 0 || height == 0 {
        return;
    }

    for y in 0..height {
        let row_start = y * row_bytes;
        for x in 0..(width / 2) {
            let left = row_start + x * 4;
            let right = row_start + (width - 1 - x) * 4;

            buf.swap(left + 0, right + 0);
            buf.swap(left + 1, right + 1);
            buf.swap(left + 2, right + 2);
            buf.swap(left + 3, right + 3);
        }
    }
}
