use crate::processor::SerializedOutputMessage;

pub mod consumer;
pub mod producer;
pub mod processor;

pub enum PendingMessage {
    Received,
    Processed {
        id: String,
        message: SerializedOutputMessage,
    },
}
