#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// VM exit hypercall number
const VM_EXIT_HYPERCALL: u64 = 0;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    // Execute HLT instruction to trigger VM-exit
    // This allows the hypervisor to detect that guest has started and executed successfully
    loop {
        core::arch::asm!("hlt");
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
