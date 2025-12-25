use core::mem::size_of;

/// Guest CPU state that must be saved/restored when entering/exiting a VM.
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct GuestState {
    // General purpose registers
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    // Control registers
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    
    // Instruction pointer and flags
    pub rip: u64,
    pub rflags: u64,
    
    // Segment registers
    pub cs_selector: u64,
    pub cs_base: u64,
    pub cs_limit: u64,
    pub cs_access_rights: u64,
    
    pub ds_selector: u64,
    pub ds_base: u64,
    pub ds_limit: u64,
    pub ds_access_rights: u64,
    
    pub es_selector: u64,
    pub es_base: u64,
    pub es_limit: u64,
    pub es_access_rights: u64,
    
    pub fs_selector: u64,
    pub fs_base: u64,
    pub fs_limit: u64,
    pub fs_access_rights: u64,
    
    pub gs_selector: u64,
    pub gs_base: u64,
    pub gs_limit: u64,
    pub gs_access_rights: u64,
    
    pub ss_selector: u64,
    pub ss_base: u64,
    pub ss_limit: u64,
    pub ss_access_rights: u64,
    
    pub ldtr_selector: u64,
    pub ldtr_base: u64,
    pub ldtr_limit: u64,
    pub ldtr_access_rights: u64,
    
    pub tr_selector: u64,
    pub tr_base: u64,
    pub tr_limit: u64,
    pub tr_access_rights: u64,
    
    // GDTR and IDTR
    pub gdtr_base: u64,
    pub gdtr_limit: u64,
    pub idtr_base: u64,
    pub idtr_limit: u64,
    
    // VM-exit information
    pub exit_reason: u32,
    pub exit_qualification: u64,
    pub guest_linear_address: u64,
    pub guest_physical_address: u64,
    
    // Interrupt state
    pub activity_state: u32,
    pub interruptibility_state: u32,
    
    // Pending debug exceptions
    pub pending_debug_exceptions: u64,
    
    // VMCS link pointer
    pub vmcs_link_pointer: u64,
}

/// Hypervisor CPU state that must be saved/restored when entering/exiting a VM.
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct HypervisorState {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

/// (v)CPU register state that must be saved or restored when entering/exiting a VM.
#[derive(Default)]
#[repr(C)]
pub struct VmCpuRegisters {
    pub guest_state: GuestState,
    pub hypervisor_state: HypervisorState,
}

#[allow(dead_code)]
fn guest_reg_offset(reg: &str) -> usize {
    match reg {
        "rax" => core::mem::offset_of!(GuestState, rax),
        "rcx" => core::mem::offset_of!(GuestState, rcx),
        "rdx" => core::mem::offset_of!(GuestState, rdx),
        "rbx" => core::mem::offset_of!(GuestState, rbx),
        "rsp" => core::mem::offset_of!(GuestState, rsp),
        "rbp" => core::mem::offset_of!(GuestState, rbp),
        "rsi" => core::mem::offset_of!(GuestState, rsi),
        "rdi" => core::mem::offset_of!(GuestState, rdi),
        "r8" => core::mem::offset_of!(GuestState, r8),
        "r9" => core::mem::offset_of!(GuestState, r9),
        "r10" => core::mem::offset_of!(GuestState, r10),
        "r11" => core::mem::offset_of!(GuestState, r11),
        "r12" => core::mem::offset_of!(GuestState, r12),
        "r13" => core::mem::offset_of!(GuestState, r13),
        "r14" => core::mem::offset_of!(GuestState, r14),
        "r15" => core::mem::offset_of!(GuestState, r15),
        "rip" => core::mem::offset_of!(GuestState, rip),
        "rflags" => core::mem::offset_of!(GuestState, rflags),
        _ => 0,
    }
}
