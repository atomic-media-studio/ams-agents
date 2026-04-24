mod ids;
mod records;
mod time;

pub use ids::*;
pub use records::AuditRecord;
pub use records::SCHEMA_VERSION;
pub use time::*;

use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct AuditHandle {
    writer: Mutex<Option<BufWriter<File>>>,
    path: PathBuf,
}

impl AuditHandle {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            writer: Mutex::new(Some(BufWriter::new(file))),
            path,
        })
    }

    pub fn disabled() -> Self {
        Self {
            writer: Mutex::new(None),
            path: PathBuf::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append_json_line(&self, record: &AuditRecord) -> io::Result<()> {
        let mut guard = self.writer.lock().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("audit mutex poisoned: {e}"))
        })?;
        if let Some(w) = guard.as_mut() {
            serde_json::to_writer(&mut *w, record)?;
            w.write_all(b"\n")?;
            w.flush()?;
        }
        Ok(())
    }
}
