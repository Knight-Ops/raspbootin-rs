//! A panic handler that infinitely waits.

use core::panic::PanicInfo;
use cortex_a::asm;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {
        asm::wfe();
    }
}
