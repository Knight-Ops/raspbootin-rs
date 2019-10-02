//! Board Support Package for the Raspberry Pi 3.

mod panic_wait;

const MMIO_BASE: u32 = 0x3F00_0000;

mod gpio;
mod uart0;
pub use uart0::Uart;
mod uart1;
pub use uart1::MiniUart;
pub mod mbox;

use crate::interface::console;
use core::fmt;
use cortex_a::{asm, regs::*};

/// The entry of the `kernel` binary.
///
/// The function must be named `_start`, because the linker is looking for this
/// exact name.
///
/// # Safety
///
/// - Linker script must ensure to place this function at `0x80_000`.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    use crate::runtime_init;

    const CORE_0: u64 = 0;
    const CORE_MASK: u64 = 0x3;
    const STACK_START: u64 = 0x80_000;

    if CORE_0 == MPIDR_EL1.get() & CORE_MASK {
        SP.set(STACK_START);
        runtime_init::init()
    } else {
        // if not core0, infinitely wait for events
        loop {
            asm::wfe();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Implementation of the kernel's BSP calls
////////////////////////////////////////////////////////////////////////////////

/// Returns a ready-to-use `console::Write` implementation.
// This is a terrible implementation, we should have to re-init each time we need this
pub fn console() -> impl console::Write {
    let console = MiniUart::new();
    console.init();

    console
}
