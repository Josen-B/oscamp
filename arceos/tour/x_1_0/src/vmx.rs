use crate::vcpu::VmCpuRegisters;
use axlog::ax_println;
use axhal::mem::PhysAddr;

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
const MSR_IA32_EFER: u32 = 0xC0000080; // Extended Feature Enable Register
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

const VMCS_GUEST_GDTR_BASE: u32 = 0x00006816;
const VMCS_GUEST_GDTR_LIMIT: u32 = 0x00004810;
const VMCS_GUEST_IDTR_BASE: u32 = 0x00006818;
const VMCS_GUEST_IDTR_LIMIT: u32 = 0x00004812;

const VMCS_GUEST_LINK_POINTER: u32 = 0x00002800;
const VMCS_IA32_EFER: u32 = 0x00002802; // Guest IA32_EFER (VMCS encoding)

const VMCS_ENTRY_MSR_LOAD_COUNT: u32 = 0x00004010;
const VMCS_ENTRY_MSR_LOAD_ADDR: u32 = 0x0000400C;
const VMCS_EXIT_MSR_STORE_COUNT: u32 = 0x00004012;
const VMCS_EXIT_MSR_STORE_ADDR: u32 = 0x0000400E;
const VMCS_EXIT_MSR_LOAD_COUNT: u32 = 0x00004014;
const VMCS_EXIT_MSR_LOAD_ADDR: u32 = 0x00004016;

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
const VMCS_HOST_IA32_PAT: u32 = 0x00002C00;
const VMCS_HOST_IA32_EFER: u32 = 0x00002C02;

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
        ax_println!("VMXON region (virt): {:#x}", vmxon_region);

        // Convert to physical address
        let vmxon_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(vmxon_region as usize));
        ax_println!("VMXON region (phys): {:#x}", usize::from(vmxon_paddr));

        // Execute VMXON with physical address
        let success = vmxon_phys(usize::from(vmxon_paddr));
        if !success {
            alloc::alloc::dealloc(
                vmxon_region as *mut u8,
                alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
            );
            panic!("VMXON failed!");
        }

        ax_println!("VMXON successful");
    }
}

/// VMXON instruction (expects physical address)
unsafe fn vmxon_phys(phys_addr: usize) -> bool {
    let mut cr4 = read_cr4();
    cr4 |= 1 << 13; // Set VMXE bit
    write_cr4(cr4);
    
    // Execute VMXON and check carry flag
    // According to Intel SDM: VMXON m64, where m64 is a memory location containing the 64-bit physical address
    let phys_addr_ptr = &phys_addr as *const usize as *const u8;
    let success: u8;
    core::arch::asm!(
        "vmxon [{0}]",
        "jnc 2f",
        "mov {1}, 0",
        "jmp 3f",
        "2: mov {1}, 1",
        "3:",
        in(reg) phys_addr_ptr,
        out(reg_byte) success,
        options(nostack)
    );
    
    ax_println!("VMXON {}", if success != 0 { "succeeded" } else { "failed" });
    success != 0
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
    let vmcs_phys = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(vmcs as usize));
    let phys_addr_ptr = &usize::from(vmcs_phys) as *const usize as *const u8;
    core::arch::asm!(
        "vmclear [{0}]",
        in(reg) phys_addr_ptr,
        options(nostack)
    );
}

/// VMPTRLD instruction
unsafe fn vmptrld(vmcs: u64) {
    let vmcs_phys = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(vmcs as usize));
    let phys_addr_ptr = &usize::from(vmcs_phys) as *const usize as *const u8;
    core::arch::asm!(
        "vmptrld [{0}]",
        in(reg) phys_addr_ptr,
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
    let value: u64;
    core::arch::asm!(
        "vmread {}, {}",
        out(reg) value,
        in(reg) field as u64,
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
    // MSR format: bits 31:0 = 1-settings (configurable), bits 63:32 = 0-settings (must-be-1)
    let pin_ctls_msr = read_msr(MSR_IA32_VMX_PINBASED_CTLS);
    // Start with 1-settings (low 32 bits) as defaults
    let pin_ctls = pin_ctls_msr as u32;
    // OR-in all must-be-1 bits (high 32 bits)
    let pin_ctls = pin_ctls | ((pin_ctls_msr >> 32) as u32);
    vmwrite(VMCS_PIN_BASED_VM_EXEC_CONTROL, pin_ctls as u64);

    // Primary processor-based VM-execution controls
    let proc_ctls_msr = read_msr(MSR_IA32_VMX_PROCBASED_CTLS);
    ax_println!("  Proc-based MSR: {:#x}", proc_ctls_msr);
    ax_println!("  Proc-based MSR low: {:#x}, high: {:#x}",
        proc_ctls_msr as u32, (proc_ctls_msr >> 32) as u32);
    // MSR format: bits 31:0 = 1-settings, bits 63:32 = 0-settings
    // Start with 1-settings (low 32 bits) as defaults
    let mut proc_ctls = proc_ctls_msr as u32;
    // OR-in all must-be-1 bits (high 32 bits)
    proc_ctls |= (proc_ctls_msr >> 32) as u32;
    ax_println!("  Initial proc_ctls: {:#x}", proc_ctls);
    ax_println!("  Check: bit 7 (unrestricted guest) set in proc_ctls? {}", (proc_ctls & (1 << 7)) != 0);

    // Enable use of EPT (configurable bit)
    proc_ctls |= (1 << 31);
    ax_println!("  EPT ENABLED");

    // Disable unrestricted guest mode (require standard guest paging)
    // proc_ctls |= (1 << 7); // UNRESTRICTED GUEST DISABLED
    ax_println!("  Final proc_ctls (UNRESTRICTED GUEST DISABLED, EPT enabled): {:#x}", proc_ctls);
    ax_println!("  Check: bit 7 (unrestricted guest) set? {}", (proc_ctls & (1 << 7)) != 0);
    ax_println!("  Check: bit 31 (EPT) set? {}", (proc_ctls & (1 << 31)) != 0);

    // Enable HLT exiting (so we can catch guest's shutdown)
    proc_ctls |= (1 << 18); // HLT exiting
    proc_ctls |= (1 << 20); // INVLPG exiting
    proc_ctls |= (1 << 22); // RDTSC exiting

    vmwrite(VMCS_PRIMARY_PROC_BASED_VM_EXEC_CONTROL, proc_ctls as u64);

    // Exception bitmap - all exceptions cause VM-exit
    vmwrite(VMCS_EXCEPTION_BITMAP, 0xFFFFFFFF);

    // VM-exit controls
    let exit_ctls_msr = read_msr(MSR_IA32_VMX_EXIT_CTLS);
    // MSR format: bits 31:0 = 1-settings, bits 63:32 = 0-settings
    // Start with 1-settings (low 32 bits) as defaults
    let mut exit_ctls = exit_ctls_msr as u32;
    // OR-in all must-be-1 bits (high 32 bits)
    exit_ctls |= (exit_ctls_msr >> 32) as u32;

    // Host address space size (64-bit) - requires HOST_IA32_EFER.LMA = 1
    exit_ctls |= (1 << 9);

    // CRITICAL: Load IA32_EFER on VM-exit (bit 21)
    // This is REQUIRED when Host address space size (bit 9) = 1
    exit_ctls |= (1 << 21);

    // Acknowledge interrupt on exit
    exit_ctls |= (1 << 15);

    ax_println!("  Exit controls (64-bit host, load IA32_EFER): {:#x}", exit_ctls);
    vmwrite(VMCR_VM_EXIT_CONTROLS, exit_ctls as u64);

    // VM-entry controls
    let entry_ctls_msr = read_msr(MSR_IA32_VMX_ENTRY_CTLS);
    ax_println!("VM-entry MSR: {:#x}", entry_ctls_msr);
    ax_println!("  MSR low 32: {:#x}, high 32: {:#x}",
        entry_ctls_msr as u32, (entry_ctls_msr >> 32) as u32);
    ax_println!("  MSR low 32 binary: {:b}", entry_ctls_msr as u32);
    ax_println!("  MSR high 32 binary: {:b}", (entry_ctls_msr >> 32) as u32);
    ax_println!("  Check: bit 2 in MSR low 32? {}", ((entry_ctls_msr >> 2) & 1) != 0);
    
    // MSR format according to Intel SDM:
    // - Bits 31:0 = "1-settings" - configurable bits (default to 1, can be cleared)
    // - Bits 63:32 = "0-settings" - must-be-1 bits (default to 0, must be set)
    
    // Start with 1-settings (low 32 bits) as defaults
    let mut entry_ctls = entry_ctls_msr as u32;
    ax_println!("  Initial entry_ctls (1-settings): {:#x}", entry_ctls);
    
    // OR-in all must-be-1 bits (high 32 bits)
    entry_ctls |= (entry_ctls_msr >> 32) as u32;
    ax_println!("  After OR-ing must-be-1 bits: {:#x}", entry_ctls);
    
    ax_println!("  Check: bit 2 in entry_ctls? {}", (entry_ctls & 4) != 0);

    // ENABLE IA-32e mode (64-bit long mode)
    entry_ctls |= (1 << 9); // Set IA-32e mode bit
    ax_println!("  IA-32e mode ENABLED (64-bit long mode with guest paging)");

    // ENABLE loading IA32_EFER (required for 64-bit mode)
    entry_ctls |= (1 << 2); // Set load IA32_EFER bit
    ax_println!("  IA32_EFER load bit ENABLED (required for 64-bit mode)");

    ax_println!("  Final entry_ctls in binary: {:b}", entry_ctls);
    ax_println!("  Check: bit 2 (load IA32_EFER) set? {}", (entry_ctls & 4) != 0);
    ax_println!("  Check: bit 9 (IA-32e mode) set? {}", (entry_ctls & 512) != 0);

    vmwrite(VMCR_VM_ENTRY_CONTROLS, entry_ctls as u64);
    let verify = vmread(VMCR_VM_ENTRY_CONTROLS);
    ax_println!("  Verify VM-entry controls after write: {:#x}", verify);
    if verify != entry_ctls as u64 {
        ax_println!("  WARNING: Write failed! Expected {:#x}, got {:#x}", entry_ctls, verify);
    }
    
    // CR3 target count
    vmwrite(VMCS_CR3_TARGET_COUNT, 0);

    // Page fault error code mask
    vmwrite(VMCS_PAGE_FAULT_ERROR_CODE_MASK, 0);
    vmwrite(VMCS_PAGE_FAULT_ERROR_CODE_MATCH, 0);

    // Setup MSR load area for VM-entry
    let msr_load_addr = create_msr_load_area();
    vmwrite(VMCS_ENTRY_MSR_LOAD_ADDR, msr_load_addr);
    vmwrite(VMCS_ENTRY_MSR_LOAD_COUNT, 1); // 1 MSR to load (IA32_EFER for 64-bit mode)

    // Set VM-exit MSR store/load to empty
    vmwrite(VMCS_EXIT_MSR_STORE_ADDR, 0);
    vmwrite(VMCS_EXIT_MSR_STORE_COUNT, 0);
    vmwrite(VMCS_EXIT_MSR_LOAD_ADDR, 0);
    vmwrite(VMCS_EXIT_MSR_LOAD_COUNT, 0);

    Ok(())
}

/// Setup VMCS guest state
unsafe fn setup_vmcs_guest_state(ctx: &mut VmCpuRegisters) -> Result<(), &'static str> {
    let gs = &ctx.guest_state;

    // Re-write VM-entry controls at start (workaround for bug)
    let entry_ctls_msr = read_msr(MSR_IA32_VMX_ENTRY_CTLS);
    // Start with 1-settings (low 32 bits)
    let mut entry_ctls = entry_ctls_msr as u32;
    // OR-in all must-be-1 bits (high 32 bits)
    entry_ctls |= (entry_ctls_msr >> 32) as u32;
    // Set IA-32e mode for 64-bit long mode
    entry_ctls |= (1 << 9);
    // Set load IA32_EFER for 64-bit mode
    entry_ctls |= (1 << 2);
    vmwrite(VMCR_VM_ENTRY_CONTROLS, entry_ctls as u64);
    ax_println!("Re-initialized VM-entry controls: {:#x} (binary: {:b})", entry_ctls, entry_ctls);
    ax_println!("  Bit 2 (load IA32_EFER): {}", (entry_ctls & 4) != 0);
    ax_println!("  Bit 9 (IA-32e mode): {}", (entry_ctls & 512) != 0);

    // Control registers
    ax_println!("Writing CR0: {:#x}, CR3: {:#x}, CR4: {:#x}", gs.cr0, gs.cr3, gs.cr4);
    vmwrite(VMCS_GUEST_CR0, gs.cr0);
    let check1 = vmread(VMCR_VM_ENTRY_CONTROLS);
    ax_println!("VM-entry controls after CR0: {:#x}", check1);

    vmwrite(VMCS_GUEST_CR3, gs.cr3);
    let check2 = vmread(VMCR_VM_ENTRY_CONTROLS);
    ax_println!("VM-entry controls after CR3: {:#x}", check2);

    vmwrite(VMCS_GUEST_CR4, gs.cr4);
    let check3 = vmread(VMCR_VM_ENTRY_CONTROLS);
    ax_println!("VM-entry controls after CR4: {:#x}", check3);

    // Save entry_ctls for later use
    let final_entry_ctls = entry_ctls;
    
    // RIP and RSP
    ax_println!("Writing guest RIP: {:#x}", gs.rip);
    vmwrite(VMCS_GUEST_RIP, gs.rip);
    let rip_check = vmread(VMCS_GUEST_RIP);
    ax_println!("Read back guest RIP: {:#x}", rip_check);

    vmwrite(VMCS_GUEST_RSP, gs.rsp);
    vmwrite(VMCS_GUEST_RFLAGS, gs.rflags);

    // For 64-bit long mode, IA32_EFER must have LMA and LME set
    // LME (bit 8) = 1 for Long Mode Enable
    // LMA (bit 10) = 1 for Long Mode Active
    // Value should be 0x0500 = 0x0400 | 0x0100
    let ia32_efer: u64 = 0x0500u64;
    vmwrite(0x2802, ia32_efer);
    ax_println!("VMCS_GUEST_IA32_EFER (0x2802) set to: {:#x} (64-bit long mode)", ia32_efer);
    let efer_check = vmread(0x2802);
    ax_println!("VMCS IA32_EFER read back: {:#x}", efer_check);

    // Check control fields (before writing CS)
    let entry_ctrl = vmread(VMCR_VM_ENTRY_CONTROLS);
    ax_println!("VM-entry controls: {:#x}", entry_ctrl);
    ax_println!("VM-entry controls binary: {:b}", entry_ctrl);
    ax_println!("  Load IA32_EFER bit (bit 2): {}", ((entry_ctrl >> 2) & 1) != 0);
    ax_println!("  IA-32e mode bit (bit 9): {}", ((entry_ctrl >> 9) & 1) != 0);

    // Check CR0 and CR4
    let cr0 = vmread(VMCS_GUEST_CR0);
    let cr4 = vmread(VMCS_GUEST_CR4);
    ax_println!("Guest CR0: {:#x} (PE={}, PG={})",
        cr0, (cr0 & 1) != 0, (cr0 >> 31) & 1 != 0);
    ax_println!("Guest CR4: {:#x} (PAE={})", cr4, (cr4 >> 5) & 1 != 0);

    // Set GDTR BEFORE setting CS selector (required for validation)
    vmwrite(VMCS_GUEST_GDTR_BASE, gs.gdtr_base);
    vmwrite(VMCS_GUEST_GDTR_LIMIT, gs.gdtr_limit as u64);
    ax_println!("GDTR set: base={:#x}, limit={:#x}", gs.gdtr_base, gs.gdtr_limit);
    
    // Verify and re-set GDTR limit (workaround)
    let gdtr_limit_check = vmread(VMCS_GUEST_GDTR_LIMIT);
    if gdtr_limit_check != gs.gdtr_limit as u64 {
        ax_println!("  WARNING: GDTR limit changed from {:#x} to {:#x}, re-setting...",
                     gs.gdtr_limit, gdtr_limit_check as u16);
        vmwrite(VMCS_GUEST_GDTR_LIMIT, gs.gdtr_limit as u64);
        let gdtr_limit_check2 = vmread(VMCS_GUEST_GDTR_LIMIT);
        ax_println!("  GDTR limit re-set to: {:#x} (read back: {:#x})",
                     gs.gdtr_limit, gdtr_limit_check2 as u16);
    }

    // Set IDTR before CS
    vmwrite(VMCS_GUEST_IDTR_BASE, gs.idtr_base);
    vmwrite(VMCS_GUEST_IDTR_LIMIT, gs.idtr_limit as u64);

    // Check CS selector and access rights (will be set later)
    let cs_sel = vmread(VMCS_GUEST_CS_SELECTOR);
    let cs_ar = vmread(VMCS_GUEST_CS_AR_BYTES);
    ax_println!("Guest CS selector before write: {:#x}, AR before write: {:#x}", cs_sel, cs_ar);
    ax_println!("Writing CS AR: {:#x} (64-bit code segment, L=1, D=0)", gs.cs_access_rights);
    ax_println!("  Expected CS AR binary: {:b}", gs.cs_access_rights);

    // Segment selectors
    vmwrite(VMCS_GUEST_ES_SELECTOR, gs.es_selector as u64);
    vmwrite(VMCS_GUEST_CS_SELECTOR, gs.cs_selector as u64);
    vmwrite(VMCS_GUEST_SS_SELECTOR, gs.ss_selector as u64);
    vmwrite(VMCS_GUEST_DS_SELECTOR, gs.ds_selector as u64);
    vmwrite(VMCS_GUEST_FS_SELECTOR, gs.fs_selector as u64);
    vmwrite(VMCS_GUEST_GS_SELECTOR, gs.gs_selector as u64);
    vmwrite(VMCS_GUEST_LDTR_SELECTOR, gs.ldtr_selector as u64);
    vmwrite(VMCS_GUEST_TR_SELECTOR, gs.tr_selector as u64);

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
    ax_println!("Writing CS limit: {:#x} (from gs.cs_limit)", gs.cs_limit);
    vmwrite(VMCS_GUEST_ES_LIMIT, gs.es_limit as u64);
    vmwrite(VMCS_GUEST_CS_LIMIT, gs.cs_limit as u64);
    let cs_limit_check = vmread(VMCS_GUEST_CS_LIMIT);
    ax_println!("CS limit after write: {:#x} (expected {:#x})", cs_limit_check, gs.cs_limit);
    vmwrite(VMCS_GUEST_SS_LIMIT, gs.ss_limit as u64);
    vmwrite(VMCS_GUEST_DS_LIMIT, gs.ds_limit as u64);
    vmwrite(VMCS_GUEST_FS_LIMIT, gs.fs_limit as u64);
    vmwrite(VMCS_GUEST_GS_LIMIT, gs.gs_limit as u64);
    vmwrite(VMCS_GUEST_LDTR_LIMIT, gs.ldtr_limit as u64);
    vmwrite(VMCS_GUEST_TR_LIMIT, gs.tr_limit as u64);

    // Segment access rights
    ax_println!("Writing CS AR: {:#x}", gs.cs_access_rights);
    vmwrite(VMCS_GUEST_ES_AR_BYTES, gs.es_access_rights);
    vmwrite(VMCS_GUEST_CS_AR_BYTES, gs.cs_access_rights);
    vmwrite(VMCS_GUEST_SS_AR_BYTES, gs.ss_access_rights);
    vmwrite(VMCS_GUEST_DS_AR_BYTES, gs.ds_access_rights);
    vmwrite(VMCS_GUEST_FS_AR_BYTES, gs.fs_access_rights);
    vmwrite(VMCS_GUEST_GS_AR_BYTES, gs.gs_access_rights);
    vmwrite(VMCS_GUEST_LDTR_AR_BYTES, gs.ldtr_access_rights);
    vmwrite(VMCS_GUEST_TR_AR_BYTES, gs.tr_access_rights);

    // Verify CS AR was written
    let cs_ar_check = vmread(VMCS_GUEST_CS_AR_BYTES);
    let cs_sel_check = vmread(VMCS_GUEST_CS_SELECTOR);
    ax_println!("CS AR after write: {:#x} (expected {:#x})", cs_ar_check, gs.cs_access_rights);
    ax_println!("CS selector after write: {:#x}", cs_sel_check);

    // Check all segments
    let ds_ar = vmread(VMCS_GUEST_DS_AR_BYTES);
    let es_ar = vmread(VMCS_GUEST_ES_AR_BYTES);
    let ss_ar = vmread(VMCS_GUEST_SS_AR_BYTES);
    let ldtr_ar = vmread(VMCS_GUEST_LDTR_AR_BYTES);
    let tr_ar = vmread(VMCS_GUEST_TR_AR_BYTES);
    let ldtr_sel = vmread(VMCS_GUEST_LDTR_SELECTOR);
    let tr_sel = vmread(VMCS_GUEST_TR_SELECTOR);
    ax_println!("DS AR: {:#x}, ES AR: {:#x}, SS AR: {:#x}", ds_ar, es_ar, ss_ar);
    ax_println!("LDTR selector: {:#x}, AR: {:#x}", ldtr_sel, ldtr_ar);
    ax_println!("TR selector: {:#x}, AR: {:#x}", tr_sel, tr_ar);

    // Check RFLAGS and activity state
    let rflags = vmread(VMCS_GUEST_RFLAGS);
    let activity = vmread(VMCS_GUEST_ACTIVITY_STATE);
    let interruptibility = vmread(VMCS_GUEST_INTERRUPTIBILITY_STATE);
    ax_println!("RFLAGS: {:#x}, Activity state: {:#x}, Interruptibility: {:#x}",
                 rflags, activity, interruptibility);

    // Check GDTR and IDTR
    let gdtr_base = vmread(VMCS_GUEST_GDTR_BASE);
    let gdtr_limit = vmread(VMCS_GUEST_GDTR_LIMIT);
    let idtr_base = vmread(VMCS_GUEST_IDTR_BASE);
    let idtr_limit = vmread(VMCS_GUEST_IDTR_LIMIT);
    ax_println!("GDTR: base={:#x}, limit={:#x}", gdtr_base, gdtr_limit);
    ax_println!("IDTR: base={:#x}, limit={:#x}", idtr_base, idtr_limit);

    // Check segment bases
    let cs_base = vmread(VMCS_GUEST_CS_BASE);
    let ds_base = vmread(VMCS_GUEST_DS_BASE);
    let es_base = vmread(VMCS_GUEST_ES_BASE);
    let ss_base = vmread(VMCS_GUEST_SS_BASE);
    ax_println!("CS base: {:#x}, DS base: {:#x}, ES base: {:#x}, SS base: {:#x}",
                 cs_base, ds_base, es_base, ss_base);

    // Read VM-instruction error before VMLAUNCH
    let vm_instr_error = vmread(VMCS_VM_INSTRUCTION_ERROR);
    ax_println!("VM-instruction error before VMLAUNCH: {:#x}", vm_instr_error);

    // Activity state and interruptibility
    vmwrite(VMCS_GUEST_ACTIVITY_STATE, gs.activity_state as u64);
    vmwrite(VMCS_GUEST_INTERRUPTIBILITY_STATE, gs.interruptibility_state as u64);

    // Link pointer
    vmwrite(VMCS_GUEST_LINK_POINTER, 0xFFFFFFFFFFFFFFFF);

    // Re-set VM-entry controls at end (workaround for bug)
    // Do this AFTER all other guest state fields are set
    let entry_ctls_msr = read_msr(MSR_IA32_VMX_ENTRY_CTLS);
    // Start with 1-settings (low 32 bits)
    let mut final_entry_ctls = entry_ctls_msr as u32;
    // OR-in all must-be-1 bits (high 32 bits)
    final_entry_ctls |= (entry_ctls_msr >> 32) as u32;
    // ENABLE IA-32e mode for 64-bit long mode
    final_entry_ctls |= (1 << 9);
    // ENABLE load IA32_EFER for 64-bit mode
    final_entry_ctls |= (1 << 2);
    vmwrite(VMCR_VM_ENTRY_CONTROLS, final_entry_ctls as u64);
    let final_check = vmread(VMCR_VM_ENTRY_CONTROLS);
    ax_println!("Re-set VM-entry controls before VMLAUNCH: {:#x}", final_entry_ctls);
    ax_println!("Final VM-entry controls read back: {:#x}", final_check);
    ax_println!("  Final bit 2 (load IA32_EFER): {}", (final_entry_ctls & 4) != 0);
    ax_println!("  Final bit 9 (IA-32e mode): {}", (final_entry_ctls & 512) != 0);

    // Verify CS AR one more time
    let cs_final = vmread(VMCS_GUEST_CS_AR_BYTES);
    ax_println!("CS AR final check: {:#x}", cs_final);

    // Re-set all segment limits at the very end (workaround for bug)
    ax_println!("Re-setting all segment limits (workaround)...");
    ax_println!("  gs.cs_limit = {:#x}", gs.cs_limit);
    // Don't re-set GDTR/IDTR limits here - they may get corrupted
    vmwrite(VMCS_GUEST_ES_LIMIT, gs.es_limit as u64);
    vmwrite(VMCS_GUEST_CS_LIMIT, gs.cs_limit as u64);
    let cs_limit_after_rewrite = vmread(VMCS_GUEST_CS_LIMIT);
    ax_println!("  CS limit after re-set: {:#x} (expected {:#x})", cs_limit_after_rewrite, gs.cs_limit);
    vmwrite(VMCS_GUEST_SS_LIMIT, gs.ss_limit as u64);
    vmwrite(VMCS_GUEST_DS_LIMIT, gs.ds_limit as u64);
    vmwrite(VMCS_GUEST_FS_LIMIT, gs.fs_limit as u64);
    vmwrite(VMCS_GUEST_GS_LIMIT, gs.gs_limit as u64);
    vmwrite(VMCS_GUEST_LDTR_LIMIT, gs.ldtr_limit as u64);
    vmwrite(VMCS_GUEST_TR_LIMIT, gs.tr_limit as u64);

    // Verify final values
    let gdtr_limit_final = vmread(VMCS_GUEST_GDTR_LIMIT);
    let idtr_limit_final = vmread(VMCS_GUEST_IDTR_LIMIT);
    let cs_limit_final = vmread(VMCS_GUEST_CS_LIMIT);
    ax_println!("Final limits - GDTR: {:#x}, IDTR: {:#x}, CS: {:#x}",
                 gdtr_limit_final as u16, idtr_limit_final as u16, cs_limit_final as u32);

    // Final GDTR/IDTR re-setting (workaround - do this AFTER segment limits)
    // This must be done at the very end of VMCS setup
    ax_println!("Final GDTR/IDTR re-setting...");
    vmwrite(VMCS_GUEST_GDTR_BASE, gs.gdtr_base);
    vmwrite(VMCS_GUEST_GDTR_LIMIT, gs.gdtr_limit as u64);
    vmwrite(VMCS_GUEST_IDTR_BASE, gs.idtr_base);
    vmwrite(VMCS_GUEST_IDTR_LIMIT, gs.idtr_limit as u64);
    
    // Final TR re-setting (do this LAST, after GDTR/IDTR)
    ax_println!("Final TR re-setting...");
    vmwrite(VMCS_GUEST_TR_SELECTOR, gs.tr_selector as u64);
    vmwrite(VMCS_GUEST_TR_BASE, gs.tr_base);
    vmwrite(VMCS_GUEST_TR_LIMIT, gs.tr_limit as u64);
    vmwrite(VMCS_GUEST_TR_AR_BYTES, gs.tr_access_rights);
    let tr_sel_final = vmread(VMCS_GUEST_TR_SELECTOR);
    let tr_ar_final = vmread(VMCS_GUEST_TR_AR_BYTES);
    let tr_base_final = vmread(VMCS_GUEST_TR_BASE);
    ax_println!("  TR after final re-set: selector={:#x}, AR={:#x}, base={:#x}", tr_sel_final, tr_ar_final, tr_base_final);
    ax_println!("  TR expected: selector={:#x}, AR={:#x}, base={:#x}", gs.tr_selector, gs.tr_access_rights, gs.tr_base);
    
    let gdtr_check = vmread(VMCS_GUEST_GDTR_LIMIT);
    let idtr_check = vmread(VMCS_GUEST_IDTR_LIMIT);
    ax_println!("Final check - GDTR limit: {:#x} (expected {:#x}), IDTR limit: {:#x} (expected {:#x})",
                 gdtr_check as u16, gs.gdtr_limit, idtr_check as u16, gs.idtr_limit);

    // Additional checks for VM-entry validation
    ax_println!("Checking all guest state fields for VM-entry validation:");
    
    // Check CR0, CR3, CR4
    let cr0_check = vmread(VMCS_GUEST_CR0);
    let cr3_check = vmread(VMCS_GUEST_CR3);
    let cr4_check = vmread(VMCS_GUEST_CR4);
    ax_println!("  CR0: {:#x} (expected {:#x}), CR3: {:#x} (expected {:#x}), CR4: {:#x} (expected {:#x})",
                 cr0_check, gs.cr0, cr3_check, gs.cr3, cr4_check, gs.cr4);
    
    // Check RFLAGS
    let rflags_check = vmread(VMCS_GUEST_RFLAGS);
    ax_println!("  RFLAGS: {:#x} (expected {:#x})", rflags_check, gs.rflags);
    
    // Check RIP
    let rip_check = vmread(VMCS_GUEST_RIP);
    ax_println!("  RIP: {:#x} (expected {:#x})", rip_check, gs.rip);
    
    // Check segments
    let cs_ar = vmread(VMCS_GUEST_CS_AR_BYTES);
    let ss_ar = vmread(VMCS_GUEST_SS_AR_BYTES);
    let tr_ar = vmread(VMCS_GUEST_TR_AR_BYTES);
    let ldtr_ar = vmread(VMCS_GUEST_LDTR_AR_BYTES);
    ax_println!("  CS AR: {:#x}, SS AR: {:#x}, TR AR: {:#x}, LDTR AR: {:#x}",
                 cs_ar, ss_ar, tr_ar, ldtr_ar);
    
    // Check GDTR, IDTR
    let gdtr_base = vmread(VMCS_GUEST_GDTR_BASE);
    let idtr_base = vmread(VMCS_GUEST_IDTR_BASE);
    ax_println!("  GDTR: base={:#x}, limit={:#x} (expected base={:#x}, limit={:#x})",
                 gdtr_base, gdtr_check as u16, gs.gdtr_base, gs.gdtr_limit);
    ax_println!("  IDTR: base={:#x}, limit={:#x} (expected base={:#x}, limit={:#x})",
                 idtr_base, idtr_check as u16, gs.idtr_base, gs.idtr_limit);
    
    // Check activity state and interruptibility
    let activity = vmread(VMCS_GUEST_ACTIVITY_STATE);
    let interruptibility = vmread(VMCS_GUEST_INTERRUPTIBILITY_STATE);
    ax_println!("  Activity state: {} (expected {}), Interruptibility: {} (expected {})",
                 activity, gs.activity_state, interruptibility, gs.interruptibility_state);
    
    // Check IA32_EFER
    let efer = vmread(0x2802);
    ax_println!("  IA32_EFER: {:#x} (expected 0x0500 for 64-bit mode)", efer);
    
    // Check link pointer
    let link = vmread(VMCS_GUEST_LINK_POINTER);
    ax_println!("  Link pointer: {:#x} (expected -1)", link);
    

    Ok(())
}

/// Create MSR load area for VM-entry
unsafe fn create_msr_load_area() -> u64 {
    // MSR load area format: Each entry is 16 bytes
    // Offset 0-3: MSR index
    // Offset 4-7: Reserved (MBZ)
    // Offset 8-15: MSR value

    let msr_area = alloc::alloc::alloc_zeroed(
        alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
    ) as u64;

    // For 64-bit long mode, load IA32_EFER on VM-entry
    // MSR load area format: Each entry is 16 bytes
    // Offset 0-3: MSR index (0xC0000080 for IA32_EFER)
    // Offset 4-7: Reserved (must be 0)
    // Offset 8-15: MSR value (0x1000 for LMA=1, LME=1)
    unsafe {
        *((msr_area + 0) as *mut u32) = MSR_IA32_EFER; // MSR index: IA32_EFER
        *((msr_area + 8) as *mut u64) = (1u64 << 8) | (1u64 << 10); // LME=1 (bit8), LMA=1 (bit10) = 0x0500
    }

    let msr_area_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(msr_area as usize));
    ax_println!("MSR load area at vaddr: {:#x}, paddr: {:#x}", msr_area, usize::from(msr_area_paddr));
    ax_println!("  MSR[0]: index={:#x}, value={:#x} (IA32_EFER with LMA=1, LME=1)", MSR_IA32_EFER, (1u64 << 8) | (1u64 << 10));

    msr_area_paddr.as_usize() as u64
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
    // Host RSP - use current stack pointer
    let mut rsp: u64;
    core::arch::asm!("mov {}, rsp", out(reg) rsp);
    vmwrite(VMCS_HOST_RSP, rsp);
    
    // CRITICAL: Set HOST_IA32_EFER.LMA = 1 for 64-bit host address space
    // This is REQUIRED when VM-exit controls bit 9 (Host address space size) = 1
    let host_efer = read_msr(MSR_IA32_EFER);
    ax_println!("  Current host IA32_EFER: {:#x}", host_efer);
    ax_println!("  Host IA32_EFER.LMA (bit 10): {}", ((host_efer >> 10) & 1) != 0);
    vmwrite(VMCS_HOST_IA32_EFER, host_efer);
    
    // Print host state for debugging
    ax_println!("  Host state debug:");
    ax_println!("    CS selector: {:#x}", cs);
    ax_println!("    DS selector: {:#x}", ds);
    ax_println!("    ES selector: {:#x}", es);
    ax_println!("    SS selector: {:#x}", ss);
    ax_println!("    FS selector: {:#x}", fs);
    ax_println!("    GS selector: {:#x}", gs);
    ax_println!("    TR selector: {:#x}", tr);
    ax_println!("    CR0: {:#x}", host_cr0);
    ax_println!("    CR3: {:#x}", cr3);
    ax_println!("    CR4: {:#x}", host_cr4);
    ax_println!("    RSP: {:#x}", rsp);
    
    Ok(())
}

/// Setup EPT pointer
unsafe fn setup_ept_pointer(ept_root: axhal::mem::PhysAddr) -> Result<(), &'static str> {
    // EPT pointer format (Intel SDM):
    // Bits 51:12 = EPT PML4 address
    // Bits 5:3 = EPT page-walk length - 1 (3 for 4-level EPT)
    // Bits 2:0 = Memory type (6 = Write-Back for code execution)
    let eptp = (usize::from(ept_root) & 0x000FFFF_FFFFF000) | (3 << 3) | 6;
    vmwrite(VMCS_EPT_POINTER, eptp as u64);

    ax_println!("EPT pointer set to: {:#x}", eptp);
    ax_println!("  EPTP format: PML4={:#x}, walk_len=3, mem_type=6 (WB)", ept_root.as_usize());
    Ok(())
}

/// Launch the VM
pub fn vmx_launch(ctx: &mut VmCpuRegisters) {
    unsafe {
        ax_println!("Launching VM...");
        
        // Set up host RSP in VMCS
        let mut host_rsp: u64;
        core::arch::asm!(
            "mov {0}, rsp",
            out(reg) host_rsp,
        );
        vmwrite(VMCS_HOST_RSP, host_rsp);
        
        // Set host RIP to instruction after VMLAUNCH
        let mut host_rip: u64;
        core::arch::asm!(
            "lea {0}, [rip + 2f]",
            "2:",
            out(reg) host_rip,
        );
        vmwrite(VMCS_HOST_RIP, host_rip);
        
        // Execute VMLAUNCH directly
        ax_println!("Executing VMLAUNCH...");
        core::arch::asm!(
            "vmlaunch",
            "2:",
        );
        
        // If we reach here, VM-exit occurred
        ax_println!("VM exit occurred");

        // Check VM-instruction error after VMLAUNCH
        let vm_instr_error_after = vmread(VMCS_VM_INSTRUCTION_ERROR);
        ax_println!("VM-instruction error after VMLAUNCH: {:#x}", vm_instr_error_after);

        // If VM-instruction error is non-zero, VMLAUNCH failed
        if vm_instr_error_after != 0 {
            ax_println!("ERROR: VMLAUNCH failed with instruction error {:#x}", vm_instr_error_after);
            ax_println!("This means VM-entry validation failed");
            return;
        }

        ax_println!("VMLAUNCH succeeded, handling VM-exit");

        // Read guest state from VMCS
        ctx.guest_state.rip = vmread(VMCS_GUEST_RIP);
        ctx.guest_state.rsp = vmread(VMCS_GUEST_RSP);
        ctx.guest_state.rflags = vmread(VMCS_GUEST_RFLAGS);
        ax_println!("Guest RIP after exit: {:#x}", ctx.guest_state.rip);

        // Read VM-exit reason and qualification
        let exit_reason_raw = vmread(VMCS_EXIT_REASON);
        ax_println!("VM-exit reason raw: {:#x}", exit_reason_raw);
        ctx.guest_state.exit_reason = exit_reason_raw as u32;
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
