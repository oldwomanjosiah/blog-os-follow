use core::fmt;

lazy_static::lazy_static! {
    pub static ref WRITER: spin::Mutex<Writer> = {
        spin::Mutex::new(Writer {
            column_position: 0,
            current_color: ColorCode::from_colors(Color::Green, Color::Black),
            buffer: unsafe { &mut *(VGA_BUFFER as *mut Buffer) },
        })
    };
}

#[macro_export]
macro_rules! print {
    () => ();
    ($($args:tt)*) => {{
        $crate::vga_buffer::_print(format_args!($($args)*)).expect("Could not write to stdout");
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!(b'\n'));
    ($($args:tt)*) => {{
        $crate::print!("{}\n", format_args!($($args)*));
    }};
}

#[allow(unused_must_use)]
pub fn _print(output: fmt::Arguments) -> fmt::Result {
    use core::fmt::Write;
    WRITER.lock().write_fmt(output)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,

    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn from_colors(foreground: Color, background: Color) -> Self {
        Self(
            (background as u8) << 4 |
            (foreground as u8)
            )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_char: u8,
    pub color: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const VGA_BUFFER: *mut u8 = 0xb8000 as _;

use volatile::Volatile;

#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    current_color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(
                    ScreenChar {
                        ascii_char: byte,
                        color: self.current_color,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn write_str(&mut self, write: &str) -> fmt::Result {
        for ascii_char in write.bytes() {
            match ascii_char {
                0x20..=0x7e | b'\n' => self.write_byte(ascii_char),

                // Not a printable char
                _ => self.write_byte(0xfE),
            }
        }
        Ok(())
    }

    pub fn set_color(&mut self, color: ColorCode) -> ColorCode {
        let old = self.current_color;
        self.current_color = color;
        old
    }

    /* Would like to be able to return old color but may require mem::transmute?
     * Should be safe as any u8 in the range [0x00, 0x0F] is a discriminant but
     * pub fn set_fg(&mut self, fg: Color) -> Color {
        let old = (self.current_color.0 & 0x0F) as Color;
        self.current_color.0 = (self.current_color.0 & 0xF0) | fg as u8;
        old

    } */

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let move_char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(move_char);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        debug_assert!(row < BUFFER_HEIGHT, "We cant clear a row that does not exist!");

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(
                ScreenChar {
                    ascii_char: b' ',
                    color: self.current_color,
                });
        }
    }
}


impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn println_works() {
        println!("I can in fact println");
    }

    #[test_case]
    fn println_many() {
        for i in 0..100 {
            println!("{}", i);
        }
    }

    #[test_case]
    fn print_to_screen() {
        let to_screen = "Prints to Screen";

        println!("{}", to_screen);

        let buffer = WRITER.lock();

        for (i, c) in to_screen.chars().enumerate() {
            assert_eq!(
                    char::from((buffer.chars[BUFFER_HEIGHT - 2][i]).read().ascii_char),
                    c
                );
        }
    }
}
