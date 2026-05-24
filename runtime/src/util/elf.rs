/// ELF loader and binary analysis
use anyhow::Result;

pub enum ElfClass {
    Elf32,
    Elf64,
}

pub enum Abi {
    Linux,
    Bionic,
}

/// Analyze an ELF binary to determine its ABI and requirements
pub fn analyze_elf(path: &str) -> Result<ElfAnalysis> {
    let data = std::fs::read(path)?;
    let elf = goblin::elf::Elf::parse(&data)?;

    let is_dynamic = !elf.dynamic.is_empty() || elf.libraries.is_some();

    Ok(ElfAnalysis {
        class: match elf.is_64 {
            true => ElfClass::Elf64,
            false => ElfClass::Elf32,
        },
        is_dynamic,
        interpreter: elf.interpreter.map(|s| s.to_string()),
        needed_libs: elf.libraries.unwrap_or_default().iter()
            .map(|s| s.to_string())
            .collect(),
        entry: elf.entry,
        has_gnu_stack: elf.flags & 0x1000000 != 0,
        abi: detect_abi(&elf, &data),
    })
}

pub struct ElfAnalysis {
    pub class: ElfClass,
    pub is_dynamic: bool,
    pub interpreter: Option<String>,
    pub needed_libs: Vec<String>,
    pub entry: u64,
    pub has_gnu_stack: bool,
    pub abi: Abi,
}

fn detect_abi(elf: &goblin::elf::Elf, _data: &[u8]) -> Abi {
    // Check for bionic-specific markers
    // bionic libc uses __aeabi_* functions, specific DT tags
    if let Some(note) = elf.note_headers.iter().next() {
        if note.name.ok() == Some("Android") {
            return Abi::Bionic;
        }
    }
    // Check for glibc by looking at interpreter
    if let Some(interp) = &elf.interpreter {
        if interp.contains("ld-linux") {
            return Abi::Linux;
        }
    }
    // Default to Linux/glibc ABI
    Abi::Linux
}
