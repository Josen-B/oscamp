/// Index of x86_64 general purpose registers.
#[allow(missing_docs)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GprIndex {
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl GprIndex {
    /// Get register index from raw value.
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0 => Some(GprIndex::RAX),
            1 => Some(GprIndex::RCX),
            2 => Some(GprIndex::RDX),
            3 => Some(GprIndex::RBX),
            4 => Some(GprIndex::RSP),
            5 => Some(GprIndex::RBP),
            6 => Some(GprIndex::RSI),
            7 => Some(GprIndex::RDI),
            8 => Some(GprIndex::R8),
            9 => Some(GprIndex::R9),
            10 => Some(GprIndex::R10),
            11 => Some(GprIndex::R11),
            12 => Some(GprIndex::R12),
            13 => Some(GprIndex::R13),
            14 => Some(GprIndex::R14),
            15 => Some(GprIndex::R15),
            _ => None,
        }
    }
}

impl core::fmt::Display for GprIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GprIndex::RAX => write!(f, "rax"),
            GprIndex::RCX => write!(f, "rcx"),
            GprIndex::RDX => write!(f, "rdx"),
            GprIndex::RBX => write!(f, "rbx"),
            GprIndex::RSP => write!(f, "rsp"),
            GprIndex::RBP => write!(f, "rbp"),
            GprIndex::RSI => write!(f, "rsi"),
            GprIndex::RDI => write!(f, "rdi"),
            GprIndex::R8 => write!(f, "r8"),
            GprIndex::R9 => write!(f, "r9"),
            GprIndex::R10 => write!(f, "r10"),
            GprIndex::R11 => write!(f, "r11"),
            GprIndex::R12 => write!(f, "r12"),
            GprIndex::R13 => write!(f, "r13"),
            GprIndex::R14 => write!(f, "r14"),
            GprIndex::R15 => write!(f, "r15"),
        }
    }
}
