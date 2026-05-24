/// Syscall translation between architectures and ABIs
/// Handles ARM64 <-> x86_64 translation for FEX/Box64 integration
/// Also handles bionic <-> glibc syscall ABI differences

use anyhow::Result;

/// Translate a syscall from one architecture to another
pub struct SyscallTranslator {
    mode: TranslationMode,
}

pub enum TranslationMode {
    /// Bionic (Android) syscalls - direct pass-through
    BionicNative,
    /// Glibc syscalls that need translation for bionic ABI
    GlibcCompat,
    /// x86_64 emulated syscalls (for FEX/Box64)
    X86_64Emu,
}

impl SyscallTranslator {
    pub fn new(mode: TranslationMode) -> Self {
        Self { mode }
    }

    /// Translate syscall number from guest to host
    pub fn translate_nr(&self, guest_nr: i64) -> (i64, TranslationAction) {
        match self.mode {
            TranslationMode::BionicNative => (guest_nr, TranslationAction::Passthrough),
            TranslationMode::GlibcCompat => self.translate_glibc_to_bionic(guest_nr),
            TranslationMode::X86_64Emu => self.translate_x86_64_to_arm64(guest_nr),
        }
    }

    /// Translate glibc-style syscalls to bionic ABI
    /// bionic uses ARM64 standard syscall numbers directly
    /// glibc may use different calling conventions
    fn translate_glibc_to_bionic(&self, nr: i64) -> (i64, TranslationAction) {
        match nr {
            // Most syscalls are identical between bionic and glibc on ARM64
            _ => (nr, TranslationAction::Passthrough),
        }
    }

    /// Translate x86_64 syscall numbers to ARM64
    fn translate_x86_64_to_arm64(&self, nr: i64) -> (i64, TranslationAction) {
        match nr {
            // x86_64 -> ARM64 syscall mapping
            0 => (63, TranslationAction::Passthrough),     // read
            1 => (64, TranslationAction::Passthrough),     // write
            2 => (63, TranslationAction::Passthrough),     // open -> openat wrapper
            3 => (57, TranslationAction::Passthrough),     // close
            9 => (222, TranslationAction::Passthrough),    // mmap
            10 => (226, TranslationAction::Passthrough),   // mprotect
            12 => (215, TranslationAction::Passthrough),   // brk
            56 => (220, TranslationAction::CloneWrapper),  // clone
            57 => (221, TranslationAction::Passthrough),   // fork -> execve (vfork)
            59 => (221, TranslationAction::ExecWrapper),   // execve
            _ => (nr, TranslationAction::Passthrough),
        }
    }
}

pub enum TranslationAction {
    Passthrough,
    CloneWrapper,
    ExecWrapper,
    FstatWrapper,
    ErrnoTransform(i32),
}
