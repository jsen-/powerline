use crate::{ColoredStream};


mod time;
pub use time::Time;

mod hostname;
pub use self::hostname::Hostname;

mod user;
pub use user::User;

mod cwd;
pub use cwd::Cwd;

mod git;
pub use git::Git;

mod openstack;
pub use openstack::Openstack;

mod k8s;
pub use k8s::K8s;

mod exitcode;
pub use exitcode::ExitCode;

pub trait Segment {
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()>;
}
