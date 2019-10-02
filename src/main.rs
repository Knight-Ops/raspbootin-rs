#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![no_main]
#![no_std]

mod bsp;

mod runtime_init;

mod interface;
mod print;

use core::sync::atomic::{compiler_fence, Ordering};

fn kernel_entry() -> ! {
    let mut uart = bsp::Uart::new();
    let mut mbox = bsp::mbox::Mbox::new();
    {
        uart.init(&mut mbox, 4_000_000);
    }
    uart.puts("Hello from pure Rust!\n");

    let macAddr = mbox.get_board_mac().unwrap();
    // let (clock_id, clock_speed) = mbox
    //     .set_clock_rate(bsp::mbox::Clocks::UART, 4_000_000, 0)
    //     .unwrap();

    uart.puts("My MAC is ");
    uart.hex((macAddr >> 32) as u32);
    uart.hex(macAddr as u32);
    uart.puts("\n");

    // uart.puts("My UART speed is ");
    // uart.hex(clock_id);
    // uart.hex(clock_speed);
    // uart.puts("\n");

    panic!("Stopping at end of kernel_entry");
}
