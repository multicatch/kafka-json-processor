use std::collections::HashMap;
use std::error::Error;
use serde_json::{Number, Value};
use log::{trace, error};

pub struct OutputMessage {
    pub key: Option<String>,
    values: HashMap<String, Value>,
}

impl Default for OutputMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputMessage {
    pub fn new() -> OutputMessage {
        OutputMessage {
            key: None,
            values: HashMap::new(),
        }
    }

    pub fn insert_str(&mut self, key: String, value: String) {
        self.values.insert(key, Value::String(value));
    }

    pub fn insert_number(&mut self, key: String, value: Number) {
        self.values.insert(key, Value::Number(value));
    }

    pub fn insert_bool(&mut self, key: String, value: bool) {
        self.values.insert(key, Value::Bool(value));
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.values.get(key)
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
    }
}

pub struct SerializedOutputMessage {
    pub key: String,
    pub message: String,
}

pub type ProcessingResult<T> = Result<T, Box<dyn Error>>;
pub type Processor = &'static (dyn Fn(&Value, &mut OutputMessage) -> ProcessingResult<()> + Sync + Send);

pub fn process_payload(id: String, payload: &[u8], processors: &[Processor]) -> ProcessingResult<SerializedOutputMessage> {
    trace!("[{id}] Start of processing.");
    let source: Value = serde_json::from_slice(payload)?;
    let mut message: OutputMessage = OutputMessage::new();

    for process in processors.iter() {
        if let Err(e) = process(&source, &mut message) {
            error!("[{id}] Cannot process message. Reason: {e}");
        }
    }

    Ok(SerializedOutputMessage {
        key: message.key.unwrap_or(id),
        message: serde_json::to_string(&message.values)?,
    })
}