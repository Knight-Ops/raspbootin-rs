#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![no_main]
#![no_std]

mod bsp;

mod runtime_init;

use cortex_a::asm;

fn kernel_entry() -> ! {
    let mut mbox = bsp::mbox::Mbox::new();
    let uart = bsp::Uart::new();

    if uart.init(&mut mbox, 4_000_000).is_err() {
        asm::wfe();
    }

    for c in "RBIN64\r\n".chars() {
        uart.send(c);
    }

    uart.send(3 as char);
    uart.send(3 as char);
    uart.send(3 as char);

    let mut size: u32 = u32::from(uart.getc());
    size |= u32::from(uart.getc()) << 8;
    size |= u32::from(uart.getc()) << 16;
    size |= u32::from(uart.getc()) << 24;

    uart.send('O');
    uart.send('K');

    let kernel_addr: *mut u8 = 0x80_000 as *mut u8;
    unsafe {
        for i in 0..size {
            *kernel_addr.offset(i as isize) = uart.getc();
        }
    }

    let kernel: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_addr as *const ()) };
    kernel()
}
