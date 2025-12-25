#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// VM exit hypercall number
const VM_EXIT_HYPERCALL: u64 = 0;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    // In x86, we can use a triple fault or HLT to exit
    // For simplicity, we'll cause a triple fault by loading invalid segment
    
    // Load an invalid segment selector to cause triple fault
    // This will cause a VM exit which the hypervisor can handle
    core::arch::asm!(
        "xor ax, ax",
        "mov ds, ax",
        "mov ss, ax",
        "hlt", // HLT will cause VM-exit due to HLT exiting control
        options(noreturn)
    );
    
    // Alternative: cause triple fault by invalid stack
    core::arch::asm!(
        "mov rsp, 0",
        "push rax",
        options(noreturn)
    );
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
