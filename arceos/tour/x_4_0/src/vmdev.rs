use alloc::vec::Vec;
use alloc::sync::Arc;
use axerrno::AxResult;
use memory_addr::VirtAddr;
use axhal::paging::MappingFlags;
use axmm::AddrSpace;
use x86_vcpu::AccessWidth;
use x86_vcpu::VmxArchVCpu;
use axvcpu::AxVCpuExitReason;

pub struct VmDev {
    start: VirtAddr,
    size: usize,
}

impl VmDev {
    pub fn new(start: VirtAddr, size: usize) -> Self {
        Self { start, size }
    }

    pub fn handle_mmio(&self, addr: VirtAddr , aspace: &mut AddrSpace) -> AxResult {
        let mapping_flags = MappingFlags::from_bits(0xf).unwrap();
        // Passthrough-Mode
        aspace.map_linear(addr, addr.as_usize().into(), 4096, mapping_flags)
    }

    pub fn handle_mmio_read(
        &self,
        addr: VirtAddr,
        width: AccessWidth,
        reg: usize,
        reg_width: AccessWidth,
        vcpu: &mut VmxArchVCpu,
        aspace: &mut AddrSpace,
    ) -> AxResult {
        // Map the region if not already mapped
        self.handle_mmio(addr, aspace)?;

        // Perform the read operation
        let val = match width {
            AccessWidth::Byte => {
                let ptr = addr.as_usize() as *const u8;
                unsafe { ptr.read_volatile() as u64 }
            }
            AccessWidth::Word => {
                let ptr = addr.as_usize() as *const u16;
                unsafe { ptr.read_volatile() as u64 }
            }
            AccessWidth::Dword => {
                let ptr = addr.as_usize() as *const u32;
                unsafe { ptr.read_volatile() as u64 }
            }
            AccessWidth::Qword => {
                let ptr = addr.as_usize() as *const u64;
                unsafe { ptr.read_volatile() }
            }
        };

        // Set the return value in the register
        vcpu.set_gpr(reg, val as usize);
        Ok(())
    }

    pub fn handle_mmio_write(
        &self,
        addr: VirtAddr,
        width: AccessWidth,
        data: u64,
        aspace: &mut AddrSpace,
    ) -> AxResult {
        // Map the region if not already mapped
        self.handle_mmio(addr, aspace)?;

        // Perform the write operation
        match width {
            AccessWidth::Byte => {
                let ptr = addr.as_usize() as *mut u8;
                unsafe { ptr.write_volatile(data as u8) }
            }
            AccessWidth::Word => {
                let ptr = addr.as_usize() as *mut u16;
                unsafe { ptr.write_volatile(data as u16) }
            }
            AccessWidth::Dword => {
                let ptr = addr.as_usize() as *mut u32;
                unsafe { ptr.write_volatile(data as u32) }
            }
            AccessWidth::Qword => {
                let ptr = addr.as_usize() as *mut u64;
                unsafe { ptr.write_volatile(data) }
            }
        }
        Ok(())
    }

    pub fn check_addr(&self, addr: VirtAddr) -> bool {
        addr >= self.start && addr < (self.start + self.size)
    }
}

pub struct VmDevGroup {
    devices: Vec<Arc<VmDev>>
}

impl VmDevGroup {
    pub fn new() -> Self {
        Self { devices: Vec::new() }
    }

    pub fn add_dev(&mut self, addr: VirtAddr, size: usize) {
        let dev = VmDev::new(addr, size);
        self.devices.push(Arc::new(dev));
    }

    pub fn find_dev(&self, addr: VirtAddr) -> Option<Arc<VmDev>> {
        self.devices
            .iter()
            .find(|&dev| dev.check_addr(addr))
            .cloned()
    }
}
