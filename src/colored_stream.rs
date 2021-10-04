use crate::{Color, ColorableStream, Segment};
use std::io::{self, Write as _};

pub struct ColoredStream<'a> {
    empty: bool,
    segment_start: bool,
    bg: Color,
    inner: &'a mut dyn ColorableStream,
}

impl<'a> ColoredStream<'a> {
    pub fn new(inner: &'a mut dyn ColorableStream) -> Self {
        Self {
            empty: true,
            segment_start: false,
            bg: Color::from_rgb(0, 0, 0),
            inner,
        }
    }
    pub fn start_segment(&mut self, bg: Color) -> io::Result<()> {
        self.segment_start = false;
        if !self.empty {
            self.set_fg_inner(self.bg)?;
            self.set_bg_inner(bg)?;
            write!(self.inner, "")?;
        } else {
            self.set_bg_inner(bg)?;
        }
        self.empty = false;
        Ok(())
    }
    fn segment_finished(&mut self) {
        self.segment_start = true;
    }
    pub fn set_fg_inner(&mut self, color: Color) -> io::Result<()> {
        self.empty = false;
        write!(self.inner, "\x01\x1B[38;2;{};{};{}m\x02", color.r, color.g, color.b)
    }
    pub fn set_fg(&mut self, color: Color) -> io::Result<()> {
        if self.segment_start {
            self.start_segment(self.bg)
        } else {
            self.set_fg_inner(color)
        }
    }
    fn set_bg_inner(&mut self, color: Color) -> io::Result<()> {
        self.empty = false;
        self.bg = color;
        write!(self.inner, "\x01\x1B[48;2;{};{};{}m\x02", color.r, color.g, color.b)
    }
    pub fn set_bg(&mut self, color: Color) -> io::Result<()> {
        if self.segment_start {
            self.start_segment(color)
        } else {
            self.set_bg_inner(color)
        }
    }
    pub fn write_segment(&mut self, segment: &mut dyn Segment) -> io::Result<()> {
        self.segment_finished();
        segment.write(self)?;
        self.segment_finished();
        Ok(())
    }
    pub fn end_line(&mut self) -> io::Result<()> {
        let fg = self.bg;
        self.reset()?;
        self.set_fg_inner(fg)?;
        self.segment_start = false;
        write!(self, "")?;
        self.empty = true;
        Ok(())
    }
    pub fn new_line(&mut self) -> io::Result<()> {
        self.end_line()?;
        writeln!(self)?;
        self.empty = true;
        Ok(())
    }

    pub fn reset(&mut self) -> io::Result<()> {
        self.bg = Color::from_rgb(0, 0, 0);
        write!(self.inner, "\x01\x1B[0m\x02")
    }
}

impl<'a> std::io::Write for ColoredStream<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.segment_start {
            self.start_segment(self.bg)?;
        }
        self.empty = false;
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
