use super::gpio;
use super::mbox::{Clocks, Mbox};
use super::MMIO_BASE;
use core::fmt;
use core::ops;
use cortex_a::asm;
use register::{mmio::*, register_bitfields};

// PL011 UART registers.
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// Flag Register
    FR [
        /// Transmit FIFO full. The meaning of this bit depends on the
        /// state of the FEN bit in the UARTLCR_ LCRH Register. If the
        /// FIFO is disabled, this bit is set when the transmit
        /// holding register is full. If the FIFO is enabled, the TXFF
        /// bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1) [],

        /// Receive FIFO empty. The meaning of this bit depends on the
        /// state of the FEN bit in the UARTLCR_H Register. If the
        /// FIFO is disabled, this bit is set when the receive holding
        /// register is empty. If the FIFO is enabled, the RXFE bit is
        /// set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1) []
    ],

    /// Integer Baud rate divisor
    IBRD [
        /// Integer Baud rate divisor
        IBRD OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud rate divisor
    FBRD [
        /// Fractional Baud rate divisor
        FBRD OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control register
    LCRH [
        /// Word length. These bits indicate the number of data bits
        /// transmitted or received in a frame.
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ]
    ],

    /// Control Register
    CR [
        /// Receive enable. If this bit is set to 1, the receive
        /// section of the UART is enabled. Data reception occurs for
        /// UART signals. When the UART is disabled in the middle of
        /// reception, it completes the current character before
        /// stopping.
        RXE    OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// Transmit enable. If this bit is set to 1, the transmit
        /// section of the UART is enabled. Data transmission occurs
        /// for UART signals. When the UART is disabled in the middle
        /// of transmission, it completes the current character before
        /// stopping.
        TXE    OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UART enable
        UARTEN OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission
            /// or reception, it completes the current character
            /// before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Interupt Clear Register
    ICR [
        /// Meta field for all pending interrupts
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

const UART_BASE: u32 = MMIO_BASE + 0x20_1000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    DR: ReadWrite<u32>,                   // 0x00
    __reserved_0: [u32; 5],               // 0x04
    FR: ReadOnly<u32, FR::Register>,      // 0x18
    __reserved_1: [u32; 2],               // 0x1c
    IBRD: WriteOnly<u32, IBRD::Register>, // 0x24
    FBRD: WriteOnly<u32, FBRD::Register>, // 0x28
    LCRH: WriteOnly<u32, LCRH::Register>, // 0x2C
    CR: WriteOnly<u32, CR::Register>,     // 0x30
    __reserved_2: [u32; 4],               // 0x34
    ICR: WriteOnly<u32, ICR::Register>,   // 0x44
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

            (*gpio::GPPUDCLK0).modify(
                gpio::GPPUDCLK0::PUDCLK14::AssertClock + gpio::GPPUDCLK0::PUDCLK15::AssertClock,
            );
            for _ in 0..150 {
                asm::nop();
            }

            (*gpio::GPPUDCLK0).set(0);
        }

        self.ICR.write(ICR::ALL::CLEAR);
        self.IBRD.write(IBRD::IBRD.val(2)); // Results in 115200 baud
        self.FBRD.write(FBRD::FBRD.val(0xB));
        self.LCRH.write(LCRH::WLEN::EightBit); // 8N1
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
    #[inline(never)]
    pub fn getc(&self) -> u8 {
        // wait until something is in the buffer
        loop {
            if !self.FR.is_set(FR::RXFE) {
                break;
            }

            asm::nop();
        }

        // read it and return
        self.DR.get() as u8
    }
}
