use std::error::Error;
use std::mem::discriminant;
use serde_json::{Map, Value};
use log::{trace, error, debug};
use crate::error::{ErrorKind, ProcessingError};

pub struct OutputMessage {
    pub key: Option<String>,
    value: Value,
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
            value: Value::Null,
        }
    }
}

pub trait ObjectTree {
    fn get_val(&self, key: &[ObjectKey]) -> Result<&Value, ProcessingError>;
    fn insert_val(&mut self, key: &[ObjectKey], value: Value) -> Result<(), ProcessingError>;
}

#[derive(Debug, Clone)]
pub enum ObjectKey {
    Index(usize),
    Key(String),
}

impl ObjectTree for OutputMessage {
    fn get_val(&self, key: &[ObjectKey]) -> Result<&Value, ProcessingError> {
        self.value.get_val(key)
    }

    fn insert_val(&mut self, key: &[ObjectKey], value: Value) -> Result<(), ProcessingError> {
        if key.is_empty() {
            return Err(ErrorKind::EmptyKey.into());
        }

        if let Value::Null = self.value {
            self.value = match key[0] {
                ObjectKey::Key(_) => Value::Object(Map::new()),
                ObjectKey::Index(_) => Value::Array(vec![]),
            };
        }

        self.value.insert_val(key, value)
    }
}

impl ObjectTree for Value {
    fn get_val(&self, key: &[ObjectKey]) -> Result<&Value, ProcessingError> {
        if key.is_empty() {
            return Err(ErrorKind::EmptyKey.into());
        }

        let mut node = Some(self);

        for k in key {
            node = node.and_then(|n| match k {
                ObjectKey::Index(i) => n.get(i),
                ObjectKey::Key(k) => n.get(k),
            })
        }

        node.ok_or_else(|| ErrorKind::FieldNotFound { key: key.to_vec()}.into())
    }

    fn insert_val(&mut self, key: &[ObjectKey], value: Value) -> Result<(), ProcessingError> {
        if key.is_empty() {
            return Err(ErrorKind::EmptyKey.into());
        }

        if let Value::Null = self {
            return Err(ErrorKind::InvalidObjectTree {
                invalid_key: key.to_vec(),
                reason: "Root node is null.".to_string(),
            }.into());
        }

        let mut cur_value = self;

        for i in 0..key.len() - 1 {
            cur_value = insert_node(cur_value, &key[i], &key[i + 1])
                .map_err(|e| fill_key_in_error(e, key))?;
        }

        insert_child(cur_value, key.last().unwrap(), value)
            .map_err(|e| fill_key_in_error(e, key))?;

        Ok(())
    }
}

fn fill_key_in_error(e: ProcessingError, key: &[ObjectKey]) -> ProcessingError {
    match e.inner {
        ErrorKind::InvalidObjectTree { reason, .. } => ErrorKind::InvalidObjectTree {
            invalid_key: key.to_vec(),
            reason
        }.into(),
        _ => e,
    }
}

fn insert_node<'a>(node: &'a mut Value, key: &ObjectKey, child: &ObjectKey) -> Result<&'a mut Value, ProcessingError> {
    let child = match child {
        ObjectKey::Key(_) => Value::Object(Map::new()),
        ObjectKey::Index(_) => Value::Array(vec![]),
    };

    insert_child(node, key, child)
}

fn insert_child<'a>(node: &'a mut Value, key: &ObjectKey, child: Value) -> Result<&'a mut Value, ProcessingError> {
    if let Value::Null = node {
        return Err(ErrorKind::InvalidObjectTree {
            invalid_key: vec![key.clone()],
            reason: "Node is null.".to_string(),
        }.into());
    }

    match key {
        ObjectKey::Key(k) => {
            if let Value::Object(values) = node {
                let cur_node = values.get(k).unwrap_or(&Value::Null);

                verify_type_compatibility(cur_node, &child)?;
                values.insert(k.clone(), child);

                Ok(values.get_mut(k).unwrap())
            } else {
                Err(ErrorKind::InvalidObjectTree {
                    invalid_key: vec![key.clone()],
                    reason: "Node has incompatible type, Object expected.".to_string(),
                }.into())
            }
        }

        ObjectKey::Index(i) => {
            let i = *i;

            if let Value::Array(values) = node {
                while values.len() <= i {
                    values.push(Value::Null);
                }

                verify_type_compatibility(&values[i], &child)?;
                values[i] = child;

                Ok(&mut values[i])
            } else {
                Err(ErrorKind::InvalidObjectTree {
                    invalid_key: vec![key.clone()],
                    reason: "Node has incompatible type, Array expected.".to_string(),
                }.into())
            }
        }
    }
}

fn verify_type_compatibility(node: &Value, child: &Value) -> Result<(), ProcessingError> {

    let child_discriminant = discriminant(child);
    let node_discriminant = discriminant(node);

    match node {
        Value::Object(_) | Value::Array(_) => {
            if node_discriminant != child_discriminant {
                return Err(ErrorKind::InvalidObjectTree {
                    invalid_key: vec![],
                    reason: format!("Child node has incompatible type, cannot merge {child_discriminant:?} into {node_discriminant:?}."),
                }.into());
            }
        }

        _ => {}
    }

    Ok(())
}

pub struct SerializedOutputMessage {
    pub key: String,
    pub message: String,
}

pub type ProcessingResult<T> = Result<T, Box<dyn Error>>;
pub type Processor = &'static (dyn Fn(&Value, &mut OutputMessage) -> Result<(), ProcessingError> + Sync + Send);

pub fn process_payload(id: String, payload: &[u8], processors: &[Processor]) -> ProcessingResult<SerializedOutputMessage> {
    trace!("[{id}] Start of processing.");
    let source: Value = serde_json::from_slice(payload)?;
    let mut message: OutputMessage = OutputMessage::new();

    for (i, process) in processors.iter().enumerate() {
        if let Err(e) = process(&source, &mut message) {
            let e: ProcessingError = e;
            match e.inner {
                ErrorKind::FieldNotFound { .. } =>
                    debug!("[{id}]#{i} {e}. Skipping processor."),

                ErrorKind::ProcessorSkipped { .. } =>
                    debug!("[{id}]#{i} {e}"),

                _ =>
                    error!("[{id}]#{i} Cannot process message. Reason: {e}"),
            }
        }
    }

    trace!("[{id}] End of processing - serializing message.");

    Ok(SerializedOutputMessage {
        key: message.key.unwrap_or(id),
        message: serde_json::to_string(&message.value)?,
    })
}