use crate::{Color, ColoredStream, Segment};
use std::{env, ffi::OsString, io::Write, path::Path};

pub struct Hostname {
    hostname: OsString,
    ssh: bool,
}

impl Hostname {
    pub fn new() -> Self {
        Self {
            hostname: hostname::get().unwrap_or(OsString::new()),
            ssh: env::var_os("SSH_CLIENT").is_some(),
        }
    }
}

impl Segment for Hostname {
    fn bg(&mut self) -> Color {
        if self.hostname.is_empty() {
            Color::from_rgb(255, 0, 0)
        } else if self.ssh {
            Color::from_rgb(255, 80, 0)
        } else {
            Color::from_rgb(30, 30, 30)
        }
    }
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        w.set_fg(Color::from_rgb(255, 255, 255))?;
        // HACK: make `Path` from `OsStr` so we can call `.display()`
        let icon = if self.ssh { '🔐' } else { '💻' };
        write!(w, " {} {} ", icon, Path::new(&self.hostname).display())?;
        Ok(())
    }
}
