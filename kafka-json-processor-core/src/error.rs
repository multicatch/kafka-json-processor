use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::processor::ObjectKey;

#[repr(transparent)]
pub struct ProcessingError {
    pub inner: ErrorKind
}

impl ProcessingError {
    pub fn new(inner: ErrorKind) -> ProcessingError {
        ProcessingError { inner }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    InvalidObjectTree {
        invalid_key: Vec<ObjectKey>,
        reason: String,
    },
    EmptyKey,
    FieldNotFound {
        key: Vec<ObjectKey>,
    },
    OtherError {
        err: Box<dyn Error>
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::InvalidObjectTree { invalid_key, reason } =>
                write!(f, "Object tree is incompatible with key [{:?}], reason: {}", invalid_key, reason),
            ErrorKind::EmptyKey =>
                write!(f, "Illegal object tree key: Key is empty."),
            ErrorKind::FieldNotFound { key } =>
                write!(f, "No key in object: [{:?}]", key),
            ErrorKind::OtherError { err } =>
                write!(f, "Unexpected error while processing: {}", err),

        }
    }
}

impl Display for ProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

impl Debug for ProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl Error for ProcessingError {}

impl<T: Error + 'static> From<T> for ErrorKind {
    fn from(err: T) -> Self {
        ErrorKind::OtherError { err: Box::new(err) }
    }
}

impl From<ErrorKind> for ProcessingError {
    fn from(e: ErrorKind) -> Self {
        ProcessingError::new(e)
    }
}