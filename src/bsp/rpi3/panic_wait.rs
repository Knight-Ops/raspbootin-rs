//! A panic handler that infinitely waits.

use crate::println;
use core::panic::PanicInfo;
use cortex_a::asm;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        asm::wfe();
    }
}
