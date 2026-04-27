#[derive(Debug, thiserror::Error)]
pub enum ProcessorError {
    #[error("plugin error {code}: {message}")]
    Plugin { code: i64, message: String },

    #[error("invalid image: {0}")]
    Image(#[from] image::ImageError),

    #[error("plugin load error: {0}")]
    Load(#[from] libloading::Error),

    #[error("params contain null byte which is not good for CSTring")]
    BrokenParams,
}
