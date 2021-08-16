use crate::{Color, ColoredStream, Segment};
use std::io::Write as _;

pub struct User {
    is_root: bool,
}

impl User {
    pub fn new() -> Self {
        Self {
            is_root: unsafe { libc::getuid() == 0 },
        }
    }
}

impl Segment for User {
    fn bg(&mut self) -> Color {
        if self.is_root {
            Color::from_rgb(255, 30, 30)
        } else {
            Color::from_rgb(30, 30, 255)
        }
    }
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        w.set_fg(Color::from_rgb(230, 230, 230))?;
        // write!(w, " 👤 ")?;
        write!(w, " ")?;
        unsafe {
            let pw = libc::getpwuid(libc::getuid());
            let mut name: *mut libc::c_char = (*pw).pw_name;
            loop {
                if *name == 0 {
                    break;
                }
                write!(w, "{}", *name as u8 as char)?;
                name = name.add(1);
            }
        };
        write!(w, " ")?;
        Ok(())
    }
}
