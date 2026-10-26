//! Importers. Each returns parsed samples plus counts for the import log.

pub mod apple;
pub mod csv;

use std::{fs::File, io::BufReader, path::Path};

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

/// Which importer handled a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    AppleHealthXml,
    AppleHealthZip,
    VitalsCsv,
}

/// Detect the format from the extension (and, for zips, the entry name) and
/// parse the file.
pub fn load(path: &Path) -> Result<(Format, Parsed), ImportError> {
    let label = path.display().to_string();
    let open = || File::open(path).map_err(|e| ImportError::Open(label.clone(), e));
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "xml" => Ok((Format::AppleHealthXml, apple::parse(BufReader::new(open()?))?)),
        "csv" => {
            let source = path.file_stem().and_then(|s| s.to_str()).unwrap_or("csv");
            Ok((Format::VitalsCsv, csv::parse(BufReader::new(open()?), source)?))
        }
        "zip" => {
            let mut archive = zip::ZipArchive::new(open()?).map_err(|e| ImportError::Malformed(e.to_string()))?;
            // The iPhone export is apple_health_export/export.xml (localised
            // exports keep the same file name); skip export_cda.xml.
            let index = (0..archive.len())
                .find(|&i| archive.name_for_index(i).is_some_and(|n| n.ends_with("/export.xml") || n == "export.xml"))
                .ok_or_else(|| ImportError::Malformed("zip has no export.xml".into()))?;
            let entry = archive.by_index(index).map_err(|e| ImportError::Malformed(e.to_string()))?;
            Ok((Format::AppleHealthZip, apple::parse(BufReader::new(entry))?))
        }
        _ => Err(ImportError::UnknownFormat(label)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("vitals-import-{}-{name}", std::process::id()))
    }

    const XML: &str = r#"<HealthData><Record type="HKQuantityTypeIdentifierStepCount" sourceName="Watch" unit="count" startDate="2026-10-01 08:00:00 +0000" endDate="2026-10-01 08:10:00 +0000" value="100"/></HealthData>"#;

    #[test]
    fn loads_an_iphone_style_zip_export() {
        let path = tmp("export.zip");
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        let opts = zip::write::SimpleFileOptions::default();
        zip.start_file("apple_health_export/export_cda.xml", opts).unwrap();
        zip.write_all(b"<ClinicalDocument/>").unwrap();
        zip.start_file("apple_health_export/export.xml", opts).unwrap();
        zip.write_all(XML.as_bytes()).unwrap();
        zip.finish().unwrap();
        let (format, parsed) = load(&path).unwrap();
        assert_eq!(format, Format::AppleHealthZip);
        assert_eq!(parsed.samples.len(), 1);
    }

    #[test]
    fn unknown_extensions_are_rejected_with_guidance() {
        let path = tmp("data.parquet");
        std::fs::write(&path, b"x").unwrap();
        let err = load(&path).unwrap_err().to_string();
        assert!(err.contains("vitals CSV format"), "{err}");
    }
}
