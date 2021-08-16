use crate::{Color, ColorableStream};
use std::io;

pub struct ColoredStream<'a> {
    bg: Color,
    inner: &'a mut dyn ColorableStream,
}

impl<'a> ColoredStream<'a> {
    pub fn new(inner: &'a mut dyn ColorableStream) -> Self {
        Self {
            bg: Color::from_rgb(0, 0, 0),
            inner,
        }
    }
    pub fn set_fg(&mut self, color: Color) -> io::Result<()> {
        write!(self.inner, "\x1B[38;2;{};{};{}m", color.r, color.g, color.b)
    }
    pub fn set_bg(&mut self, color: Color) -> io::Result<()> {
        self.bg = color;
        write!(self.inner, "\x1B[48;2;{};{};{}m", color.r, color.g, color.b)
    }
    pub fn get_bg(&mut self) -> Color {
        self.bg
    }
    pub fn reset(&mut self) -> io::Result<()> {
        self.bg = Color::from_rgb(0, 0, 0);
        write!(self.inner, "\x1B[0m")
    }
}

impl<'a> std::io::Write for ColoredStream<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
