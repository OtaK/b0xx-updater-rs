use failure_derive::Fail;

macro_rules! from_error {
    ($type:ty, $target:ident, $targetvar:expr) => {
        impl From<$type> for $target {
            fn from(s: $type) -> Self {
                $targetvar(s.into())
            }
        }
    };
}

#[allow(dead_code)]
#[derive(Debug, Fail)]
pub enum UpdaterError {
    #[fail(display = "IoError: {}", _0)]
    IoError(std::io::Error),
    #[fail(display = "DeserializationError: {}", _0)]
    DeserializationError(toml::de::Error),
    #[fail(display = "SerializationError: {}", _0)]
    SerializationError(toml::ser::Error),
    #[fail(display = "An unknown error occured, sorry")]
    UnknownError,
}

from_error!(std::io::Error, UpdaterError, UpdaterError::IoError);
from_error!(
    toml::de::Error,
    UpdaterError,
    UpdaterError::DeserializationError
);
from_error!(
    toml::ser::Error,
    UpdaterError,
    UpdaterError::SerializationError
);
