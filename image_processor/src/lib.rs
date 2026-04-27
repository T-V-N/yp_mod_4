pub mod error;
pub mod plugin_loader;
use std::ffi::CString;

use error::ProcessorError;
use image::RgbaImage;

pub fn process(img: &mut RgbaImage, plugin_path: &str, params: &str) -> Result<(), ProcessorError> {
    let plugin = plugin_loader::Plugin::new(plugin_path)?;
    let plugin = plugin.interface()?;

    let params = format!("{}\0", params);

    let params_c = CString::new(params).map_err(|_| ProcessorError::BrokenParams)?;

    unsafe {
        (plugin.process_image)(
            img.height(),
            img.width(),
            img.as_mut_ptr(),
            params_c.as_ptr() as *const i8,
        )
    };

    Ok(())
}
