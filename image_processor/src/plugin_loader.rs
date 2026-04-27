use libloading::{Library, Symbol};
use std::os::raw::c_char;

#[cfg(target_os = "windows")]
const LIB_EXT: &str = "dll";

#[cfg(target_os = "macos")]
const LIB_EXT: &str = "dylib";

#[cfg(target_os = "linux")]
const LIB_EXT: &str = "so";

/// Загруженные символы плагина, привязанные к времени жизни [`Plugin`].
pub struct PluginInterface<'a> {
    /// FFI-функция обработки изображения.
    ///
    /// # Safety
    /// Вызывающий обязан гарантировать:
    /// - `rgba_data` указывает на валидный буфер размером `width * height * 4` байт;
    /// - `params` — валидная null-terminated C-строка;
    /// - буфер остаётся живым до возврата из функции.
    pub process_image: Symbol<
        'a,
        unsafe extern "C" fn(width: u32, height: u32, rgba_data: *mut u8, params: *const c_char),
    >,
}

/// Обёртка над загруженной динамической библиотекой плагина.
pub struct Plugin {
    plugin: Library,
}

impl Plugin {
    /// Загружает динамическую библиотеку по пути `filename` (без расширения).
    ///
    /// Расширение добавляется автоматически по ОС: `.so` / `.dylib` / `.dll`.
    ///
    /// # Errors
    /// [`libloading::Error`] — файл не найден или не является корректной shared library.
    pub fn new(filename: &str) -> Result<Self, libloading::Error> {
        let path = format!("{}.{}", filename, LIB_EXT);
        Ok(Plugin {
            plugin: unsafe { Library::new(path) }?,
        })
    }

    /// Загружает символ `process_image` из библиотеки.
    ///
    /// # Errors
    /// [`libloading::Error`] — символ не экспортирован библиотекой.
    pub fn interface(&self) -> Result<PluginInterface<'_>, libloading::Error> {
        Ok(PluginInterface {
            process_image: unsafe { self.plugin.get("process_image") }?,
        })
    }
}
