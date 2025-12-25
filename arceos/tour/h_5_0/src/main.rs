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
    if let Err(e) = loader::load_vm_image("/sbin/skernel-x86", &mut uspace) {
        panic!("Cannot load app! {:?}", e);
    }

    // Setup context to prepare to enter guest mode.
    let mut ctx = VmCpuRegisters::default();
    prepare_guest_context(&mut ctx, VM_ENTRY);

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

fn prepare_guest_context(ctx: &mut VmCpuRegisters, entry: u64) {
    // Set guest general purpose registers
    ctx.guest_state.rip = entry;
    ctx.guest_state.rsp = 0x10_0000;
    ctx.guest_state.rflags = 0x2; // Interrupt enable flag
    
    // Set guest control registers
    ctx.guest_state.cr0 = 0x60000010; // PE, ET, NE
    ctx.guest_state.cr3 = 0;
    ctx.guest_state.cr4 = 0x2000; // VMXE bit
    
    // Set guest segment registers (flat model)
    ctx.guest_state.cs_selector = 0;
    ctx.guest_state.cs_base = 0;
    ctx.guest_state.cs_limit = 0xFFFFFFFF;
    ctx.guest_state.cs_access_rights = 0x9B; // Present, R/W, Accessed, DPL=0
    
    ctx.guest_state.ds_selector = 0;
    ctx.guest_state.ds_base = 0;
    ctx.guest_state.ds_limit = 0xFFFFFFFF;
    ctx.guest_state.ds_access_rights = 0x93; // Present, R/W, Accessed, DPL=0
    
    ctx.guest_state.es_selector = 0;
    ctx.guest_state.es_base = 0;
    ctx.guest_state.es_limit = 0xFFFFFFFF;
    ctx.guest_state.es_access_rights = 0x93;
    
    ctx.guest_state.fs_selector = 0;
    ctx.guest_state.fs_base = 0;
    ctx.guest_state.fs_limit = 0xFFFFFFFF;
    ctx.guest_state.fs_access_rights = 0x93;
    
    ctx.guest_state.gs_selector = 0;
    ctx.guest_state.gs_base = 0;
    ctx.guest_state.gs_limit = 0xFFFFFFFF;
    ctx.guest_state.gs_access_rights = 0x93;
    
    ctx.guest_state.ss_selector = 0;
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
    ctx.guest_state.gdtr_base = 0;
    ctx.guest_state.gdtr_limit = 0xFFFF;
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
