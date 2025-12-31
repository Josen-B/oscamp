#![no_std]
#![no_main]

mod vmdev;

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;
extern crate axstd as std;
use alloc::string::ToString;
use x86_vcpu::{VmxArchVCpu, VmxExitReason, has_hardware_support};
use axvcpu::{arch_vcpu::AxArchVCpu, AxVCpuExitReason};
use axerrno::{ax_err_type, AxResult};
use axhal::mem::VirtAddr;
use alloc::string::String;
use std::fs::File;
use axmm::AddrSpace;
use axhal::paging::MappingFlags;
use vmdev::VmDevGroup;

const VM_ASPACE_BASE: usize = 0x0;
const VM_ASPACE_SIZE: usize = 0x7fff_ffff_f000;
const PHY_MEM_START: usize = 0x100_0000;
const PHY_MEM_SIZE: usize = 0x100_0000;
const KERNEL_BASE: usize = 0x1_0000;

#[no_mangle]
fn main() {
    info!("Starting x86_64 virtualization...");

    // Check hardware support
    if !x86_vcpu::has_hardware_support() {
        panic!("This platform does not support VT-x virtualization");
    }
    info!("VT-x is supported");

    // Setup AddressSpace and regions.
    let mut aspace = AddrSpace::new_empty(VirtAddr::from(VM_ASPACE_BASE), VM_ASPACE_SIZE).unwrap();

    // Physical memory region. Full access flags.
    let mapping_flags = MappingFlags::from_bits(0xf).unwrap();
    aspace.map_alloc(PHY_MEM_START.into(), PHY_MEM_SIZE, mapping_flags, true).unwrap();

    // Load corresponding images for VM.
    info!("VM created success, loading images...");
    let image_fname = "/sbin/m_1_1_x86_64-qemu-q35.bin";
    load_vm_image(image_fname.to_string(), KERNEL_BASE.into(), &aspace).expect("Failed to load VM images");

    // Register pflash device into vm.
    let mut vmdevs = VmDevGroup::new();
    vmdevs.add_dev(0xc000_0000.into(), 0x200_0000);

    // Create VCpus.
    let vm_id = 0;
    let vcpu_id = 0;
    let mut arch_vcpu = VmxArchVCpu::new(vm_id, vcpu_id).expect("Failed to create VCPU");

    // Setup VCpus.
    info!("bsp_entry: {:#x}; ept: {:#x}", KERNEL_BASE, aspace.page_table_root());
    arch_vcpu.set_entry(axaddrspace::addr::GuestPhysAddr::from(KERNEL_BASE)).unwrap();
    arch_vcpu.set_ept_root(axaddrspace::addr::HostPhysAddr::from(aspace.page_table_root())).unwrap();

    // Setup VCPU and bind to current processor
    arch_vcpu.setup(
        axaddrspace::addr::HostPhysAddr::from(aspace.page_table_root()),
        axaddrspace::addr::GuestPhysAddr::from(KERNEL_BASE)
    ).expect("Failed to setup VCPU");
    arch_vcpu.bind().expect("Failed to bind VCPU");

    loop {
        match vcpu_run(&mut arch_vcpu) {
            Ok(exit_reason) => match exit_reason {
                x86_vcpu::AxVCpuExitReason::Nothing => {},
                x86_vcpu::AxVCpuExitReason::Halt => {
                    info!("VM halted");
                    break;
                },
                x86_vcpu::AxVCpuExitReason::SystemDown => {
                    info!("VM system down");
                    break;
                },
                x86_vcpu::AxVCpuExitReason::MmioRead { addr, width, reg, reg_width } => {
                    debug!("MMIO read: addr {:#x}, width {:?}", addr, width);
                    if addr.as_usize() < PHY_MEM_START {
                        // Find dev and handle mmio region.
                        let dev = vmdevs.find_dev(addr).expect("No dev.");
                        dev.handle_mmio_read(addr, width, reg, reg_width, &mut arch_vcpu, &aspace).unwrap();
                    } else {
                        unimplemented!("Handle MMIO read for memory region.");
                    }
                },
                x86_vcpu::AxVCpuExitReason::MmioWrite { addr, width, data } => {
                    debug!("MMIO write: addr {:#x}, width {:?}, data {:#x}", addr, width, data);
                    if addr.as_usize() < PHY_MEM_START {
                        // Find dev and handle mmio region.
                        let dev = vmdevs.find_dev(addr).expect("No dev.");
                        dev.handle_mmio_write(addr, width, data, &aspace).unwrap();
                    } else {
                        unimplemented!("Handle MMIO write for memory region.");
                    }
                },
                _ => {
                    panic!("Unhandled VM-Exit: {:?}", exit_reason);
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

fn vcpu_run(arch_vcpu: &mut VmxArchVCpu) -> AxResult<x86_vcpu::AxVCpuExitReason> {
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
