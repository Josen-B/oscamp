#![no_std]
#![no_main]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;
extern crate axstd as std;
use alloc::string::ToString;
use x86_vcpu::{VmxExitReason, VmxArchVCpu};
use axerrno::{ax_err_type, AxResult};
use axhal::mem::VirtAddr;
use alloc::string::String;
use std::fs::File;
use axvcpu::AxVCpuExitReason;
use axaddrspace::NestedPageFaultInfo;

const VM_ASPACE_BASE: usize = 0x0;
const VM_ASPACE_SIZE: usize = 0x1_0000_0000; // 4GB for x86_64
const PHY_MEM_START: usize = 0x0;
const PHY_MEM_SIZE: usize = 0x100_0000; // 16MB physical memory
const KERNEL_BASE: usize = 0x10_0000;

use axmm::AddrSpace;
use axhal::paging::MappingFlags;

#[no_mangle]
fn main() {
    info!("Starting x86_64 virtualization...");

    // Check VMX support
    if !x86_vcpu::has_hardware_support() {
        panic!("VMX not supported on this platform");
    }
    info!("VMX is supported");

    // Setup AddressSpace and regions.
    let mut aspace = AddrSpace::new_empty(VirtAddr::from(VM_ASPACE_BASE), VM_ASPACE_SIZE).unwrap();

    // Physical memory region. Full access flags.
    let mapping_flags = MappingFlags::from_bits(0xf).unwrap();
    aspace.map_alloc(PHY_MEM_START.into(), PHY_MEM_SIZE, mapping_flags, true).unwrap();

    // Load corresponding images for VM.
    info!("VM created success, loading images...");
    let image_fname = "/sbin/u_6_0_x86_64-qemu-q35.bin";
    load_vm_image(image_fname.to_string(), KERNEL_BASE.into(), &aspace).expect("Failed to load VM images");

    // Create VCpus.
    let mut arch_vcpu = VmxArchVCpu::new(0, 0, ()).expect("Failed to create VCPU");

    // Setup VCpus.
    info!("bsp_entry: {:#x}; ept: {:#x}", KERNEL_BASE, aspace.page_table_root());
    arch_vcpu.set_entry(KERNEL_BASE.into()).expect("Failed to set entry");
    arch_vcpu.set_ept_root(aspace.page_table_root().as_usize().into()).expect("Failed to set EPT root");
    arch_vcpu.setup(()).expect("Failed to setup VCPU");

    loop {
        match vcpu_run(&mut arch_vcpu) {
            Ok(exit_reason) => match exit_reason {
                AxVCpuExitReason::Nothing => {},
                _ => {
                    // Check if this is an EPT violation (nested page fault)
                    if let Some(npf_info) = arch_vcpu.nested_page_fault_info().ok() {
                        let addr = npf_info.guest_addr;
                        let access_flags = npf_info.access_flags;

                        debug!("addr {:#x} access {:#x}", addr, access_flags);
                        assert_eq!(addr.as_usize(), 0xe000_0000, "Now we ONLY handle flash at 0xE0000000.");

                        let mapping_flags = MappingFlags::from_bits(0xf).unwrap();
                        // Passthrough-Mode
                        let _ = aspace.map_linear(addr, addr.as_usize().into(), 4096, mapping_flags);

                        /*
                        // Emulator-Mode
                        // Pretend to load file to fill buffer.
                        let buf = "pfld";
                        aspace.map_alloc(addr, 4096, mapping_flags, true);
                        aspace.write(addr, buf.as_bytes());
                        */
                    } else {
                        warn!("Unhandled VM-Exit: {:?}", exit_reason);
                    }
                }
            },
            Err(err) => {
                panic!("run VCpu get error {:?}", err);
            }
        }
    }
}

fn load_vm_image(image_path: String, image_load_gpa: VirtAddr, aspace: &AddrSpace) -> AxResult {
    use std::io::{BufReader, Read};
    let (image_file, image_size) = open_image_file(image_path.as_str())?;

    let image_load_regions = aspace
        .translated_byte_buffer(image_load_gpa, image_size)
        .expect("Failed to translate kernel image load address");
    let mut file = BufReader::new(image_file);

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

fn vcpu_run(arch_vcpu: &mut VmxArchVCpu) -> AxResult<AxVCpuExitReason> {
    use axhal::arch::{local_irq_save_and_disable, local_irq_restore};
    let flags = local_irq_save_and_disable();
    let ret = arch_vcpu.run();
    local_irq_restore(flags);
    ret
}

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
                    "Failed to get metadate of file {}, err {:?}",
                    file_name, err
                )
            )
        })?
        .size() as usize;
    Ok((file, file_size))
}
