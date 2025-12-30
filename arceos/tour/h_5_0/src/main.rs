#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
extern crate axstd as std;
extern crate alloc;
#[macro_use]
extern crate axlog;

mod task;
mod vcpu;
mod regs;
mod vmx;
mod loader;

use vcpu::VmCpuRegisters;
use vmx::vmexit_handler;

const VM_ENTRY: u64 = 0x10_0000;

fn create_gdt() -> (u64, u64) {
    // Allocate a page for GDT
    let gdt_vaddr = unsafe {
        alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as *mut u8 as u64
    };

    // GDT entry format: [base 31:24][access][limit 19:16][base 23:16][base 15:0][limit 15:0]
    // Entry 0: Null descriptor
    unsafe {
        *((gdt_vaddr + 0) as *mut u64) = 0;
    }

    // Entry 1: 32-bit code segment (selector = 0x8) - changed from 64-bit
    // Base=0, Limit=0xFFFFFFFF, Present=1, DPL=0, System=1, Type=Execute/Read, Accessed=1, L=0, D=1
    unsafe {
        *((gdt_vaddr + 8) as *mut u64) = 0x00CF9B000000FFFF; // Limit=0xFFFF, Base=0, Access=0x9B, Flags=0xC
    }

    // Entry 2: Data segment (selector = 0x10)
    // Base=0, Limit=0xFFFFFFFF, Present=1, DPL=0, System=1, Type=Read/Write, Accessed=1
    unsafe {
        *((gdt_vaddr + 16) as *mut u64) = 0x00CF93000000FFFF;
    }

    let gdt_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(gdt_vaddr as usize));

    (gdt_vaddr, gdt_paddr.as_usize() as u64)
}

fn create_identity_pagetable() -> u64 {
    // Allocate pages for pagetable: PML4 -> PDP -> PD -> PT
    let pml4_vaddr = unsafe {
        alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as *mut u8 as u64
    };

    // Create PDP and PD entries for first entry (identity mapping for low memory)
    unsafe {
        // PML4[0] -> PDP[0]
        let pdp_vaddr = alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as *mut u8 as u64;
        let pdp_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(pdp_vaddr as usize));
        *((pml4_vaddr + 0) as *mut u64) = pdp_paddr.as_usize() as u64 | 3; // Present, R/W

        // PDP[0] -> PD[0]
        let pd_vaddr = alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as *mut u8 as u64;
        let pd_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(pd_vaddr as usize));
        *((pdp_vaddr + 0) as *mut u64) = pd_paddr.as_usize() as u64 | 3; // Present, R/W

        // PD[0] -> PT[0] (use 4KB pages)
        let pt_vaddr = alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as *mut u8 as u64;
        let pt_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(pt_vaddr as usize));
        *((pd_vaddr + 0) as *mut u64) = pt_paddr.as_usize() as u64 | 3; // Present, R/W

        // PT[0-512]: Identity map first 2MB (512 * 4KB)
        for i in 0..512 {
            let paddr = (i as u64) * 4096;
            *((pt_vaddr + (i * 8)) as *mut u64) = paddr | 3; // Present, R/W
        }
    }

    let pml4_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(pml4_vaddr as usize));
    ax_println!("Identity pagetable PML4 at vaddr: {:#x}, paddr: {:#x}", pml4_vaddr, usize::from(pml4_paddr));

    pml4_paddr.as_usize() as u64
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    ax_println!("x86_64 Hypervisor ...");

    // Check VMX support
    if !vmx::check_vmx_support() {
        panic!("VMX not supported!");
    }
    ax_println!("VMX is supported!");

    // Initialize VMX
    vmx::vmx_init();

    // A new address space for VM.
    let mut uspace = axmm::new_user_aspace().unwrap();

    // Load VM binary file into address space.
    // Note: For x86, we use skernel-x86
    let guest_paddr = match loader::load_vm_image("/sbin/skernel-x86", &mut uspace) {
        Ok(paddr) => paddr,
        Err(e) => panic!("Cannot load app! {:?}", e),
    };

    // Setup context to prepare to enter guest mode.
    let mut ctx = VmCpuRegisters::default();

    // Create GDT for guest
    let (gdt_vaddr, gdt_paddr) = create_gdt();
    ax_println!("Guest GDT at vaddr: {:#x}, paddr: {:#x}", gdt_vaddr, gdt_paddr);

    // Create identity pagetable for guest
    let guest_cr3 = create_identity_pagetable();

    // Use VM_ENTRY virtual address as guest RIP (will be translated by guest pagetable)
    prepare_guest_context(&mut ctx, VM_ENTRY as u64, gdt_paddr, guest_cr3);

    // Setup pagetable (EPT in VMX terminology)
    let ept_root = uspace.page_table_root();
    ax_println!("EPT root: {:#x}", ept_root);

    // Setup VMCS and EPT
    if let Err(e) = vmx::setup_vmcs(&mut ctx, ept_root) {
        panic!("Setup VMCS failed: {:?}", e);
    }

    // Kick off VM and wait for it to exit.
    run_guest(&mut ctx);

    ax_println!("VM exited successfully!");

    // Clean up VMX
    vmx::vmx_cleanup();
}

fn prepare_guest_context(ctx: &mut VmCpuRegisters, entry: u64, gdt_paddr: u64, guest_cr3: u64) {
    // Set guest general purpose registers
    ctx.guest_state.rip = entry;
    ctx.guest_state.rsp = 0x10_0000;
    ctx.guest_state.rflags = 0x2; // Interrupt enable flag

    // Set guest control registers
    // CR0: PE (protected mode), NE (numeric error), PG (paging enabled)
    ctx.guest_state.cr0 = 0x80000001; // PE=1, PG=1
    ctx.guest_state.cr3 = guest_cr3; // Guest pagetable for identity mapping
    ctx.guest_state.cr4 = 0x20A0; // PAE (bit 5), VMXE (bit 13)
    ax_println!("Guest CR0: {:#x}, CR3: {:#x}, CR4: {:#x}",
                ctx.guest_state.cr0, ctx.guest_state.cr3, ctx.guest_state.cr4);
    
    // Set guest segment registers (flat model)
    // CS: Code segment in protected mode (32-bit)
    // Access rights: Present=1, DPL=0, S=1, Type=execute/read, Accessed=1, D=1 (32-bit), L=0, G=0
    ctx.guest_state.cs_selector = 0x8; // Index=1 (GDT entry 1), TI=0, RPL=0
    ctx.guest_state.cs_base = 0;
    ctx.guest_state.cs_limit = 0xFFFFFFFF;
    ctx.guest_state.cs_access_rights = 0x00809B; // D=1 (32-bit code), not L=1






    
    ctx.guest_state.ds_selector = 0x10; // Index=2 (GDT entry 2), TI=0, RPL=0
    ctx.guest_state.ds_base = 0;
    ctx.guest_state.ds_limit = 0xFFFFFFFF;
    ctx.guest_state.ds_access_rights = 0x93; // Present, R/W, Accessed, DPL=0

    ctx.guest_state.es_selector = 0x10;
    ctx.guest_state.es_base = 0;
    ctx.guest_state.es_limit = 0xFFFFFFFF;
    ctx.guest_state.es_access_rights = 0x93;

    ctx.guest_state.fs_selector = 0x10;
    ctx.guest_state.fs_base = 0;
    ctx.guest_state.fs_limit = 0xFFFFFFFF;
    ctx.guest_state.fs_access_rights = 0x93;

    ctx.guest_state.gs_selector = 0x10;
    ctx.guest_state.gs_base = 0;
    ctx.guest_state.gs_limit = 0xFFFFFFFF;
    ctx.guest_state.gs_access_rights = 0x93;

    ctx.guest_state.ss_selector = 0x10;
    ctx.guest_state.ss_base = 0;
    ctx.guest_state.ss_limit = 0xFFFFFFFF;
    ctx.guest_state.ss_access_rights = 0x93;

    
    ctx.guest_state.ldtr_selector = 0;
    ctx.guest_state.ldtr_base = 0;
    ctx.guest_state.ldtr_limit = 0xFFFF;
    ctx.guest_state.ldtr_access_rights = 0x82; // Present
    
    ctx.guest_state.tr_selector = 0;
    ctx.guest_state.tr_base = 0;
    ctx.guest_state.tr_limit = 0xFFFF;
    ctx.guest_state.tr_access_rights = 0x8B; // Present, TSS
    
    // Set GDTR and IDTR
    ctx.guest_state.gdtr_base = gdt_paddr; // Use real GDT
    ctx.guest_state.gdtr_limit = 0x17; // 3 entries, 8 bytes each
    ctx.guest_state.idtr_base = 0;
    ctx.guest_state.idtr_limit = 0xFFFF;
    
    // Activity state
    ctx.guest_state.activity_state = 0; // Active
    ctx.guest_state.interruptibility_state = 0;
    
    ax_println!("Guest context prepared");
}

fn run_guest(ctx: &mut VmCpuRegisters) {
    ax_println!("Entering guest mode...");
    vmx::vmx_launch(ctx);
}
