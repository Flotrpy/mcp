//! Importers. Each returns parsed samples plus counts for the import log.

pub mod apple;
pub mod csv;

use crate::store::Sample;

#[derive(Debug, Default)]
pub struct Parsed {
    pub samples: Vec<Sample>,
    /// Records seen in the file.
    pub read: usize,
    /// Records ignored (unknown type, unparseable date or unit).
    pub skipped: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("could not open {0}: {1}")]
    Open(String, std::io::Error),
    #[error("malformed file: {0}")]
    Malformed(String),
    #[error("unrecognised format for {0}; use an Apple Health export.xml/.zip or the vitals CSV format")]
    UnknownFormat(String),
}
