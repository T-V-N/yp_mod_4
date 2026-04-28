/// Ошибки, возникающие при обработке изображения.
#[derive(Debug, thiserror::Error)]
pub enum ProcessorError {
    /// Ошибка декодирования или сохранения изображения.
    #[error("invalid image: {0}")]
    Image(#[from] image::ImageError),

    /// Ошибка загрузки динамической библиотеки или поиска символа `process_image`.
    #[error("plugin load error: {0}")]
    Load(#[from] libloading::Error),

    /// Строка параметров содержит null-байт и не может быть передана плагину через FFI.
    #[error("params string contains a null byte, cannot convert to CString")]
    BrokenParams,

    /// Плагин вернул код ошибки.
    #[error("plugin returned error code: {0}")]
    PluginError(i32),
}
