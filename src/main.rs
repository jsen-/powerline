mod colored_stream;
mod segments;

pub use crate::colored_stream::ColoredStream;
pub use segments::Segment;

use std::io::{self, Write};

pub trait ColorableStream: Write {}

impl ColorableStream for std::io::Stdout {}
impl<'a> ColorableStream for std::io::StdoutLock<'a> {}
impl ColorableStream for std::io::Stderr {}
impl<'a> ColorableStream for std::io::StderrLock<'a> {}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(argh::FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, short = 'e', long = "exit-code")]
    /// value for exit-code segment
    exit_code: Option<i32>,
}

fn as_mut_dyn_segment<T: Segment>(t: &mut T) -> &mut dyn Segment {
    t
}

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    let mut time = segments::Time;
    let mut user = segments::User::new();
    let mut cwd = segments::Cwd;
    let mut hostname = segments::Hostname::new();
    let mut git = segments::Git::new();
    let mut gitstatus = git.as_ref().and_then(|git| segments::GitStatus::new(git));
    let mut openstack = segments::Openstack::new();
    let mut k8s = segments::K8s::new();
    let mut exitcode = segments::ExitCode::new(args.exit_code);

    let segments = [
        Some(&mut time as &mut dyn Segment),
        Some(&mut hostname),
        Some(&mut user),
        Some(&mut cwd),
        git.as_mut().map(as_mut_dyn_segment),
        gitstatus.as_mut().map(as_mut_dyn_segment),
        openstack.as_mut().map(as_mut_dyn_segment),
        k8s.as_mut().map(as_mut_dyn_segment),
        Some(&mut exitcode),
    ];

    let stdout_ = std::io::stdout();
    let mut stdout_lock = stdout_.lock();
    let mut stdout = ColoredStream::new(&mut stdout_lock);

    let segments = std::array::IntoIter::new(segments).filter_map(|v| v);
    let mut first = true;
    for cur in segments {
        let bg = cur.bg();
        let fg = stdout.get_bg();
        stdout.set_fg(fg)?;
        stdout.set_bg(bg)?;
        if first {
            first = false;
        } else {
            write!(&mut stdout, "")?;
        }
        cur.write(&mut stdout)?;
    }
    let fg = stdout.get_bg();
    stdout.reset()?;
    stdout.set_fg(fg)?;
    write!(&mut stdout, " ")?;
    stdout.reset()
}
