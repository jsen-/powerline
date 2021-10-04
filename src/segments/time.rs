use crate::{Color, ColoredStream, Segment};
use chrono::Timelike;
use std::io::Write as _;

pub struct Time;

impl Segment for Time {
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        w.set_bg(Color::from_rgb(80, 80, 80))?;
        w.set_fg(Color::from_rgb(200, 200, 200))?;
        let time = chrono::Local::now().time();

        let minute = time.minute();
        let (hour, half) = {
            let (_, hour) = time.hour12();
            if minute >= 45 {
                (hour + 1, false)
            } else if minute > 15 {
                (hour, true)
            } else {
                (hour, false)
            }
        };
        // source: https://www.alt-codes.net/clock-symbols
        let code = (hour - 1) % 12 + 128336 + if half { 12 } else { 0 };
        write!(
            w,
            " {} {} ",
            unsafe { char::from_u32_unchecked(code) },
            chrono::Local::now().format("%T.%3f")
        )?;
        Ok(())
    }
}
