use crate::{Color, ColoredStream, Segment};
use std::io::Write;

pub struct Cwd;

impl Segment for Cwd {
    fn bg(&mut self) -> Color {
        Color::from_rgb(60, 60, 60)
    }
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        let wd = std::env::current_dir().unwrap_or(std::path::PathBuf::from(" /?/?/? "));
        w.set_fg(Color::from_rgb(210, 210, 210))?;
        if let Some(homedir) = dirs::home_dir() {
            if let Ok(stripped) = wd.strip_prefix(&homedir) {
                write!(w, " ~/{} ", stripped.display())?;
                return Ok(());
            }
        }
        write!(w, " {} ", wd.display())
    }
}
