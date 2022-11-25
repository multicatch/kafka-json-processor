use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::processor::ObjectKey;

#[derive(Debug)]
pub enum ProcessingError {
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

impl Display for ProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidObjectTree { invalid_key, reason } =>
                write!(f, "Object tree is incompatible with key [{:?}], reason: {}", invalid_key, reason),
            ProcessingError::EmptyKey =>
                write!(f, "Illegal object tree key: Key is empty."),
            ProcessingError::FieldNotFound { key } => 
                write!(f, "No key in object: [{:?}]", key),
            ProcessingError::OtherError { err } =>
                write!(f, "Unexpected error while processing: {}", err),

        }
    }
}

impl Error for ProcessingError {}

impl From<Box<dyn Error>> for ProcessingError {
    fn from(err: Box<dyn Error>) -> Self {
        ProcessingError::OtherError { err }
    }
}
