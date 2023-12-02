use std::fmt::Display;

pub type Result<T> = std::result::Result<T, ColekError>;
#[derive(Debug)]
pub enum ColekError {
    NoGenericDrive,
    Err(String),
    StaticErr(&'static str),
    IoError(std::io::Error),
    Ignore(ignore::Error),
    Zip(zip::result::ZipError),
}

impl std::error::Error for ColekError {}
impl Display for ColekError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColekError::NoGenericDrive => f.write_str("No Generic Drive Detected"),
            ColekError::Err(err) => write!(f, "{err}"),
            ColekError::StaticErr(err) => f.write_str(err),
            ColekError::IoError(ioerr) => write!(f, "IO: {ioerr}"),
            ColekError::Ignore(err) => write!(f, "walkdir: {err}"),
            ColekError::Zip(err) => write!(f, "zip: {err}"),
        }
    }
}

impl From<std::io::Error> for ColekError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<String> for ColekError {
    fn from(value: String) -> Self {
        Self::Err(value)
    }
}

impl From<&'static str> for ColekError {
    fn from(value: &'static str) -> Self {
        Self::StaticErr(value)
    }
}

impl From<ignore::Error> for ColekError {
    fn from(value: ignore::Error) -> Self {
        Self::Ignore(value)
    }
}

impl From<Box<dyn std::error::Error>> for ColekError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self::Err(value.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send>> for ColekError {
    fn from(value: Box<dyn std::error::Error + Send>) -> Self {
        Self::Err(value.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ColekError {
    fn from(value: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Err(value.to_string())
    }
}

impl From<zip::result::ZipError> for ColekError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value)
    }
}
