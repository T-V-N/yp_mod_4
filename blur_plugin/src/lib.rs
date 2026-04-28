use serde::Deserialize;
use serde_json;
use std::ffi::{CStr, c_char};

#[derive(Debug, Deserialize)]
struct Params {
    radius: u32,
    iterations: u32,
}

#[repr(C)]
enum PluginResponse {
    Ok = 0,
    NullPointerInInput = -1,
    InvalidUTF8 = -2,
    InvalidJSON = -3,
    Overflow = -4
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

    // SAFETY: rgba_data is non-null (checked above); len matches width * height * 4 also checked above
    let data = unsafe { std::slice::from_raw_parts_mut(rgba_data, len) };

    for _ in 0..params.iterations {
        let status = apply_blur(data, width as usize, height as usize, params.radius);
        if status != 0 {
            return status
        }
    }

    return PluginResponse::Ok as i32;
}

fn apply_blur(data: &mut [u8], width: usize, height: usize, radius: u32) -> i32 {
    let r: isize = match radius.try_into() {
        Ok(r) => r,
        Err(_) => return PluginResponse::Overflow as i32,
    };

    let w: isize = match width.try_into() {
        Ok(w) => w,
        Err(_) => return PluginResponse::Overflow as i32,
    };
    let h: isize = match height.try_into() {
        Ok(h) => h,
        Err(_) => return PluginResponse::Overflow as i32,
    };
    
    // for each pixel
    for y in 0..h {
        for x in 0..w {
            // for each channel of the pixel
            for c in 0..3 {
                let mut val = 0.0f32;
                let mut weight_sum = 0.0f32;

                // take the pixel and the surrounding pixels
                for ky in -r..=r {
                    for kx in -r..=r {
                        // use surronding pixel data to calculate new pixel value
                        let px = (x as isize).saturating_add(kx);
                        let py = (y as isize).saturating_add(ky);

                        // check if the pixel is within the image
                        if px >= 0 && px < w  && py >= 0 && py < h {
                            // the closer the surrounding pixel is to the central pixel, the higher the weight
                            let dist = ((kx as f32).powi(2) + (ky as f32).powi(2)).sqrt();
                            let weight = if dist == 0.0 { 1.0 } else { 1.0 / dist };

                            // get the index of the surrounding pixel
                            let idx = (py as usize * w as usize + px as usize) * 4 + c;
                            // sum the weighted pixel channel value
                            val += data[idx] as f32 * weight;
                            // sum the weights
                            weight_sum += weight;
                        }
                    }
                }

                // set the new pixel value
                // val / weight_sum is the new pixel value
                data[(y as usize * w as usize + x as usize) * 4 + c as usize] = (val / weight_sum) as u8;
            }
        }
    }

    return PluginResponse::Ok as i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageReader;
    use std::ffi::CString;

    #[test]
    fn test_blur_matches_reference() {
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

        apply_blur(input.as_mut(), width, height, 2);

        assert_eq!(input.as_raw(), reference.as_raw());
    }

    #[test]
    fn test_blur_output_saved() {
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

        apply_blur(input.as_mut(), width, height, 2);

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
        let params = CString::new(r#"{"radius":1,"iterations":2}"#).unwrap();
        unsafe {
            let result = process_image(u32::MAX, u32::MAX, data.as_mut_ptr(), params.as_ptr());
            assert_eq!(result, PluginResponse::Overflow as i32);
        }
    }
}
