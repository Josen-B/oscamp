use std::io::{self, Read};
use std::fs::File;
use axhal::paging::MappingFlags;
use axhal::mem::{PAGE_SIZE_4K, phys_to_virt, VirtAddr, PhysAddr};
use axmm::AddrSpace;
use crate::VM_ENTRY;

pub fn load_vm_image(fname: &str, uspace: &mut AddrSpace) -> io::Result<PhysAddr> {
    // Read the entire binary (max 1 page for simplicity)
    let mut buf = [0u8; PAGE_SIZE_4K];
    let n = load_file(fname, &mut buf)?;
    
    ax_println!("Read {} bytes from {}", n, fname);

    // Map the guest code to the VM_ENTRY address
    // The physical address will be determined by the page allocator
    uspace.map_alloc(
        VirtAddr::from(VM_ENTRY as usize),
        PAGE_SIZE_4K,
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE | MappingFlags::USER,
        true
    ).unwrap();

    let (paddr, _, _) = uspace
        .page_table()
        .query(VirtAddr::from(VM_ENTRY as usize))
        .unwrap_or_else(|_| panic!("Mapping failed for segment: {:#x}", VM_ENTRY));

    ax_println!("Guest image loaded at paddr: {:#x}", paddr);

    unsafe {
        core::ptr::copy_nonoverlapping(
            buf.as_ptr(),
            phys_to_virt(paddr).as_mut_ptr(),
            n,
        );
    }
    
    Ok(paddr)
}

fn load_file(fname: &str, buf: &mut [u8]) -> io::Result<usize> {
    ax_println!("Loading app: {}", fname);
    let mut file = File::open(fname)?;
    let n = file.read(buf)?;
    Ok(n)
}
