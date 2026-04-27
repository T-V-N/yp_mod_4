use serde::Deserialize;
use serde_json;
use std::ffi::{CStr, c_char, c_uchar, c_uint};

#[derive(Debug, Deserialize)]
struct Params {
    radius: u32,
    iterations: u32,
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

    for _ in 0..params.iterations {
        apply_blur(data, width as usize, height as usize, params.radius);
    }
}

fn apply_blur(data: &mut [u8], width: usize, height: usize, radius: u32) {
    let r = radius as isize;

    // for each pixel
    for y in 0..height {
        for x in 0..width {
            // for each channel of the pixel
            for c in 0..3 {
                let mut val = 0.0f32;
                let mut weight_sum = 0.0f32;

                // take the pixel and the surrounding pixels
                for ky in -r..=r {
                    for kx in -r..=r {
                        // use surronding pixel data to calculate new pixel value
                        let px = x as isize + kx;
                        let py = y as isize + ky;

                        // check if the pixel is within the image
                        if px >= 0 && px < width as isize && py >= 0 && py < height as isize {
                            // the closer the surrounding pixel is to the central pixel, the higher the weight
                            let dist = ((kx * kx + ky * ky) as f32).sqrt();
                            let weight = if dist == 0.0 { 1.0 } else { 1.0 / dist };

                            // get the index of the surrounding pixel
                            let idx = (py as usize * width + px as usize) * 4 + c;
                            // sum the weighted pixel channel value
                            val += data[idx] as f32 * weight;
                            // sum the weights
                            weight_sum += weight;
                        }
                    }
                }

                // set the new pixel value
                // val / weight_sum is the new pixel value
                data[(y * width + x) * 4 + c] = (val / weight_sum) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageReader;

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
}
