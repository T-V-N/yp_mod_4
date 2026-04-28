use serde::Deserialize;
use serde_json;
use std::ffi::{CStr, c_char};

#[derive(Debug, Deserialize)]
struct Params {
    horizontal: bool,
    vertical: bool,
}

#[repr(C)]
enum PluginResponse {
    Ok = 0,
    NullPointerInInput = -1,
    InvalidUTF8 = -2,
    InvalidJSON = -3,
    Overflow = -4,
}

/// # SAFETY
/// - `rgba_data` must be a not null pointer to a buffer of exactly `width * height * 4` bytes valid until end of processing
/// - `params` must be a null-terminated C string vsalid until end of processing
#[unsafe(no_mangle)]
unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) -> i32 {
    if rgba_data.is_null() || params.is_null() {
        return PluginResponse::NullPointerInInput as i32;
    }

    // SAFETY: ptr is non-null, checked above
    let json_str = match unsafe { CStr::from_ptr(params).to_str() } {
        Ok(s) => s,
        Err(_) => return PluginResponse::InvalidUTF8 as i32, // invalid UTF-8
    };

    let params: Params = match serde_json::from_str(json_str) {
        Ok(p) => p,
        Err(_) => return PluginResponse::InvalidJSON  as i32, // invalid JSON
    };

    let len_checked = (height as usize)
        .checked_mul(width as usize)
        .and_then(|n| n.checked_mul(4));

    let len = match len_checked {
        Some(l) => l,
        None => return PluginResponse::Overflow as i32,
    };

    // SAFETY: rgba_data is non-null (checked above); len matches width * height * 4 also checked
    let data = unsafe { std::slice::from_raw_parts_mut(rgba_data, len) };

    apply_mirror(
        data,
        width as usize,
        height as usize,
        params.vertical,
        params.horizontal,
    );

    return PluginResponse::Ok as i32;
}

fn apply_mirror(data: &mut [u8], width: usize, height: usize, vertical: bool, horizontal: bool) {
    // for each pixel
    if vertical {
        for y in 0..height / 2 {
            for x in 0..width {
                for c in 0..4 {
                    let a = data[(y * width + x) * 4 + c];
                    let buff = data[((height - y - 1) * width + x) * 4 + c];
                    data[((height - y - 1) * width + x) * 4 + c] = a;
                    data[(y * width + x) * 4 + c] = buff;
                }
            }
        }
    }

    if horizontal {
        for y in 0..height {
            for x in 0..width / 2 {
                for c in 0..4 {
                    let a = data[(y * width + x) * 4 + c];
                    let b = data[(y * width + (width - x - 1)) * 4 + c];
                    data[(y * width + x) * 4 + c] = b;
                    data[(y * width + (width - x - 1)) * 4 + c] = a;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageReader;
    use std::ffi::CString;
    use PluginResponse;

    #[test]
    fn test_vertical_mirror_matches_reference() {
        let mut input = ImageReader::open("tests/input.png")
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

        let reference = ImageReader::open("tests/reference.png")
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

        let height = input.height() as usize;
        let width = input.width() as usize;

        apply_mirror(input.as_mut(), width, height, true, true);

        assert_eq!(input.as_raw(), reference.as_raw());
    }

    #[test]
    fn test_mirror_output_saved() {
        let mut input = ImageReader::open("tests/input.png")
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

        let reference = ImageReader::open("tests/reference.png")
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

        let height = input.height() as usize;
        let width = input.width() as usize;

        apply_mirror(input.as_mut(), width, height, true, true);

        input.save("tests/output.png").unwrap();

        let output = ImageReader::open("tests/output.png")
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

        assert_eq!(output.as_raw(), reference.as_raw());
    }

    #[test]
    fn test_overflow_dimensions_returns_error() {
        let mut data = vec![0u8; 4];
        let params = CString::new(r#"{"vertical":true,"horizontal":false}"#).unwrap();
        unsafe {
            let result =  process_image(u32::MAX, u32::MAX, data.as_mut_ptr(), params.as_ptr());
            assert_eq!(result, PluginResponse::Overflow as i32);
        }
    }
}
