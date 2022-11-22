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
}

impl Display for ProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidObjectTree { invalid_key, reason } =>
                write!(f, "Object tree is incompatible with key [{:?}], reason: {}", invalid_key, reason),
            ProcessingError::EmptyKey =>
                write!(f, "Illegal object tree key: Key is empty."),
        }
    }
}

impl Error for ProcessingError {}