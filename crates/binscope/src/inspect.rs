//! Parse a binary into one format-independent [`Binary`] model.

use goblin::{Object, mach::Mach};
use serde::Serialize;

use crate::entropy;

/// Container formats binscope understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    Elf,
    Pe,
    MachO,
    /// A Mach-O universal ("fat") binary; the first architecture is described.
    MachOUniversal,
}

/// One section, normalised across formats.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Section {
    pub name: String,
    /// Offset of the section's bytes in the file.
    pub file_offset: u64,
    /// Bytes stored in the file (0 for zero-fill sections like `.bss`).
    pub file_size: u64,
    pub virtual_address: u64,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    /// Entropy of the section's bytes in the file, bits per byte.
    pub entropy: f64,
}

/// A symbol a binary imports, with the library that provides it when known.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Import {
    pub name: String,
    pub library: Option<String>,
}

/// Everything binscope knows about a file.
#[derive(Debug, Clone, Serialize)]
pub struct Binary {
    pub format: Format,
    pub architecture: String,
    pub bits: u8,
    pub little_endian: bool,
    pub entry_point: u64,
    pub file_size: u64,
    /// No local symbol table (debug/function names removed).
    pub stripped: bool,
    pub libraries: Vec<String>,
    pub sections: Vec<Section>,
    pub imports: Vec<Import>,
    pub exports: Vec<String>,
    /// Architectures in a universal binary; empty otherwise.
    pub universal_architectures: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum InspectError {
    #[error("not a recognised executable format (ELF, PE or Mach-O)")]
    UnknownFormat,
    #[error("archives (.a/.lib) are not supported; inspect the member objects instead")]
    Archive,
    #[error("malformed binary: {0}")]
    Malformed(String),
}

impl From<goblin::error::Error> for InspectError {
    fn from(e: goblin::error::Error) -> Self {
        InspectError::Malformed(e.to_string())
    }
}

fn slice(bytes: &[u8], offset: u64, size: u64) -> &[u8] {
    let start = usize::try_from(offset).unwrap_or(usize::MAX).min(bytes.len());
    let end = start.saturating_add(usize::try_from(size).unwrap_or(usize::MAX)).min(bytes.len());
    &bytes[start..end]
}

fn section(
    bytes: &[u8],
    name: String,
    offset: u64,
    size: u64,
    vaddr: u64,
    r: bool,
    w: bool,
    x: bool,
) -> Section {
    Section {
        name,
        file_offset: offset,
        file_size: size,
        virtual_address: vaddr,
        readable: r,
        writable: w,
        executable: x,
        entropy: entropy::shannon(slice(bytes, offset, size)),
    }
}

/// Parse `bytes` (the whole file).
pub fn parse(bytes: &[u8]) -> Result<Binary, InspectError> {
    let file_size = bytes.len() as u64;
    match Object::parse(bytes)? {
        Object::Elf(elf) => {
            use goblin::elf::section_header::{SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE, SHT_NOBITS};
            let sections = elf
                .section_headers
                .iter()
                .filter(|sh| sh.sh_type != goblin::elf::section_header::SHT_NULL)
                .map(|sh| {
                    let name = elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("").to_string();
                    let stored = if sh.sh_type == SHT_NOBITS { 0 } else { sh.sh_size };
                    section(
                        bytes,
                        name,
                        sh.sh_offset,
                        stored,
                        sh.sh_addr,
                        sh.sh_flags & u64::from(SHF_ALLOC) != 0,
                        sh.sh_flags & u64::from(SHF_WRITE) != 0,
                        sh.sh_flags & u64::from(SHF_EXECINSTR) != 0,
                    )
                })
                .collect();
            let mut imports = Vec::new();
            let mut exports = Vec::new();
            for sym in elf.dynsyms.iter() {
                let Some(name) = elf.dynstrtab.get_at(sym.st_name).filter(|n| !n.is_empty()) else {
                    continue;
                };
                if sym.is_import() {
                    imports.push(Import { name: name.to_string(), library: None });
                } else if sym.st_bind() != goblin::elf::sym::STB_LOCAL && sym.st_value != 0 {
                    exports.push(name.to_string());
                }
            }
            Ok(Binary {
                format: Format::Elf,
                architecture: goblin::elf::header::machine_to_str(elf.header.e_machine).to_string(),
                bits: if elf.is_64 { 64 } else { 32 },
                little_endian: elf.little_endian,
                entry_point: elf.entry,
                file_size,
                stripped: elf.syms.is_empty(),
                libraries: elf.libraries.iter().map(|s| s.to_string()).collect(),
                sections,
                imports: sorted(imports),
                exports: sorted(exports),
                universal_architectures: Vec::new(),
            })
        }
        Object::PE(pe) => {
            use goblin::pe::section_table::{IMAGE_SCN_MEM_EXECUTE, IMAGE_SCN_MEM_READ, IMAGE_SCN_MEM_WRITE};
            let sections = pe
                .sections
                .iter()
                .map(|s| {
                    section(
                        bytes,
                        s.name().unwrap_or("").to_string(),
                        u64::from(s.pointer_to_raw_data),
                        u64::from(s.size_of_raw_data),
                        u64::from(s.virtual_address),
                        s.characteristics & IMAGE_SCN_MEM_READ != 0,
                        s.characteristics & IMAGE_SCN_MEM_WRITE != 0,
                        s.characteristics & IMAGE_SCN_MEM_EXECUTE != 0,
                    )
                })
                .collect();
            let imports = pe
                .imports
                .iter()
                .map(|i| Import { name: i.name.to_string(), library: Some(i.dll.to_string()) })
                .collect();
            let exports = pe.exports.iter().filter_map(|e| e.name.map(str::to_string)).collect();
            Ok(Binary {
                format: Format::Pe,
                architecture: goblin::pe::header::machine_to_str(pe.header.coff_header.machine).to_string(),
                bits: if pe.is_64 { 64 } else { 32 },
                little_endian: true,
                entry_point: pe.entry as u64,
                file_size,
                stripped: pe.header.coff_header.number_of_symbol_table == 0 && pe.debug_data.is_none(),
                libraries: pe.libraries.iter().map(|s| s.to_string()).collect(),
                sections,
                imports: sorted(imports),
                exports: sorted(exports),
                universal_architectures: Vec::new(),
            })
        }
        Object::Mach(Mach::Binary(macho)) => mach(bytes, &macho, Format::MachO, Vec::new()),
        Object::Mach(Mach::Fat(fat)) => {
            let arches = fat.arches()?;
            let names: Vec<String> = arches
                .iter()
                .map(|a| {
                    goblin::mach::constants::cputype::get_arch_name_from_types(a.cputype, a.cpusubtype)
                        .unwrap_or("unknown")
                        .to_string()
                })
                .collect();
            let first = arches.first().ok_or(InspectError::UnknownFormat)?;
            let inner = first.slice(bytes);
            match fat.get(0)? {
                goblin::mach::SingleArch::MachO(macho) => {
                    let mut b = mach(inner, &macho, Format::MachOUniversal, names)?;
                    b.file_size = file_size;
                    Ok(b)
                }
                goblin::mach::SingleArch::Archive(_) => Err(InspectError::Archive),
            }
        }
        Object::Archive(_) => Err(InspectError::Archive),
        _ => Err(InspectError::UnknownFormat),
    }
}

fn mach(
    bytes: &[u8],
    macho: &goblin::mach::MachO,
    format: Format,
    universal_architectures: Vec<String>,
) -> Result<Binary, InspectError> {
    let mut sections = Vec::new();
    for seg in macho.segments.iter() {
        let prot = seg.initprot;
        for (s, _) in seg.sections()? {
            sections.push(section(
                bytes,
                format!("{},{}", s.segname().unwrap_or(""), s.name().unwrap_or("")),
                u64::from(s.offset),
                if s.offset == 0 { 0 } else { s.size },
                s.addr,
                prot & 1 != 0,
                prot & 2 != 0,
                prot & 4 != 0,
            ));
        }
    }
    let imports = macho
        .imports()
        .unwrap_or_default()
        .into_iter()
        .map(|i| Import { name: i.name.to_string(), library: Some(i.dylib.to_string()) })
        .collect();
    let exports = macho.exports().unwrap_or_default().into_iter().map(|e| e.name).collect();
    Ok(Binary {
        format,
        architecture: goblin::mach::constants::cputype::get_arch_name_from_types(
            macho.header.cputype(),
            macho.header.cpusubtype(),
        )
        .unwrap_or("unknown")
        .to_string(),
        bits: if macho.is_64 { 64 } else { 32 },
        little_endian: macho.little_endian,
        entry_point: macho.entry,
        file_size: bytes.len() as u64,
        stripped: macho.symbols.is_none(),
        libraries: macho.libs.iter().filter(|l| **l != "self").map(|s| s.to_string()).collect(),
        sections,
        imports: sorted(imports),
        exports: sorted(exports),
        universal_architectures,
    })
}

fn sorted<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    v.sort();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_binaries_with_a_clear_error() {
        let err = parse(b"just some text, not an executable").unwrap_err();
        assert!(matches!(err, InspectError::UnknownFormat | InspectError::Malformed(_)), "{err}");
    }

    #[test]
    fn parses_the_running_test_binary() {
        let me = std::fs::read(std::env::current_exe().unwrap()).unwrap();
        let b = parse(&me).unwrap();
        assert_eq!(b.file_size, me.len() as u64);
        assert!(!b.sections.is_empty());
        assert!(b.sections.iter().any(|s| s.executable), "some section holds code");
        assert!(b.sections.iter().all(|s| (0.0..=8.0).contains(&s.entropy)));
        #[cfg(target_os = "linux")]
        assert_eq!(b.format, Format::Elf);
    }
}
