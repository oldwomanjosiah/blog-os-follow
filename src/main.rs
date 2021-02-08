#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga_buffer;
mod serial;

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("Starting blog_os 0.1.0");

    #[cfg(test)]
    test_main();

    println!("Hello QEMU!");

    for i in (0..=100).step_by(10) {
        println!("Loading: {}%", i);
    }

    loop {}
}

#[panic_handler]
#[cfg(not(test))]
fn panic(panic_info: &PanicInfo) -> ! {
    {
        let mut writer = vga_buffer::WRITER.lock();
        use vga_buffer::{Color, ColorCode};
        writer.set_color(ColorCode::from_colors(Color::Red, Color::Black));
    }
    println!("{}", panic_info);
    loop {}
}

#[panic_handler]
#[cfg(test)]
fn panic_test(panic_info: &PanicInfo) -> ! {
    println_serial!("[ ABRT ]: {}", panic_info);
    exit_qemu(QemuExitCode::Failed);
    // We will never get here but we need this loop in order to satisfy return type !
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    println_serial!("[      ]: Running {} Tests", tests.len());

    for test in tests {
        println_serial!();
        test.run();
    }

    println_serial!("\n[      ]: All Tests passed");

    exit_qemu(QemuExitCode::Success);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        Port::new(0xf4)
            .write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where T: Fn() -> () {
    fn run(&self) -> () {
        println_serial!("[ TEST ]: {}", core::any::type_name::<T>());
        self();
        println_serial!("[  OK  ]");
    }
}

#[cfg(test)]
mod main_tests {
    use super::*;

    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1 + 1, 2, "Uh oh.");
    }
}
