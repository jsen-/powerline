use std::{env, io::Write as _};

use crate::{Color, ColoredStream, Segment};

pub struct Openstack {
    project_name: String,
}

impl Openstack {
    pub fn new() -> Option<Self> {
        env::var_os("OS_PROJECT_NAME")
            .or_else(|| env::var_os("OS_TENANT_NAME"))
            .map(|val| Self {
                project_name: val.to_string_lossy().into_owned(),
            })
    }
}

impl Segment for Openstack {
    fn bg(&mut self) -> Color {
        Color::from_rgb(50, 50, 255)
    }

    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        w.set_fg(Color::from_rgb(200, 200, 255))?;
        write!(w, " ⏹  {} ", self.project_name)
    }
}
