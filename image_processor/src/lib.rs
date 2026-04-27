pub mod error;
pub mod plugin_loader;
use std::ffi::CString;

use error::ProcessorError;
use image::RgbaImage;

/// Применяет плагин к изображению.
///
/// Загружает динамическую библиотеку по пути `plugin_path` (без расширения),
/// передаёт пиксельный буфер `img` в формате RGBA8 и строку `params` через FFI.
/// Плагин модифицирует буфер на месте.
///
/// # Errors
/// - [`ProcessorError::Load`] — библиотека не найдена или символ `process_image` отсутствует;
/// - [`ProcessorError::BrokenParams`] — строка `params` содержит null-байт;
/// - [`ProcessorError::Image`] — ошибка работы с изображением.
pub fn process(img: &mut RgbaImage, plugin_path: &str, params: &str) -> Result<(), ProcessorError> {
    let plugin = plugin_loader::Plugin::new(plugin_path)?;
    let plugin = plugin.interface()?;

    let params_c = CString::new(params).map_err(|_| ProcessorError::BrokenParams)?;

    unsafe {
        (plugin.process_image)(
            img.width(),
            img.height(),
            img.as_mut_ptr(),
            params_c.as_ptr() as *const i8,
        )
    };

    Ok(())
}
