use std::ffi::CStr;
use std::os::raw::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    if rgba_data.is_null() {
        return;
    }

    let params_str = unsafe {
        if params.is_null() {
            ""
        } else {
            // SAFETY:
            // - We checked `params` is not null.
            // - FFI contract requires `params` to be a valid NUL-terminated C string
            //   that lives at least for the duration of this call.
            // - `from_ptr` only reads memory until the first NUL bytes.
            CStr::from_ptr(params).to_str().unwrap_or("")
        }
    };

    let radius = get_u32(params_str, "radius").unwrap_or(3);
    let iterations = get_u32(params_str, "iterations").unwrap_or(1);

    let w = width as usize;
    let h = height as usize;
    let len = w.checked_mul(h).and_then(|wh| wh.checked_mul(4));

    let Some(total_len) = len else {
        return;
    };

    // SAFETY:
    // - We checked `rgba_data` is not null above.
    // - FFI contract requires `rgba_data` to point to at least `len` writable bytes.
    // - The buffer is assumed valid for the duration of this call.
    // - `u8` has alignment 1, so alignment requirements are trivially satisfied.
    // - No other mutable references to this buffer may exist during this call
    //   (caller must ensure no aliasing).
    let buf = unsafe { std::slice::from_raw_parts_mut(rgba_data, total_len) };

    blur_in_place(w, h, buf, radius, iterations);
}

fn get_u32(s: &str, key: &str) -> Option<u32> {
    let lower = s.to_lowercase();
    let key = key.to_lowercase();

    let pos = lower.find(&key)?;
    let tail = &lower[pos + key.len()..];

    let digits: String = tail.chars().skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();

    if digits.is_empty() { None } else { digits.parse().ok() }
}

fn blur_in_place(width: usize, height: usize, buf: &mut [u8], radius: u32, iterations: u32) {
    if width == 0 || height == 0 || radius == 0 || iterations == 0 {
        return;
    }

    let r = radius as i32;
    let row_bytes = width * 4;
    let expected_len = row_bytes * height;

    if buf.len() < expected_len {
        return;
    }

    let mut tmp = vec![0u8; expected_len];

    for _ in 0..iterations {
        let src: &[u8] = &buf[..expected_len];
        let dst: &mut [u8] = &mut tmp[..];

        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let mut acc = [0.0f32; 4];
                let mut wsum = 0.0f32;

                let y0 = (y - r).max(0);
                let y1 = (y + r).min(height as i32 - 1);
                let x0 = (x - r).max(0);
                let x1 = (x + r).min(width as i32 - 1);

                for ny in y0..=y1 {
                    for nx in x0..=x1 {
                        let dx = (nx - x) as f32;
                        let dy = (ny - y) as f32;
                        let dist = (dx * dx + dy * dy).sqrt();
                        let w = 1.0f32 / (1.0f32 + dist);

                        let idx = ((ny as usize) * width + (nx as usize)) * 4;
                        acc[0] += src[idx + 0] as f32 * w;
                        acc[1] += src[idx + 1] as f32 * w;
                        acc[2] += src[idx + 2] as f32 * w;
                        acc[3] += src[idx + 3] as f32 * w;
                        wsum += w;
                    }
                }

                let out_idx = ((y as usize) * width + (x as usize)) * 4;
                let inv = if wsum > 0.0 { 1.0 / wsum } else { 0.0 };

                dst[out_idx + 0] = (acc[0] * inv).round().clamp(0.0, 255.0) as u8;
                dst[out_idx + 1] = (acc[1] * inv).round().clamp(0.0, 255.0) as u8;
                dst[out_idx + 2] = (acc[2] * inv).round().clamp(0.0, 255.0) as u8;
                dst[out_idx + 3] = (acc[3] * inv).round().clamp(0.0, 255.0) as u8;
            }
        }

        buf[..expected_len].copy_from_slice(&tmp);
    }
}
