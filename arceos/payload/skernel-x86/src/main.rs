#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// VM exit hypercall number
const VM_EXIT_HYPERCALL: u64 = 0;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    panic!("VM exited via hypercall!");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
