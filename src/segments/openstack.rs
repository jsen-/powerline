use std::{env, io::Write as _};

use crate::{Color, ColoredStream, Segment};

pub struct Openstack {
    project_name: Option<String>,
}

impl Openstack {
    pub fn new() -> Self {
        env::var_os("OS_PROJECT_NAME")
            .or_else(|| env::var_os("OS_TENANT_NAME"))
            .map(|project_name| Self {
                project_name: Some(project_name.to_string_lossy().into_owned()),
            })
            .unwrap_or(Self { project_name: None })
    }
}

impl Segment for Openstack {
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        if let Some(ref project_name) = self.project_name {
            w.set_bg(Color::from_rgb(80, 80, 255))?;
            w.set_fg(Color::from_rgb(255, 255, 255))?;
            write!(w, " ⏹  {} ", project_name)?;
        }
        Ok(())
    }
}
