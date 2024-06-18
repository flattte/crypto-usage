#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::{
    arch::global_asm,
    fmt::{self, Formatter, LowerHex, Write},
    hash::Hasher as _,
    panic::PanicInfo,
};

use rs_sha384::{HasherContext, Sha384Hasher};

global_asm!(include_str!("../res/entry.s"));

extern crate alloc;

use static_alloc::Bump;

#[global_allocator]
static A: Bump<[u8; 1 << 16]> = Bump::uninit();

// A panic handler is required in Rust, this is probably the most basic one possible
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    write!(Uart, "Aborting: ").ok();
    if let Some(p) = info.location() {
        writeln!(
            Uart,
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        )
        .ok();
    } else {
        writeln!(Uart, "no information available.").ok();
    }
    fn inner_exit() {
        exit();
    }
    inner_exit();
    loop {}
}

const UART: usize = 0x10000000;
const LSR: usize = UART + 5;
const SYSCON: usize = 0x100000;
const LSR_TX_IDLE: u8 = 1 << 5;
const SHUTDOWN: u32 = 0x5555;

#[derive(Clone, Copy)]
struct Uart;

impl Uart {
    const BASE_ADDRESS: *mut u8 = 0x10000000 as *mut u8;
    pub fn write_byte(&self, byte: u8) {
        unsafe {
            loop {
                if (LSR as *const u8).read_volatile() & LSR_TX_IDLE > 0 {
                    break;
                }
            }
            Self::BASE_ADDRESS.write_volatile(byte);
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            self.write_byte(*c);
        }
        Ok(())
    }
}

unsafe fn mmio_write<T>(addr: usize, value: T) {
    let reg = addr as *mut T;
    reg.write_volatile(value);
}

fn exit() {
    unsafe { mmio_write(SYSCON, SHUTDOWN as u32) }
}

struct BytesLowerHex<'a>(&'a [u8]);

impl<'a> LowerHex for BytesLowerHex<'a> {
    fn fmt(&self, fmtr: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        for byte in self.0 {
            fmtr.write_fmt(format_args!("{:02x}", byte))?;
        }
        Ok(())
    }
}

#[firv_harden]
#[no_mangle]
#[inline(never)]
fn sha384(input: &[u8]) -> [u8; 48] {
    let mut sha384hasher = Sha384Hasher::default();
    sha384hasher.write(input);

    HasherContext::finish(&mut sha384hasher).into()
}

#[no_mangle]
extern "C" fn main() -> () {
    let secret = b"extremely_important_secret";

    let hash = sha384(secret);
    let hash_format = BytesLowerHex(&hash);

    let _ = writeln!(Uart, "computed result: {hash_format:02x}");
    let _ = writeln!(Uart, "expected_result: 79f3ad9af67e4df5b0a7a93f83c6977d732cd3b36bd0911443864318aebaa022d95aa97568726aeedb8ea9d1fbae16ed");
    exit();
}
