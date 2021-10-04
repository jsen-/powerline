use crate::{Color, ColoredStream, Segment};
use std::io::Write;

pub struct ExitCode {
    code: Option<i32>,
}

impl ExitCode {
    pub fn new(code: Option<i32>) -> Self {
        Self { code }
    }
}

impl Segment for ExitCode {
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        let bg = if let Some(code) = self.code {
            if code == 0 {
                Color::from_rgb(0, 100, 0)
            } else {
                Color::from_rgb(100, 0, 0)
            }
        } else {
            Color::from_rgb(0, 0, 100)
        };
        w.set_bg(bg)?;
        w.set_fg(Color::from_rgb(200, 200, 200))?;
        match self.code {
            Some(0) | None => write!(w, "   "),
            Some(code) => write!(w, " {} ", code),
        }
    }
}
