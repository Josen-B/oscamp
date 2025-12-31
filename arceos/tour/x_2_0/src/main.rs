#![no_std]
#![no_main]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;
extern crate axstd as std;

use alloc::string::String;
use alloc::string::ToString;
use axerrno::{ax_err_type, AxResult};
use axhal::mem::VirtAddr;
use std::fs::File;
use std::io::{BufReader, Read};
use axmm::AddrSpace;
use axhal::paging::MappingFlags;
use axvcpu::{AxVCpu, AxVCpuExitReason};
use axvma::{AxVM, AxVMConfig, AxVCpuConfig};
use axvisor_api::vmm::VCpuId;

// x86_64 virtualization constants
const VM_ASPACE_BASE: usize = 0x0;
const VM_ASPACE_SIZE: usize = 0x1_0000_0000; // 4GB address space
const PHY_MEM_START: usize = 0x0;
const PHY_MEM_SIZE: usize = 0x100_0000; // 16MB physical memory
const KERNEL_BASE: usize = 0x10_0000; // Guest entry point

/// Load VM image from disk into guest memory
fn load_vm_image(image_path: String, image_load_gpa: VirtAddr, aspace: &AddrSpace) -> AxResult {
    let (image_file, image_size) = open_image_file(image_path.as_str())?;
    let mut file = BufReader::new(image_file);

    let image_load_regions = aspace
        .translated_byte_buffer(image_load_gpa, image_size)
        .expect("Failed to translate kernel image load address");

    for buffer in image_load_regions {
        file.read_exact(buffer).map_err(|err| {
            ax_err_type!(
                Io,
                format!("Failed in reading from file {}, err {:?}", image_path, err)
            )
        })?
    }

    Ok(())
}

/// Open image file and return file handle and size
fn open_image_file(file_name: &str) -> AxResult<(File, usize)> {
    let file = File::open(file_name).map_err(|err| {
        ax_err_type!(
            NotFound,
            format!(
                "Failed to open {}, err {:?}, please check your disk.img",
                file_name, err
            )
        )
    })?;
    let file_size = file
        .metadata()
        .map_err(|err| {
            ax_err_type!(
                Io,
                format!(
                    "Failed to get metadata of file {}, err {:?}",
                    file_name, err
                )
            )
        })?
        .size() as usize;
    Ok((file, file_size))
}

/// Main entry point
#[no_mangle]
fn main() {
    info!("Starting x86_64 virtualization...");

    // Check if VMX is supported
    if !x86_vcpu::has_hardware_support() {
        panic!("VMX not supported on this CPU!");
    }
    info!("VMX is supported!");

    // Setup AddressSpace and regions
    let mut aspace = AddrSpace::new_empty(VirtAddr::from(VM_ASPACE_BASE), VM_ASPACE_SIZE).unwrap();

    // Physical memory region - full access flags
    let mapping_flags = MappingFlags::from_bits(0xf).unwrap(); // READ | WRITE | EXECUTE | USER
    aspace.map_alloc(PHY_MEM_START.into(), PHY_MEM_SIZE, mapping_flags, true).unwrap();

    // Load corresponding images for VM
    info!("VM created successfully, loading images...");
    let image_fname = "/sbin/skernel-x86";
    load_vm_image(image_fname.to_string(), KERNEL_BASE.into(), &aspace).expect("Failed to load VM images");

    // Create VCPU - using VM ID 0 and VCPU ID 0
    let mut arch_vcpu = VmxArchVCpu::<SimpleVCpuHal>::new(0, 0).unwrap();

    // Setup VCPU with entry point and EPT root
    info!("bsp_entry: {:#x}; ept: {:#x}", KERNEL_BASE, aspace.page_table_root());
    arch_vcpu.setup(KERNEL_BASE.into(), aspace.page_table_root().as_usize().into()).unwrap();

    // Bind VCPU to current CPU
    arch_vcpu.bind().unwrap();

    // Main VCPU run loop
    loop {
        match vcpu_run(&mut arch_vcpu) {
            Ok(exit_reason) => match exit_reason {
                AxVCpuExitReason::Nothing => {
                    // Continue running
                }
                AxVCpuExitReason::NestedPageFault {
                    addr,
                    access_flags,
                } => {
                    debug!("Nested page fault at addr {:#x}, access flags {:#x?}", addr, access_flags);

                    // For this exercise, we only handle a specific region
                    // This is similar to RISC-V implementation handling pflash#2
                    assert_eq!(addr.as_usize(), 0x2200_0000, "Now we ONLY handle region at 0x2200_0000");

                    let mapping_flags = MappingFlags::from_bits(0xf).unwrap();

                    // Passthrough-Mode: Direct mapping of guest physical to host physical
                    let host_virt_addr = VirtAddr::from(addr.as_usize());
                    let host_phys_addr = axhal::mem::phys_to_virt(axhal::mem::virt_to_phys(host_virt_addr));
                    let _ = aspace.map_linear(host_virt_addr, host_phys_addr, 4096, mapping_flags);

                    /*
                    // Emulator-Mode: Simulate loading data
                    let buf = b"pfld";
                    aspace.map_alloc(host_addr, 4096, mapping_flags, true);
                    aspace.write(host_addr, buf);
                    */
                }
                AxVCpuExitReason::Halt | AxVCpuExitReason::SystemDown => {
                    info!("Guest halted, stopping VM");
                    break;
                }
                AxVCpuExitReason::IoWrite {
                    port,
                    width,
                    data,
                } => {
                    info!("IO write: port={:#x}, width={:?}, data={:#x}", port.0, width, data);
                    // Handle I/O writes
                }
                AxVCpuExitReason::IoRead {
                    port,
                    width,
                } => {
                    info!("IO read: port={:#x}, width={:?}", port.0, width);
                    // Handle I/O reads
                }
                AxVCpuExitReason::Hypercall { nr, args } => {
                    info!("Hypercall: nr={:#x}, args={:?}", nr, args);
                }
                _ => {
                    info!("VM exit: {:?}", exit_reason);
                    // Handle other exit reasons
                }
            },
            Err(err) => {
                panic!("run VCPU got error: {:?}", err);
            }
        }
    }

    info!("VM exited successfully!");
}

/// Run the VCPU and handle VM exits
fn vcpu_run(arch_vcpu: &mut VmxArchVCpu<SimpleVCpuHal>) -> AxResult<AxVCpuExitReason> {
    use axhal::arch::{disable_irqs, enable_irqs};

    // Disable interrupts while running VM
    let flags = unsafe { disable_irqs() };

    // Run VCPU
    let ret = arch_vcpu.run();

    // Restore interrupts
    unsafe { enable_irqs() };

    ret
}
