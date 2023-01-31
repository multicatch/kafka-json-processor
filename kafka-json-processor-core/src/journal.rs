use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use log::{debug, error};
use crate::MessageOffset;

pub struct MessageOffsetHolder {
    journal_enabled: bool,
    journal_dir: String,
    inner: Arc<Mutex<HashMap<OffsetKey, Offset>>>
}

type Offset = i64;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct OffsetKey(pub String, pub i32);

impl MessageOffsetHolder {
    pub fn new(journal_dir: String, enabled: bool) -> Result<MessageOffsetHolder, Box<dyn Error>> {
        let offsets = if enabled {
            read_offsets_from(&journal_dir)?
        } else {
            HashMap::new()
        };

        Ok(MessageOffsetHolder {
            journal_dir,
            journal_enabled: enabled,
            inner: Arc::new(Mutex::new(offsets)),
        })
    }

    pub fn offsets(&self) -> HashMap<OffsetKey, Offset> {
        if self.journal_enabled {
            self.lock().unwrap().clone()
        } else {
            HashMap::new()
        }
    }

    pub fn update(&self, offset: MessageOffset) {
        if !self.journal_enabled {
            return;
        }

        let mut guard = self.inner.lock().unwrap();
        (*guard).insert(OffsetKey(offset.topic, offset.partition), offset.offset);
    }

    pub fn flush(&self) {
        if !self.journal_enabled {
            return;
        }

        let offsets = self.offsets();
        save_offsets_in(offsets, &self.journal_dir);
    }
}

fn read_offsets_from<P: AsRef<Path>>(directory: P) -> Result<HashMap<OffsetKey, Offset>, Box<dyn Error>> {
    ensure_dir_exists(&directory)?;

    let map = fs::read_dir(directory.as_ref())?
        .filter_map(|file| {
            file.map_err(|e| {
                error!("Error reading file: {e}")
            }).ok()
        })
        .filter_map(|file| {
            let path = file.path();

            fs::read_to_string(&path)
                .map_err(|e| {
                    error!("Error reading file {path:?}! Reason: {e}")
                })
                .ok()
                .and_then(|content| {
                    let offset_key = file_name_to_offset_key(path.file_name(), path.file_stem())?;
                    Some((offset_key, content.parse().unwrap()))
                })
        })
        .collect();

    Ok(map)
}

/// Parses file_name as "$topic.$partition" (eg. sampletopic.1) and returns OffsetKey if successful
fn file_name_to_offset_key(file_name: Option<&OsStr>, file_stem: Option<&OsStr>) -> Option<OffsetKey> {
    let file_name = file_name?.to_str()?;
    let file_stem = file_stem?.to_str()?;

    let topic = file_name.strip_suffix(&file_stem)?.to_string();

    Some(OffsetKey(topic, file_stem.parse().ok()?))
}

fn save_offsets_in<P: AsRef<Path>>(offsets: HashMap<OffsetKey, Offset>, directory: P) {
    if let Err(e) = ensure_dir_exists(&directory) {
        error!("Cannot save offsets! {e}");
        return;
    }

    offsets.into_iter()
        .for_each(|(key, offset)| {
            save_offset(&directory, key, offset);
        });
}

fn save_offset<P: AsRef<Path>>(base_path: P, offset_key: OffsetKey, offset: Offset) {
    let base_path = PathBuf::from(base_path.as_ref());
    let file_path = base_path.join(format!("{}.{}", &offset_key.0, &offset_key.1));

    debug!("Saving offset [Topic: {}] [Partition: {}] [Offset: {}] to {:?}", &offset_key.0, &offset_key.1, offset, &file_path);

    // save offset to file with name "$topic.$partition" (eg. sampletopic.1)
    if let Err(e) = fs::write(&file_path, format!("{}", offset)) {
        error!("Failed to write journal to file {file_path:?}. Topic: [{}], Partition: [{}], Offset: [{}]. Reason: {e}", offset_key.0, offset_key.1, offset);
    }
}

fn ensure_dir_exists<P: AsRef<Path>>(directory: P) -> Result<(), String> {
    let dir = directory.as_ref();
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(dir) {
            return Err(format!("Cannot create directory {dir:?}! Journal will not be saved. Reason: {e}"));
        }
    }
    if dir.is_file() {
        return Err(format!("Directory is a file: {dir:?}! Cannot use as journal."));
    }

    Ok(())
}

impl Deref for MessageOffsetHolder {
    type Target = Mutex<HashMap<OffsetKey, Offset>>;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl Drop for MessageOffsetHolder {
    fn drop(&mut self) {
        // when json-processor finishes, we should save current offsets
        self.flush()
    }
}