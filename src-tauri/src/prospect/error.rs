use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProspectError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Invalid prospect file: {0}")]
    InvalidFile(String),

    #[error("Property parse error at offset {offset}: {message}")]
    PropertyParse { offset: u64, message: String },

    #[error("Component not found: index {0}")]
    ComponentNotFound(usize),

    #[error("Property path not found: {0}")]
    PropertyPathNotFound(String),

    #[error("Unsupported property type: {0}")]
    UnsupportedPropertyType(String),
}

impl serde::Serialize for ProspectError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
