use std::io::Read;
use alloc::vec::Vec;
use axhal::paging::MappingFlags;
use axhal::mem::{PAGE_SIZE_4K, phys_to_virt, VirtAddr, PhysAddr};
use axmm::AddrSpace;

pub fn load_vm_image(fname: &str, uspace: &mut AddrSpace) -> PhysAddr {
    ax_println!("Loading app: {}", fname);

    // Read entire file
    let mut file = std::fs::File::open(fname).expect("Cannot open file");
    let file_len = file.metadata().expect("Cannot get metadata").len() as usize;
    let mut elf_data = Vec::new();
    elf_data.resize(file_len, 0u8);
    file.read_exact(&mut elf_data).expect("Cannot read file");

    ax_println!("ELF file size: {} bytes", file_len);

    let elf_data = elf_data.as_slice();

    // Map to 0x100000 (VM_ENTRY address)
    let vaddr = 0x100000usize;
    let num_pages = (file_len + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K;

    ax_println!("Mapping {:#x} ({} pages)", vaddr, num_pages);

    // Map pages
    for page_idx in 0..num_pages {
        let page_vaddr = VirtAddr::from(vaddr + page_idx * PAGE_SIZE_4K);
        uspace.map_alloc(
            page_vaddr,
            PAGE_SIZE_4K,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE | MappingFlags::USER,
            true
        ).unwrap();
    }

    // Get physical address of mapping
    let (paddr, _, _) = uspace
        .page_table()
        .query(VirtAddr::from(vaddr))
        .unwrap_or_else(|_| panic!("Mapping failed for segment: {:#x}", vaddr));

    // Copy entire file
    unsafe {
        let dst = phys_to_virt(paddr).as_mut_ptr();
        std::ptr::copy_nonoverlapping(elf_data.as_ptr(), dst, file_len);
    }

    ax_println!("Copied {} bytes to {:#x}", file_len, paddr);

    // Return the physical address where code is loaded
    // This should match the identity mapping
    PhysAddr::from(paddr.as_usize())
}
