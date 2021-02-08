use uart_16550::SerialPort;
use spin::Mutex;

lazy_static::lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // Safety:
        // We know that this is okay as this address is the default address for the
        // first serial port on x86_64
        Mutex::new(unsafe {
            let mut sp = SerialPort::new(0x3F8);
            sp.init();
            sp
        })
    };
}

use core::fmt;

pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Could not write to serial 1");
}

#[macro_export]
macro_rules! print_serial {
    () => ();
    ($($args:tt)*) => {{
        $crate::serial::_print(format_args!($($args)*));
    }};
}

#[macro_export]
macro_rules! println_serial {
    () => (print_serial!("\n"));
    ($fmt:expr) => ($crate::print_serial!(concat!($fmt, "\n")));
    ($fmt:expr, $($args:tt)*) => {
        $crate::print_serial!(concat!($fmt, "\n"), $($args)*)
    };
}
