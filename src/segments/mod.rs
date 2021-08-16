use crate::{Color, ColoredStream};

mod cwd;
pub use cwd::Cwd;

mod exitcode;
pub use exitcode::ExitCode;

mod time;
pub use time::Time;

mod user;
pub use user::User;

mod hostname;
pub use self::hostname::Hostname;

mod git;
pub use git::Git;
pub use git::GitStatus;

mod openstack;
pub use openstack::Openstack;

mod k8s;
pub use k8s::K8s;

pub trait Segment {
    fn bg(&mut self) -> Color;
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()>;
}
