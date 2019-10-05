#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![no_main]
#![no_std]

mod bsp;

mod runtime_init;

mod interface;
mod print;

fn kernel_entry() -> ! {
    // uart.puts("Hello from pure Rust!\n");
    println!("Hello from pure Rust!");

    let mut mbox = bsp::mbox::Mbox::new();

    let macAddr = mbox.get_board_mac().unwrap();
    println!("Got my MAC : {:X}", macAddr);
    // // let (clock_id, clock_speed) = mbox
    // //     .set_clock_rate(bsp::mbox::Clocks::UART, 4_000_000, 0)
    // //     .unwrap();

    // uart.puts("My MAC is ");
    // uart.hex((macAddr >> 32) as u32);
    // uart.hex(macAddr as u32);
    // uart.puts("\n");

    // uart.puts("My UART speed is ");
    // uart.hex(clock_id);
    // uart.hex(clock_speed);
    // uart.puts("\n");

    panic!("Stopping at end of kernel_entry");
}
