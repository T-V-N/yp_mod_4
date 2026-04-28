use libloading::{Library, Symbol, library_filename};
use std::os::raw::c_char;

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
        unsafe extern "C" fn(width: u32, height: u32, rgba_data: *mut u8, params: *const c_char) -> i32,
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
        let path = library_filename(filename);
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
