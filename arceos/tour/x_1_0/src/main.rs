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

    // Entry 1: 32-bit code segment (selector = 0x8)
    // Base=0, Limit=0xFFFFFFFF, Present=1, DPL=0, System=1, Type=Execute/Read, Accessed=1, L=0, D=1
    // Flags = 0xC (G=1, L=0, D=1) - 32-bit code segment
    // GDT entry format: [base 31:24][access][limit 19:16][base 23:16][base 15:0][limit 15:0]
    // For 0x00CF9B000000FFFF:
    //   Low dword: limit=0xFFFF, base[15:0]=0
    //   High dword: base[31:24]=0, access=0x9B, limit[19:16]=0xF, base[23:16]=0, flags=0xC
    unsafe {
        *((gdt_vaddr + 8) as *mut u64) = 0x00CF9B000000FFFF; // 32-bit code segment
    }

    // Entry 2: Data segment (selector = 0x10)
    // Base=0, Limit=0xFFFFFFFF, Present=1, DPL=0, System=1, Type=Read/Write, Accessed=1
    unsafe {
        *((gdt_vaddr + 16) as *mut u64) = 0x00CF93000000FFFF;
    }

    let gdt_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(gdt_vaddr as usize));

    // GDT has 3 entries (24 bytes): null, code, data
    let gdt_limit = 23u16; // 24 - 1
    ax_println!("GDT created: {:#x} entries, limit={:#x}",
                 gdt_paddr.as_usize() as u64, gdt_limit);

    (gdt_vaddr, gdt_paddr.as_usize() as u64)
}

fn create_idt() -> (u64, u64) {
    // Allocate a page for IDT
    let idt_vaddr = unsafe {
        alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as *mut u8 as u64
    };

    // IDT entries are 16 bytes each in 64-bit mode
    // For simplicity, we'll create a minimal IDT with null entries
    // Each entry: [offset 63:32][reserved][type][ist][selector][offset 31:16][reserved][offset 15:0]
    // We'll set all entries to 0, which will cause faults if triggered

    let idt_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(idt_vaddr as usize));

    // IDT can have up to 256 entries, but we'll limit it
    let idt_limit = 256u16 * 16 - 1; // 256 entries, 16 bytes each

    ax_println!("IDT created: {:#x} entries, limit={:#x}",
                 idt_paddr.as_usize() as u64, idt_limit);

    (idt_vaddr, idt_paddr.as_usize() as u64)
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

        // PD[0] -> PT[0] (use 4KB pages for first 2MB)
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

        // PD[1] -> PT[1] (use 4KB pages for 2-4MB)
        let pt_vaddr1 = alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as *mut u8 as u64;
        let pt_paddr1 = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(pt_vaddr1 as usize));
        *((pd_vaddr + 8) as *mut u64) = pt_paddr1.as_usize() as u64 | 3; // Present, R/W

        // PT[1]: Identity map 2-4MB
        for i in 0..512 {
            let paddr = ((i as u64) + 512) * 4096;
            *((pt_vaddr1 + (i * 8)) as *mut u64) = paddr | 3; // Present, R/W
        }

        // PD[2] -> PT[2] (use 4KB pages for 4-6MB)
        let pt_vaddr2 = alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as *mut u8 as u64;
        let pt_paddr2 = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(pt_vaddr2 as usize));
        *((pd_vaddr + 16) as *mut u64) = pt_paddr2.as_usize() as u64 | 3; // Present, R/W

        // PT[2]: Identity map 4-6MB (covers 0x455000)
        for i in 0..512 {
            let paddr = ((i as u64) + 1024) * 4096;
            *((pt_vaddr2 + (i * 8)) as *mut u64) = paddr | 3; // Present, R/W
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
    let guest_paddr = loader::load_vm_image("/sbin/skernel-x86", &mut uspace);
    // Use VM_ENTRY as guest virtual address (fixed for simplicity)
    let guest_vaddr = VM_ENTRY;

    // Setup context to prepare to enter guest mode.
    let mut ctx = VmCpuRegisters::default();

    // Create GDT for guest
    let (gdt_vaddr, gdt_paddr) = create_gdt();
    ax_println!("Guest GDT at vaddr: {:#x}, paddr: {:#x}", gdt_vaddr, gdt_paddr);

    // Create IDT for guest
    let (_idt_vaddr, idt_paddr) = create_idt();

    // Create identity pagetable for guest
    // Guest will use paging (CR0.PG=1) instead of EPT
    let guest_cr3 = create_identity_pagetable();

    // Use guest_vaddr (virtual address) as guest RIP
    // EPT will translate virtual address 0x100000 to physical address 0x45b000
    prepare_guest_context(&mut ctx, guest_vaddr, gdt_vaddr, gdt_paddr, idt_paddr, guest_cr3);

    // Setup pagetable (EPT in VMX terminology)
    let ept_root = uspace.page_table_root();
    ax_println!("EPT root: {:#x}", ept_root);

    // Check if guest code is mapped in EPT
    let (guest_code_paddr, _, _) = uspace
        .page_table()
        .query(axhal::mem::VirtAddr::from(VM_ENTRY as usize))
        .unwrap_or_else(|_| panic!("Guest code not mapped in EPT: {:#x}", VM_ENTRY));
    ax_println!("Guest code at VA {:#x} -> PA {:#x} in EPT", VM_ENTRY, guest_code_paddr);

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

fn prepare_guest_context(ctx: &mut VmCpuRegisters, entry: u64, gdt_vaddr: u64, gdt_paddr: u64, idt_paddr: u64, guest_cr3: u64) {
    // Set guest general purpose registers
    ctx.guest_state.rip = entry;
    ctx.guest_state.rsp = 0x10_0000;
    ctx.guest_state.rflags = 0x202; // Bit 1=1 (must be 1), Bit 9=1 (interrupt enable)

    // Set guest control registers
    // CR0: protected mode WITH paging (required for 64-bit long mode)
    ctx.guest_state.cr0 = 0x80010001; // PE=1, PG=1, NE=1, WP=0
    ctx.guest_state.cr3 = guest_cr3; // Guest CR3 for paging
    ctx.guest_state.cr4 = 0x00002020; // VMXE=1, PAE=1 (required for 64-bit)
    ax_println!("Guest CR0: {:#x}, CR3: {:#x}, CR4: {:#x} (64-bit long mode with guest paging)",
                ctx.guest_state.cr0, ctx.guest_state.cr3, ctx.guest_state.cr4);

    // Set guest segment registers for 64-bit long mode
    ctx.guest_state.cs_selector = 0x8; // GDT code segment selector
    ctx.guest_state.cs_base = 0;
    ctx.guest_state.cs_limit = 0xFFFFFFFF; // Large limit for flat model
    // 64-bit long mode code segment
    // AR = 0x909B (binary: 1001 0000 1001 1011)
    //   P=1, DPL=0, S=1 (code/data segment), Type=0xB (Execute/Read code, Accessed=1)
    //   L=1 (64-bit mode), D=0 (ignored in 64-bit mode), G=1 (4KB granularity)
    ctx.guest_state.cs_access_rights = 0x909B;
    ax_println!("CS: selector={:#x}, base={:#x}, limit={:#x}, AR={:#x} (64-bit long mode, L=1, D=0, G=1)",
        ctx.guest_state.cs_selector, ctx.guest_state.cs_base, ctx.guest_state.cs_limit, ctx.guest_state.cs_access_rights);







    // Data segments for 32-bit compatibility mode
    ctx.guest_state.ds_selector = 0x10; // GDT data segment selector
    ctx.guest_state.ds_base = 0;
    ctx.guest_state.ds_limit = 0xFFFFFFFF; // Large limit
    ctx.guest_state.ds_access_rights = 0xC013; // Present, R/W, G=1 (bits 4-2=000)

    ctx.guest_state.es_selector = 0x10;
    ctx.guest_state.es_base = 0;
    ctx.guest_state.es_limit = 0xFFFFFFFF;
    ctx.guest_state.es_access_rights = 0xC013; // Present, R/W, G=1 (bits 4-2=000)

    ctx.guest_state.fs_selector = 0;
    ctx.guest_state.fs_base = 0;
    ctx.guest_state.fs_limit = 0;
    ctx.guest_state.fs_access_rights = 0x10000; // Unusable (bit 16 = 1)

    ctx.guest_state.gs_selector = 0;
    ctx.guest_state.gs_base = 0;
    ctx.guest_state.gs_limit = 0;
    ctx.guest_state.gs_access_rights = 0x10000; // Unusable (bit 16 = 1)

    // Stack segment
    ctx.guest_state.ss_selector = 0x10;
    ctx.guest_state.ss_base = 0;
    ctx.guest_state.ss_limit = 0xFFFFFFFF;
    ctx.guest_state.ss_access_rights = 0xC013; // Present, R/W, G=1 (bits 4-2=000 for valid)


    // Set LDTR to null (unusable)
    // AR = 0x10000 (Unusable, bit 16 = 1)
    ctx.guest_state.ldtr_selector = 0;
    ctx.guest_state.ldtr_base = 0;
    ctx.guest_state.ldtr_limit = 0;
    ctx.guest_state.ldtr_access_rights = 0x10000; // Unusable (bit 16 = 1)

    // Allocate TSS area for guest (minimum 104 bytes for 32-bit TSS)
    // TSS needs to be in guest memory (EPT mapped)
    let tss_vaddr = unsafe {
        alloc::alloc::alloc_zeroed(
            alloc::alloc::Layout::from_size_align(4096, 4096).unwrap()
        ) as u64
    };
    let tss_paddr = axhal::mem::virt_to_phys(axhal::mem::VirtAddr::from(tss_vaddr as usize));
    ax_println!("Guest TSS at vaddr: {:#x}, paddr: {:#x}", tss_vaddr, tss_paddr.as_usize() as u64);    // Set up a minimal TSS descriptor in GDT (Entry 3, selector 0x18)
    // TSS descriptor for 32-bit task state segment (available, not busy)
    // Type=0x1 (Available 32-bit TSS), P=1, DPL=0, G=0
    let tss_limit = 0x67; // Minimum TSS size for 32-bit (104 bytes - 1)
    // TSS descriptor format (32-bit):
    // Low 32 bits: [limit 15:0][base 15:0]
    // High 32 bits: [base 31:24][G][AVL][0][limit 19:16][P][DPL][S][Type][base 23:16]
    // Use actual TSS physical address
    let tss_base = tss_paddr.as_usize() as u32;
    let tss_desc_low = ((tss_limit as u32 & 0xFFFF) << 16) | (tss_base & 0xFFFF);
    let tss_desc_high = ((tss_base >> 24) & 0xFF) << 24  // base[31:24]
        | 0 << 20  // G=0
        | 0 << 19  // AVL=0
        | ((tss_limit >> 16) & 0xF) << 16  // limit[19:16]
        | 1 << 15  // P=1
        | 0 << 13  // DPL=0
        | 0 << 12  // S=0
        | 0x1 << 8  // Type=0x1 (Available 32-bit TSS)
        | ((tss_base >> 16) & 0xFF);  // base[23:16]
    ax_println!("TSS desc_low = {:#010x}", tss_desc_low);
    ax_println!("TSS desc_high = {:#010x}", tss_desc_high);
    let tss_desc_low64 = tss_desc_low as u64;
    let tss_desc_high64 = (tss_desc_high as u64) << 32;
    let tss_desc_full = tss_desc_low64 | tss_desc_high64;
    ax_println!("TSS desc_low64 = {:#018x}", tss_desc_low64);
    ax_println!("TSS desc_high64 = {:#018x}", tss_desc_high64);
    ax_println!("TSS desc_full = {:#018x}", tss_desc_full);
    unsafe {
        *((gdt_vaddr + 0x18) as *mut u64) = tss_desc_full;
    }
    ax_println!("TSS descriptor written to GDT at offset 0x18: {:#016x}", tss_desc_full);

    // Verify TSS descriptor was written correctly
    let tss_desc_check = unsafe { *((gdt_vaddr + 0x18) as *const u64) };
    ax_println!("TSS descriptor read back: {:#016x}", tss_desc_check);

    // Dump GDT entries for debugging
    ax_println!("GDT entries:");
    for i in 0..8 {
        let offset = i * 8;
        let desc = unsafe { *((gdt_vaddr + offset) as *const u64) };
        ax_println!("  GDT[{}] at offset {:#04x}: {:#018x}", i, offset, desc);
    }

    // Set TR to point to TSS (selector 0x18, RPL=0)
    // Note: With unrestricted guest enabled, TR can be null
    // But for simplicity, we'll keep a valid TSS
    ctx.guest_state.tr_selector = 0x18;
    ctx.guest_state.tr_base = tss_paddr.as_usize() as u64; // Use actual TSS base
    ctx.guest_state.tr_limit = tss_limit as u64; // TSS limit
    ctx.guest_state.tr_access_rights = 0x81; // Available 32-bit TSS (P=1, DPL=0, S=0, Type=0x1)
    ax_println!("TR set: selector={:#x}, base={:#x}, limit={:#x}, AR={:#x}",
        ctx.guest_state.tr_selector, ctx.guest_state.tr_base, ctx.guest_state.tr_limit, ctx.guest_state.tr_access_rights);
    ax_println!("  Note: Unrestricted guest enabled, TSS is optional but configured for compatibility");
    
    
    // Set GDTR and IDTR
    ctx.guest_state.gdtr_base = gdt_paddr; // Use real GDT
    ctx.guest_state.gdtr_limit = 0x1f; // 4 entries (null, code, data, TSS), 32 bytes - 1
    ctx.guest_state.idtr_base = idt_paddr; // Use real IDT
    ctx.guest_state.idtr_limit = 0xFFF; // 256 entries * 16 bytes - 1
    ax_println!("GDTR set: base={:#x}, limit={:#x}",
        ctx.guest_state.gdtr_base, ctx.guest_state.gdtr_limit);
    ax_println!("IDTR set: base={:#x}, limit={:#x}",
        ctx.guest_state.idtr_base, ctx.guest_state.idtr_limit);

    // Activity state
    ctx.guest_state.activity_state = 0; // Active
    ctx.guest_state.interruptibility_state = 0;
    
    ax_println!("Guest context prepared");
}

fn run_guest(ctx: &mut VmCpuRegisters) {
    ax_println!("Entering guest mode...");
    vmx::vmx_launch(ctx);
}
