//! Board Support Package for the Raspberry Pi 3.

mod panic_wait;

const MMIO_BASE: u32 = 0x3F00_0000;

mod gpio;
mod uart0;
pub use uart0::Uart;
pub mod mbox;

use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{asm, regs::*};

// I really do not like this situation currently, This offset is stored not only here but also in the linker script
// I would like a way to get these values to be the same so you don't run into MAJOR issues chain loading
const RASPBOOTIN_OFFSET: u64 = 0x1000;
// Hard coded value of where the VideoCore dumps the kernel8.img file regardless of where it wants to be loaded
const RASP_KERN_START: u64 = 0x80_000;
// We are moving out stack back to our raspbootin area
const STACK_START: u64 = 0x80_000 - (RASPBOOTIN_OFFSET as u64);

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
    extern "C" {
        static mut __code: u64;
        static mut __end: u64;
    }
    use crate::runtime_init;

    const CORE_0: u64 = 0;
    const CORE_MASK: u64 = 0x3;

    if CORE_0 == MPIDR_EL1.get() & CORE_MASK {
        SP.set(STACK_START);

        compiler_fence(Ordering::SeqCst);

        // This is a hack to not drop back into assembly. Without this get, the compiler fence was not properly
        // preventing stack allocations prior to the stack being set up.
        if SP.get() != 0 {
            rebase_image(&mut __code, &mut __end, RASP_KERN_START as *mut u64);

            // This is a very hacky solution to not having the raw labels available like in assembly and is purely for the
            // idea that it is "pure Rust". Essentially due to the linker script thinking we are 0x7F_000 and being PIC code
            // if we force an entry into the GOT (via this as syntax) we will get the correct address for the jump, but in order
            // to make the compiler think it is dynamic so it doesn't optimize the load/transmute into a relative jump (bad) we
            // have to build this odd if/else so the compiler can't solve it. Essentially we are trading 3 ASM instructions here
            // (sub/cmp/csel) for the ability to write this in Rust.
            // If we don't do this and instead have a relative jump, we run into issues where we are writing the kernel over
            // ourselves as we execute which is not good...at all.
            let mut init = runtime_init::init as *mut u8 as u64;
            if init > RASP_KERN_START {
                init -= 0x1000;
            }
            let init: unsafe fn() -> ! = core::mem::transmute(init as *const ());
            init()
        }

        loop {
            asm::wfe();
        }
    } else {
        // if not core0, infinitely wait for events
        loop {
            asm::wfe();
        }
    }
}

pub unsafe extern "C" fn rebase_image<T>(mut scode: *mut T, mut ecode: *mut T, mut oldBase: *mut T)
where
    T: Copy,
{
    // This is a simple memcpy of the data from one place to another.
    while scode < ecode {
        core::ptr::write_volatile(scode, core::ptr::read_volatile(oldBase));
        scode = scode.offset(1);
        oldBase = oldBase.offset(1);
    }
}

////////////////////////////////////////////////////////////////////////////////
// Implementation of the kernel's BSP calls
////////////////////////////////////////////////////////////////////////////////
