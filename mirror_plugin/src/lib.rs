use serde::Deserialize;
use serde_json;
use std::ffi::{CStr, c_char, c_uchar, c_uint};

#[derive(Debug, Deserialize)]
struct Params {
    horizontal: bool,
    vertical: bool,
}

#[unsafe(no_mangle)]
extern "C" fn process_image(
    height: c_uint,
    width: c_uint,
    rgba_data: *mut c_uchar,
    params: *const c_char,
) {
    let json_str = match unsafe { CStr::from_ptr(params).to_str() } {
        Ok(s) => s,
        Err(_) => return eprintln!("Plugin error: InvalidUtf8"), // invalid UTF-8
    };

    let params: Params = match serde_json::from_str(json_str) {
        Ok(p) => p,
        Err(_) => return eprintln!("Plugin error: InvalidJSON"), // invalid JSON
    };

    let len = (height * width * 4) as usize;
    let data = unsafe { std::slice::from_raw_parts_mut(rgba_data, len) };

    apply_mirror(
        data,
        width as usize,
        height as usize,
        params.vertical,
        params.horizontal,
    );
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
}
