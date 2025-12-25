use crate::vcpu::VmCpuRegisters;
use axlog::ax_println;

/// VMX MSR numbers
const MSR_IA32_VMX_BASIC: u32 = 0x480;
const MSR_IA32_VMX_PINBASED_CTLS: u32 = 0x481;
const MSR_IA32_VMX_PROCBASED_CTLS: u32 = 0x482;
const MSR_IA32_VMX_EXIT_CTLS: u32 = 0x483;
const MSR_IA32_VMX_ENTRY_CTLS: u32 = 0x484;
const MSR_IA32_VMX_CR0_FIXED0: u32 = 0x486;
const MSR_IA32_VMX_CR0_FIXED1: u32 = 0x487;
const MSR_IA32_VMX_CR4_FIXED0: u32 = 0x488;
const MSR_IA32_VMX_CR4_FIXED1: u32 = 0x489;
const MSR_IA32_FEATURE_CONTROL: u32 = 0x3A;

/// VMCS field encodings
const VMCS_GUEST_ES_SELECTOR: u32 = 0x00000800;
const VMCS_GUEST_CS_SELECTOR: u32 = 0x00000802;
const VMCS_GUEST_SS_SELECTOR: u32 = 0x00000804;
const VMCS_GUEST_DS_SELECTOR: u32 = 0x00000806;
const VMCS_GUEST_FS_SELECTOR: u32 = 0x00000808;
const VMCS_GUEST_GS_SELECTOR: u32 = 0x0000080A;
const VMCS_GUEST_LDTR_SELECTOR: u32 = 0x0000080C;
const VMCS_GUEST_TR_SELECTOR: u32 = 0x0000080E;

const VMCS_HOST_ES_SELECTOR: u32 = 0x00000C00;
const VMCS_HOST_CS_SELECTOR: u32 = 0x00000C02;
const VMCS_HOST_SS_SELECTOR: u32 = 0x00000C04;
const VMCS_HOST_DS_SELECTOR: u32 = 0x00000C06;
const VMCS_HOST_FS_SELECTOR: u32 = 0x00000C08;
const VMCS_HOST_GS_SELECTOR: u32 = 0x00000C0A;
const VMCS_HOST_TR_SELECTOR: u32 = 0x00000C0C;

const VMCS_16BIT_CONTROL_FIELDS: u32 = 0x00004000;
const VMCS_64BIT_CONTROL_FIELDS: u32 = 0x00004002;
const VMCS_16BIT_GUEST_STATE_FIELDS: u32 = 0x00008000;
const VMCS_64BIT_GUEST_STATE_FIELDS: u32 = 0x00008002;
const VMCS_32BIT_CONTROL_FIELDS: u32 = 0x00004002;
const VMCS_32BIT_GUEST_STATE_FIELDS: u32 = 0x00008002;
const VMCS_NATURAL_WIDTH_CONTROL_FIELDS: u32 = 0x00004000;
const VMCS_NATURAL_WIDTH_GUEST_STATE_FIELDS: u32 = 0x00008000;

// Control fields
const VMCS_PIN_BASED_VM_EXEC_CONTROL: u32 = 0x00004000;
const VMCS_PRIMARY_PROC_BASED_VM_EXEC_CONTROL: u32 = 0x00004002;
const VMCS_EXCEPTION_BITMAP: u32 = 0x00004004;
const VMCR_VM_EXIT_CONTROLS: u32 = 0x0000400C;
const VMCR_VM_ENTRY_CONTROLS: u32 = 0x00004012;
const VMCS_CR3_TARGET_COUNT: u32 = 0x0000400E;
const VMCS_PAGE_FAULT_ERROR_CODE_MASK: u32 = 0x00004014;
const VMCS_PAGE_FAULT_ERROR_CODE_MATCH: u32 = 0x00004016;

// Guest state fields
const VMCS_GUEST_CR0: u32 = 0x00006800;
const VMCS_GUEST_CR3: u32 = 0x00006802;
const VMCS_GUEST_CR4: u32 = 0x00006804;
const VMCS_GUEST_RSP: u32 = 0x0000681C;
const VMCS_GUEST_RIP: u32 = 0x0000681E;
const VMCS_GUEST_RFLAGS: u32 = 0x00006820;

const VMCS_GUEST_ES_BASE: u32 = 0x00006806;
const VMCS_GUEST_CS_BASE: u32 = 0x00006808;
const VMCS_GUEST_SS_BASE: u32 = 0x0000680A;
const VMCS_GUEST_DS_BASE: u32 = 0x0000680C;
const VMCS_GUEST_FS_BASE: u32 = 0x0000680E;
const VMCS_GUEST_GS_BASE: u32 = 0x00006810;
const VMCS_GUEST_LDTR_BASE: u32 = 0x00006812;
const VMCS_GUEST_TR_BASE: u32 = 0x00006814;

const VMCS_GUEST_ES_LIMIT: u32 = 0x00004800;
const VMCS_GUEST_CS_LIMIT: u32 = 0x00004802;
const VMCS_GUEST_SS_LIMIT: u32 = 0x00004804;
const VMCS_GUEST_DS_LIMIT: u32 = 0x00004806;
const VMCS_GUEST_FS_LIMIT: u32 = 0x00004808;
const VMCS_GUEST_GS_LIMIT: u32 = 0x0000480A;
const VMCS_GUEST_LDTR_LIMIT: u32 = 0x0000480C;
const VMCS_GUEST_TR_LIMIT: u32 = 0x0000480E;

const VMCS_GUEST_ES_AR_BYTES: u32 = 0x00004814;
const VMCS_GUEST_CS_AR_BYTES: u32 = 0x00004816;
const VMCS_GUEST_SS_AR_BYTES: u32 = 0x00004818;
const VMCS_GUEST_DS_AR_BYTES: u32 = 0x0000481A;
const VMCS_GUEST_FS_AR_BYTES: u32 = 0x0000481C;
const VMCS_GUEST_GS_AR_BYTES: u32 = 0x0000481E;
const VMCS_GUEST_LDTR_AR_BYTES: u32 = 0x00004820;
const VMCS_GUEST_TR_AR_BYTES: u32 = 0x00004822;

const VMCS_GUEST_ACTIVITY_STATE: u32 = 0x00004826;
const VMCS_GUEST_INTERRUPTIBILITY_STATE: u32 = 0x00004824;

const VMCS_GUEST_GDTR_BASE: u32 = 0x0000681C;
const VMCS_GUEST_GDTR_LIMIT: u32 = 0x0000480A;
const VMCS_GUEST_IDTR_BASE: u32 = 0x0000681E;
const VMCS_GUEST_IDTR_LIMIT: u32 = 0x0000480C;

const VMCS_GUEST_LINK_POINTER: u32 = 0x00002800;

// Host state fields
const VMCS_HOST_CR0: u32 = 0x00006C00;
const VMCS_HOST_CR3: u32 = 0x00006C02;
const VMCS_HOST_CR4: u32 = 0x00006C04;
const VMCS_HOST_RSP: u32 = 0x00006C1C;
const VMCS_HOST_RIP: u32 = 0x00006C1E;

const VMCS_HOST_ES_BASE: u32 = 0x00006C06;
const VMCS_HOST_CS_BASE: u32 = 0x00006C08;
const VMCS_HOST_SS_BASE: u32 = 0x00006C0A;
const VMCS_HOST_DS_BASE: u32 = 0x00006C0C;
const VMCS_HOST_FS_BASE: u32 = 0x00006C0E;
const VMCS_HOST_GS_BASE: u32 = 0x00006C10;
const VMCS_HOST_TR_BASE: u32 = 0x00006C12;

// VM-exit information fields
const VMCS_EXIT_REASON: u32 = 0x00006402;
const VMCS_EXIT_QUALIFICATION: u32 = 0x00006400;
const VMCS_GUEST_LINEAR_ADDRESS: u32 = 0x0000640A;
const VMCS_GUEST_PHYSICAL_ADDRESS: u32 = 0x00002400;
const VMCS_VM_INSTRUCTION_ERROR: u32 = 0x00004400;
const VMCS_INSTRUCTION_LEN: u32 = 0x0000440C;

// EPT pointer
const VMCS_EPT_POINTER: u32 = 0x0000201A;

/// Check if VMX is supported
pub fn check_vmx_support() -> bool {
    let mut eax: u32 = 1;
    let mut ecx: u32;
    let mut edx: u32;
    unsafe {
        core::arch::asm!(
            "cpuid",
            inlateout("eax") eax,
            lateout("ecx") ecx,
            lateout("edx") edx,
        );
    }
    
    // Check ECX bit 5 (VMX)
    if ecx & (1 << 5) == 0 {
        return false;
    }
    
    // Check if VMX is enabled in IA32_FEATURE_CONTROL MSR
    unsafe {
        let feature_control = read_msr(MSR_IA32_FEATURE_CONTROL);
        if feature_control & 1 == 0 {
            // BIOS didn't lock it, we can enable VMX
            write_msr(MSR_IA32_FEATURE_CONTROL, feature_control | (1 << 2));
        } else if feature_control & (1 << 2) == 0 {
            // BIOS locked but VMX not enabled
            return false;
        }
    }
    
    true
}

/// Read MSR
unsafe fn read_msr(msr: u32) -> u64 {
    let (high, low): (u32, u32);
    core::arch::asm!(
        "rdmsr",
        inlateout("ecx") msr => _,
        lateout("eax") low,
        lateout("edx") high,
        options(nomem, nostack)
    );
    ((high as u64) << 32) | (low as u64)
}

/// Write MSR
unsafe fn write_msr(msr: u32, value: u64) {
    let low = (value & 0xFFFFFFFF) as u32;
    let high = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nostack)
    );
}

/// Read CR0
#[inline]
unsafe fn read_cr0() -> u64 {
    let value: u64;
    core::arch::asm!(
        "mov {}, cr0",
        out(reg) value,
        options(nomem, nostack, pure)
    );
    value
}

/// Write CR0
#[inline]
unsafe fn write_cr0(value: u64) {
    core::arch::asm!(
        "mov cr0, {}",
        in(reg) value,
        options(nostack)
    );
}

/// Read CR4
#[inline]
unsafe fn read_cr4() -> u64 {
    let value: u64;
    core::arch::asm!(
        "mov {}, cr4",
        out(reg) value,
        options(nomem, nostack, pure)
    );
    value
}

/// Write CR4
#[inline]
unsafe fn write_cr4(value: u64) {
    core::arch::asm!(
        "mov cr4, {}",
        in(reg) value,
        options(nostack)
    );
}

/// Enable VMX operation in CR4
unsafe fn enable_vmx_cr4() {
    let mut cr4 = read_cr4();
    cr4 |= (1 << 13); // Set VMXE bit
    write_cr4(cr4);
}

/// Read CR3
#[inline]
unsafe fn read_cr3() -> u64 {
    let value: u64;
    core::arch::asm!(
        "mov {}, cr3",
        out(reg) value,
        options(nomem, nostack, pure)
    );
    value
}

/// Initialize VMX
pub fn vmx_init() {
    ax_println!("Initializing VMX...");
    
    unsafe {
        // Enable VMX in CR4
        enable_vmx_cr4();
        
        // Allocate VMXON region (4KB aligned)
        let vmxon_region = alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as u64;
        
        // Set revision ID in VMXON region
        let vmx_basic = read_msr(MSR_IA32_VMX_BASIC);
        let revision_id = vmx_basic & 0x7FFFFFFF;
        *(vmxon_region as *mut u32) = revision_id as u32;
        
        ax_println!("VMX revision: {:#x}", revision_id);
        ax_println!("VMXON region: {:#x}", vmxon_region);
        
        // Execute VMXON
        let cf = vmxon(vmxon_region);
        if cf {
            alloc::alloc::dealloc(
                vmxon_region as *mut u8,
                alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
            );
            panic!("VMXON failed!");
        }
        
        ax_println!("VMXON successful");
    }
}

/// VMXON instruction
unsafe fn vmxon(phys_addr: u64) -> bool {
    let mut cr4 = read_cr4();
    cr4 |= 1 << 13; // Set VMXE bit
    write_cr4(cr4);
    
    let rflags: u64;
    core::arch::asm!(
        "vmxon [{}]",
        in(reg) &phys_addr,
        lateout("eax") rflags,
        options(nostack)
    );
    
    // Check carry flag (bit 0 of rflags after vmxon)
    // If CF=0, VMXON succeeded; if CF=1, it failed
    (rflags & 0x1) == 0
}

/// VMXOFF instruction
unsafe fn vmxoff() {
    core::arch::asm!(
        "vmxoff",
        options(nostack)
    );
}

/// Clean up VMX
pub fn vmx_cleanup() {
    ax_println!("Cleaning up VMX...");
    unsafe {
        vmxoff();
    }
    ax_println!("VMX cleanup complete");
}

/// Allocate VMCS
unsafe fn allocate_vmcs() -> u64 {
    let vmcs = alloc::alloc::alloc_zeroed(
        alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
    ) as u64;
    
    // Set revision ID
    let vmx_basic = read_msr(MSR_IA32_VMX_BASIC);
    let revision_id = vmx_basic & 0x7FFFFFFF;
    *(vmcs as *mut u32) = revision_id as u32;
    
    vmcs
}

/// VMCLEAR instruction
unsafe fn vmclear(vmcs: u64) {
    core::arch::asm!(
        "vmclear [{}]",
        in(reg) &vmcs,
        options(nostack)
    );
}

/// VMPTRLD instruction
unsafe fn vmptrld(vmcs: u64) {
    core::arch::asm!(
        "vmptrld [{}]",
        in(reg) &vmcs,
        options(nostack)
    );
}

/// VMWRITE instruction
unsafe fn vmwrite(field: u32, value: u64) {
    core::arch::asm!(
        "vmwrite {}, {}",
        in(reg) field as u64,
        in(reg) value,
        options(nostack)
    );
}

/// VMREAD instruction
unsafe fn vmread(field: u32) -> u64 {
    let mut value: u64;
    core::arch::asm!(
        "vmread {}, {}",
        inlateout(reg) field as u64 => _,
        lateout(reg) value,
        options(nostack)
    );
    value
}

/// Setup VMCS for the guest
pub fn setup_vmcs(ctx: &mut VmCpuRegisters, ept_root: axhal::mem::PhysAddr) -> Result<(), &'static str> {
    ax_println!("Setting up VMCS...");
    
    unsafe {
        let vmcs = allocate_vmcs();
        ax_println!("Allocated VMCS at: {:#x}", vmcs);
        
        // Clear and load VMCS
        vmclear(vmcs);
        vmptrld(vmcs);
        
        // Setup control fields
        setup_vmcs_control_fields()?;
        
        // Setup guest state
        setup_vmcs_guest_state(ctx)?;
        
        // Setup host state
        setup_vmcs_host_state()?;
        
        // Setup EPT pointer
        setup_ept_pointer(ept_root)?;
        
        ax_println!("VMCS setup complete");
    }
    
    Ok(())
}

/// Setup VMCS control fields
unsafe fn setup_vmcs_control_fields() -> Result<(), &'static str> {
    // Pin-based VM-execution controls
    let pin_ctls_msr = read_msr(MSR_IA32_VMX_PINBASED_CTLS);
    let pin_ctls = (pin_ctls_msr as u32) & 0x0000FFFF; // 0s are 1-settings
    vmwrite(VMCS_PIN_BASED_VM_EXEC_CONTROL, pin_ctls as u64);
    
    // Primary processor-based VM-execution controls
    let proc_ctls_msr = read_msr(MSR_IA32_VMX_PROCBASED_CTLS);
    let mut proc_ctls = (proc_ctls_msr as u32) & 0x0000FFFF;
    
    // Enable use of EPT
    proc_ctls |= (1 << 31);
    
    // Enable HLT exiting (so we can catch guest's shutdown)
    proc_ctls |= (1 << 18); // HLT exiting
    proc_ctls |= (1 << 20); // INVLPG exiting
    proc_ctls |= (1 << 22); // RDTSC exiting
    proc_ctls |= (1 << 0);  // Interrupt window exiting
    
    // Ensure all required bits are set (0-setting bits)
    let required_0_bits = (proc_ctls_msr >> 32) as u32;
    proc_ctls |= required_0_bits;
    
    // Clear all 1-setting bits we don't want
    let required_1_bits = (proc_ctls_msr & 0xFFFFFFFF) as u32;
    proc_ctls &= required_1_bits | !0xFFFFFFFF;
    
    vmwrite(VMCS_PRIMARY_PROC_BASED_VM_EXEC_CONTROL, proc_ctls as u64);
    
    // Exception bitmap - all exceptions cause VM-exit
    vmwrite(VMCS_EXCEPTION_BITMAP, 0xFFFFFFFF);
    
    // VM-exit controls
    let exit_ctls_msr = read_msr(MSR_IA32_VMX_EXIT_CTLS);
    let mut exit_ctls = (exit_ctls_msr as u32) & 0x0000FFFF;
    
    // Host address space size (64-bit)
    exit_ctls |= (1 << 9);
    
    // Acknowledge interrupt on exit
    exit_ctls |= (1 << 15);
    
    // Ensure all required bits are set
    let exit_required_0_bits = (exit_ctls_msr >> 32) as u32;
    exit_ctls |= exit_required_0_bits;
    
    vmwrite(VMCR_VM_EXIT_CONTROLS, exit_ctls as u64);
    
    // VM-entry controls
    let entry_ctls_msr = read_msr(MSR_IA32_VMX_ENTRY_CTLS);
    let mut entry_ctls = (entry_ctls_msr as u32) & 0x0000FFFF;
    
    // IA-32e mode guest (64-bit)
    entry_ctls |= (1 << 9);
    
    // Ensure all required bits are set
    let entry_required_0_bits = (entry_ctls_msr >> 32) as u32;
    entry_ctls |= entry_required_0_bits;
    
    vmwrite(VMCR_VM_ENTRY_CONTROLS, entry_ctls as u64);
    
    // CR3 target count
    vmwrite(VMCS_CR3_TARGET_COUNT, 0);
    
    // Page fault error code mask
    vmwrite(VMCS_PAGE_FAULT_ERROR_CODE_MASK, 0);
    vmwrite(VMCS_PAGE_FAULT_ERROR_CODE_MATCH, 0);
    
    Ok(())
}

/// Setup VMCS guest state
unsafe fn setup_vmcs_guest_state(ctx: &mut VmCpuRegisters) -> Result<(), &'static str> {
    let gs = &ctx.guest_state;
    
    // Control registers
    vmwrite(VMCS_GUEST_CR0, gs.cr0);
    vmwrite(VMCS_GUEST_CR3, gs.cr3);
    vmwrite(VMCS_GUEST_CR4, gs.cr4);
    
    // RIP and RSP
    vmwrite(VMCS_GUEST_RIP, gs.rip);
    vmwrite(VMCS_GUEST_RSP, gs.rsp);
    vmwrite(VMCS_GUEST_RFLAGS, gs.rflags);
    
    // Segment selectors
    vmwrite(VMCS_GUEST_ES_SELECTOR, gs.es_selector as u64);
    vmwrite(VMCS_GUEST_CS_SELECTOR, gs.cs_selector as u64);
    vmwrite(VMCS_GUEST_SS_SELECTOR, gs.ss_selector as u64);
    vmwrite(VMCS_GUEST_DS_SELECTOR, gs.ds_selector as u64);
    vmwrite(VMCS_GUEST_FS_SELECTOR, gs.fs_selector as u64);
    vmwrite(VMCS_GUEST_GS_SELECTOR, gs.gs_selector as u64);
    vmwrite(VMCS_GUEST_LDTR_SELECTOR, gs.ldtr_selector as u64);
    vmwrite(VMCS_GUEST_TR_SELECTOR, gs.tr_selector as u64);
    
    // Segment bases
    vmwrite(VMCS_GUEST_ES_BASE, gs.es_base);
    vmwrite(VMCS_GUEST_CS_BASE, gs.cs_base);
    vmwrite(VMCS_GUEST_SS_BASE, gs.ss_base);
    vmwrite(VMCS_GUEST_DS_BASE, gs.ds_base);
    vmwrite(VMCS_GUEST_FS_BASE, gs.fs_base);
    vmwrite(VMCS_GUEST_GS_BASE, gs.gs_base);
    vmwrite(VMCS_GUEST_LDTR_BASE, gs.ldtr_base);
    vmwrite(VMCS_GUEST_TR_BASE, gs.tr_base);
    
    // Segment limits
    vmwrite(VMCS_GUEST_ES_LIMIT, gs.es_limit as u64);
    vmwrite(VMCS_GUEST_CS_LIMIT, gs.cs_limit as u64);
    vmwrite(VMCS_GUEST_SS_LIMIT, gs.ss_limit as u64);
    vmwrite(VMCS_GUEST_DS_LIMIT, gs.ds_limit as u64);
    vmwrite(VMCS_GUEST_FS_LIMIT, gs.fs_limit as u64);
    vmwrite(VMCS_GUEST_GS_LIMIT, gs.gs_limit as u64);
    vmwrite(VMCS_GUEST_LDTR_LIMIT, gs.ldtr_limit as u64);
    vmwrite(VMCS_GUEST_TR_LIMIT, gs.tr_limit as u64);
    
    // Segment access rights
    vmwrite(VMCS_GUEST_ES_AR_BYTES, gs.es_access_rights);
    vmwrite(VMCS_GUEST_CS_AR_BYTES, gs.cs_access_rights);
    vmwrite(VMCS_GUEST_SS_AR_BYTES, gs.ss_access_rights);
    vmwrite(VMCS_GUEST_DS_AR_BYTES, gs.ds_access_rights);
    vmwrite(VMCS_GUEST_FS_AR_BYTES, gs.fs_access_rights);
    vmwrite(VMCS_GUEST_GS_AR_BYTES, gs.gs_access_rights);
    vmwrite(VMCS_GUEST_LDTR_AR_BYTES, gs.ldtr_access_rights);
    vmwrite(VMCS_GUEST_TR_AR_BYTES, gs.tr_access_rights);
    
    // GDTR and IDTR
    vmwrite(VMCS_GUEST_GDTR_BASE, gs.gdtr_base);
    vmwrite(VMCS_GUEST_GDTR_LIMIT, gs.gdtr_limit as u64);
    vmwrite(VMCS_GUEST_IDTR_BASE, gs.idtr_base);
    vmwrite(VMCS_GUEST_IDTR_LIMIT, gs.idtr_limit as u64);
    
    // Activity state and interruptibility
    vmwrite(VMCS_GUEST_ACTIVITY_STATE, gs.activity_state as u64);
    vmwrite(VMCS_GUEST_INTERRUPTIBILITY_STATE, gs.interruptibility_state as u64);
    
    // Link pointer
    vmwrite(VMCS_GUEST_LINK_POINTER, 0xFFFFFFFFFFFFFFFF);
    
    Ok(())
}

/// Setup VMCS host state
unsafe fn setup_vmcs_host_state() -> Result<(), &'static str> {
    // Read current CR0, CR3, CR4
    let cr0 = read_cr0();
    let cr3 = read_cr3();
    let cr4 = read_cr4();
    
    // Ensure proper CR0/CR4 values for VMX operation
    let cr0_fixed0 = read_msr(MSR_IA32_VMX_CR0_FIXED0) as u64;
    let cr0_fixed1 = read_msr(MSR_IA32_VMX_CR0_FIXED1) as u64;
    let host_cr0 = (cr0 | cr0_fixed0) & cr0_fixed1;
    
    let cr4_fixed0 = read_msr(MSR_IA32_VMX_CR4_FIXED0) as u64;
    let cr4_fixed1 = read_msr(MSR_IA32_VMX_CR4_FIXED1) as u64;
    let host_cr4 = (cr4 | cr4_fixed0) & cr4_fixed1;
    
    vmwrite(VMCS_HOST_CR0, host_cr0);
    vmwrite(VMCS_HOST_CR3, cr3);
    vmwrite(VMCS_HOST_CR4, host_cr4);
    
    // Get current segment selectors (assume flat model)
    let mut cs: u16;
    let mut ds: u16;
    let mut es: u16;
    let mut fs: u16;
    let mut gs: u16;
    let mut ss: u16;
    let mut tr: u16;
    
    core::arch::asm!(
        "mov {0:x}, cs",
        "mov {1:x}, ds",
        "mov {2:x}, es",
        "mov {3:x}, fs",
        "mov {4:x}, gs",
        "mov {5:x}, ss",
        "str {6:x}",
        out(reg) cs,
        out(reg) ds,
        out(reg) es,
        out(reg) fs,
        out(reg) gs,
        out(reg) ss,
        out(reg) tr,
    );
    
    vmwrite(VMCS_HOST_CS_SELECTOR, cs as u64);
    vmwrite(VMCS_HOST_DS_SELECTOR, ds as u64);
    vmwrite(VMCS_HOST_ES_SELECTOR, es as u64);
    vmwrite(VMCS_HOST_FS_SELECTOR, fs as u64);
    vmwrite(VMCS_HOST_GS_SELECTOR, gs as u64);
    vmwrite(VMCS_HOST_SS_SELECTOR, ss as u64);
    vmwrite(VMCS_HOST_TR_SELECTOR, tr as u64);
    
    // Segment bases (flat model = 0)
    vmwrite(VMCS_HOST_ES_BASE, 0);
    vmwrite(VMCS_HOST_CS_BASE, 0);
    vmwrite(VMCS_HOST_SS_BASE, 0);
    vmwrite(VMCS_HOST_DS_BASE, 0);
    vmwrite(VMCS_HOST_FS_BASE, 0);
    vmwrite(VMCS_HOST_GS_BASE, 0);
    vmwrite(VMCS_HOST_TR_BASE, 0);
    
    // Host RIP will be set by VMLAUNCH automatically to instruction after VMLAUNCH
    // Host RSP - we'll use the current stack
    
    Ok(())
}

/// Setup EPT pointer
unsafe fn setup_ept_pointer(ept_root: axhal::mem::PhysAddr) -> Result<(), &'static str> {
    // EPT pointer format: [63:52] reserved, [51:12] PML4 address, [11:3] 0, [2:0] memory type
    // For simplicity, use write-back memory type (6)
    let eptp = (usize::from(ept_root) & 0x000FFFF_FFFFF000) | 6;
    vmwrite(VMCS_EPT_POINTER, eptp as u64);
    
    ax_println!("EPT pointer set to: {:#x}", eptp);
    Ok(())
}

/// Launch the VM
pub fn vmx_launch(ctx: &mut VmCpuRegisters) {
    unsafe {
        let mut rip: u64;
        let mut rsp: u64;
        let mut rflags: u64;
        
        // Save host state
        core::arch::asm!(
            "mov {0}, rsp",
            out(reg) rsp,
        );
        
        // Set up host RSP in VMCS
        vmwrite(VMCS_HOST_RSP, rsp);
        
        // Set host RIP to instruction after VMLAUNCH
        let mut host_rip: u64;
        core::arch::asm!(
            "lea {0}, [rip + 2f]",
            "2:",
            out(reg) host_rip,
        );
        vmwrite(VMCS_HOST_RIP, host_rip);
        
        ax_println!("Launching VM...");
        
        // Execute VMLAUNCH
        let mut cf: u8;
        core::arch::asm!(
            "3:",
            "vmlaunch",
            "4:",
            "setc {al}",
            al = out(reg_byte) cf,
        );
        
        // If we reach here, VM-exit occurred
        ax_println!("VM exit occurred");
        
        // Read guest state from VMCS
        ctx.guest_state.rip = vmread(VMCS_GUEST_RIP);
        ctx.guest_state.rsp = vmread(VMCS_GUEST_RSP);
        ctx.guest_state.rflags = vmread(VMCS_GUEST_RFLAGS);
        
        // Read VM-exit reason and qualification
        ctx.guest_state.exit_reason = vmread(VMCS_EXIT_REASON) as u32;
        ctx.guest_state.exit_qualification = vmread(VMCS_EXIT_QUALIFICATION);
        ctx.guest_state.guest_linear_address = vmread(VMCS_GUEST_LINEAR_ADDRESS);
        ctx.guest_state.guest_physical_address = vmread(VMCS_GUEST_PHYSICAL_ADDRESS);
        
        // Handle the VM-exit
        vmexit_handler(ctx);
    }
}

/// Handle VM-exit
pub fn vmexit_handler(ctx: &mut VmCpuRegisters) {
    let exit_reason = ctx.guest_state.exit_reason & 0x7FFF; // Basic exit reason
    let exit_qual = ctx.guest_state.exit_qualification;
    
    ax_println!("VM Exit - Reason: {:#x} ({})", exit_reason, exit_reason_to_string(exit_reason));
    ax_println!("  Exit qualification: {:#x}", exit_qual);
    ax_println!("  Guest RIP: {:#x}", ctx.guest_state.rip);
    
    match exit_reason {
        12 => {
            // Triple fault - guest is shutting down
            ax_println!("Triple fault - guest shutting down");
            // We'll exit normally
        }
        10 => {
            // CR access
            ax_println!("CR access");
            // For simplicity, just ignore and advance RIP
            unsafe { advance_guest_rip(ctx); }
        }
        28 => {
            // I/O instruction
            ax_println!("I/O instruction");
            // For simplicity, ignore and advance RIP
            unsafe { advance_guest_rip(ctx); }
        }
        21 => {
            // RDMSR
            ax_println!("RDMSR");
            // Return 0 for simplicity
            let ecx = (ctx.guest_state.rcx & 0xFFFFFFFF) as u32;
            ax_println!("  RDMSR ECX: {}", ecx);
            unsafe { advance_guest_rip(ctx); }
        }
        22 => {
            // WRMSR
            ax_println!("WRMSR");
            // Ignore for simplicity
            unsafe { advance_guest_rip(ctx); }
        }
        48 => {
            // HLT instruction
            ax_println!("HLT - guest is halting");
            // Exit normally
        }
        _ => {
            ax_println!("Unhandled VM exit reason: {}", exit_reason);
            // For this basic implementation, we'll just exit
        }
    }
}

/// Advance guest RIP after handling an instruction
unsafe fn advance_guest_rip(ctx: &mut VmCpuRegisters) {
    let inst_len = vmread(VMCS_INSTRUCTION_LEN) as u64;
    ax_println!("Advancing RIP by {} bytes", inst_len);
    ctx.guest_state.rip += inst_len;
    vmwrite(VMCS_GUEST_RIP, ctx.guest_state.rip);
}

/// Convert exit reason to string
fn exit_reason_to_string(reason: u32) -> &'static str {
    match reason {
        0 => "EXCEPTION_NMI",
        1 => "EXTERNAL_INTERRUPT",
        2 => "TRIPLE_FAULT",
        3 => "INIT_SIGNAL",
        4 => "STARTUP_IPI",
        5 => "IO_SMI",
        6 => "OTHER_SMI",
        7 => "INTERRUPT_WINDOW",
        8 => "NMI_WINDOW",
        9 => "TASK_SWITCH",
        10 => "CPUID",
        12 => "HLT",
        28 => "INVLPG",
        30 => "RDPMC",
        31 => "RDTSC",
        32 => "RSM",
        33 => "VMCALL",
        34 => "VMCLEAR",
        35 => "VMLAUNCH",
        36 => "VMPTRLD",
        37 => "VMPTRST",
        38 => "VMREAD",
        39 => "VMRESUME",
        40 => "VMWRITE",
        41 => "VMXOFF",
        42 => "VMXON",
        44 => "CR_ACCESS",
        45 => "DR_ACCESS",
        46 => "IO_INSTRUCTION",
        48 => "RDMSR",
        49 => "WRMSR",
        50 => "VM_ENTRY_FAIL",
        51 => "VM_ENTRY_FAIL_MSR_LOAD",
        52 => "MWAIT",
        53 => "MTF",
        54 => "MONITOR",
        55 => "PAUSE",
        56 => "VM_ENTRY_FAIL_MC",
        57 => "TPR_BELOW_THRESHOLD",
        58 => "APIC_ACCESS",
        59 => "VIRTUALIZED_EOI",
        60 => "GDTR_IDTR",
        61 => "LDTR_TR",
        62 => "EPT_VIOLATION",
        63 => "EPT_MISCONFIG",
        64 => "INVEPT",
        65 => "RDTSCP",
        66 => "VMX_PREEMPT_TIMER",
        67 => "INVVPID",
        68 => "WBINVD",
        69 => "XSETBV",
        _ => "UNKNOWN",
    }
}
