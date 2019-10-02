use super::gpio;
use super::mbox::{Clocks, Mbox};
use super::MMIO_BASE;
use crate::interface::console;
use core::fmt;
use core::ops;
use cortex_a::asm;
use register::{mmio::*, register_bitfields};

register_bitfields! {
    u32,

    DR [
        OE OFFSET(11) NUMBITS(1) [],

        BE OFFSET(10) NUMBITS(1) [],

        PE OFFSET(9) NUMBITS(1) [],

        FE OFFSET(8) NUMBITS(1) [],

        DATA OFFSET(0) NUMBITS(8) []
    ],

    RSRECR [
        OE OFFSET(3) NUMBITS(1) [],
        BE OFFSET(2) NUMBITS(1) [],
        PE OFFSET(1) NUMBITS(1) [],
        FE OFFSET(0) NUMBITS(1) []
    ],

    FR [
        RI OFFSET(8) NUMBITS(1) [],
        TXFE OFFSET(7) NUMBITS(1) [],
        RXFF OFFSET(6) NUMBITS(1) [],
        TXFF OFFSET(5) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
        DCD OFFSET(2) NUMBITS(1) [],
        DSR OFFSET(1) NUMBITS(1) [],
        CTS OFFSET(0) NUMBITS(1) []
    ],

    IBRD [
        IBRD OFFSET(0) NUMBITS(16) []
    ],

    FBRD [
        FBRD OFFSET(0) NUMBITS(6) []
    ],

    LCRH [
        SPS OFFSET(7) NUMBITS(1) [],
        WLEN OFFSET(5) NUMBITS(2) [
            Fivebit = 0b00,
            Sixbit = 0b01,
            Sevenbit = 0b10,
            Eightbit = 0b11
        ],
        FEN OFFSET(4) NUMBITS(1) [],
        STP2 OFFSET(3) NUMBITS(1) [],
        EPS OFFSET(2) NUMBITS(1) [],
        PEN OFFSET(1) NUMBITS(1) [],
        BRK OFFSET(0) NUMBITS(1) []
    ],

    CR [
        CTSEN OFFSET(15) NUMBITS(1) [],
        RTSEN OFFSET(14) NUMBITS(1) [],
        RTS OFFSET(11) NUMBITS(1) [],
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0b0,
            Enabled = 0b1
        ],
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0b0,
            Enabled = 0b1
        ],
        LBE OFFSET(7) NUMBITS(1) [],
        UARTEN OFFSET(0) NUMBITS(1) [
            Disabled = 0b0,
            Enabled = 0b1
        ]
    ],

    ICR [ // TODO this is wrong
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

const UART_BASE: u32 = MMIO_BASE + 0x20_1000;

//TODO this needs to be completely filled out
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    DR: ReadWrite<u32, DR::Register>,         // 0x00
    RSRECR: ReadWrite<u32, RSRECR::Register>, // 0x04
    _reserved0: [u32; 4],                     // 0x08
    FR: ReadWrite<u32, FR::Register>,         // 0x18
    _reserved1: [u32; 1],                     // 0x1c
    ILPR: ReadWrite<u32>,                     // 0x20
    IBRD: ReadWrite<u32, IBRD::Register>,     // 0x24
    FBRD: ReadWrite<u32, FBRD::Register>,     // 0x28
    LCRH: ReadWrite<u32, LCRH::Register>,     // 0x2c
    CR: ReadWrite<u32, CR::Register>,         // 0x30
    _reserved2: [u32; 4],                     //TODO                // 0x34
    ICR: WriteOnly<u32, ICR::Register>,       // 0x44
}

pub enum UartError {
    MailboxError,
}
pub type Result<T> = ::core::result::Result<T, UartError>;

pub struct Uart;

impl ops::Deref for Uart {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Uart {
    pub fn new() -> Uart {
        Uart
    }

    fn ptr() -> *const RegisterBlock {
        UART_BASE as *const _
    }

    pub fn init(&self, mbox: &mut Mbox, clock_speed: u32) -> Result<()> {
        self.CR.set(0);

        mbox.set_clock_rate(Clocks::UART, clock_speed, 0);

        // map UART0 to GPIO pins
        unsafe {
            (*gpio::GPFSEL1).modify(gpio::GPFSEL1::FSEL14::TXD0 + gpio::GPFSEL1::FSEL15::RXD0);

            (*gpio::GPPUD).set(0); // enable pins 14 and 15
            for _ in 0..150 {
                asm::nop();
            }

            (*gpio::GPPUDCLK0).write(
                gpio::GPPUDCLK0::PUDCLK14::AssertClock + gpio::GPPUDCLK0::PUDCLK15::AssertClock,
            );
            for _ in 0..150 {
                asm::nop();
            }

            (*gpio::GPPUDCLK0).set(0);
        }

        self.ICR.write(ICR::ALL::CLEAR);
        self.IBRD.write(IBRD::IBRD.val(2));
        self.FBRD.write(FBRD::FBRD.val(0xB));
        self.LCRH.write(LCRH::WLEN::Eightbit);
        self.CR
            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);

        Ok(())
    }

    /// Send a character
    pub fn send(&self, c: char) {
        // wait until we can send
        loop {
            if !self.FR.is_set(FR::TXFF) {
                break;
            }

            asm::nop();
        }

        // write the character to the buffer
        self.DR.set(c as u32);
    }

    /// Receive a character
    pub fn getc(&self) -> char {
        // wait until something is in the buffer
        loop {
            if !self.FR.is_set(FR::RXFE) {
                break;
            }

            asm::nop();
        }

        // read it and return
        let mut ret = self.DR.get() as u8 as char;

        // convert carrige return to newline
        if ret == '\r' {
            ret = '\n'
        }

        ret
    }

    /// Display a string
    pub fn puts(&self, string: &str) {
        for c in string.chars() {
            // convert newline to carrige return + newline
            if c == '\n' {
                self.send('\r')
            }

            self.send(c);
        }
    }

    /// Display a binary value in hexadecimal
    pub fn hex(&self, d: u32) {
        let mut n;

        for i in 0..8 {
            // get highest tetrad
            n = d.wrapping_shr(28 - i * 4) & 0xF;

            // 0-9 => '0'-'9', 10-15 => 'A'-'F'
            // Add proper offset for ASCII table
            if n > 9 {
                n += 0x37;
            } else {
                n += 0x30;
            }

            self.send(n as u8 as char);
        }
    }
}
