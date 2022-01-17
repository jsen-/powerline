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

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    let mut time = segments::Time;
    let mut hostname = segments::Hostname::new();
    let mut user = segments::User::new();
    let mut cwd = segments::Cwd;
    let mut git = segments::Git::new();
    let mut openstack = segments::Openstack::new();
    let mut k8s = segments::K8s::new();
    let mut exitcode = segments::ExitCode::new(args.exit_code);

    let segments = [
        &mut time as &mut dyn Segment,
        &mut hostname,
        &mut user,
        &mut cwd,
        &mut git,
        &mut openstack,
        &mut k8s,
    ];

    let stdout_ = std::io::stdout();
    let mut stdout_lock = stdout_.lock();
    let mut stream = ColoredStream::new(&mut stdout_lock);

    let segments = segments.into_iter();
    for segment in segments {
        stream.write_segment(segment)?;
    }
    stream.new_line()?;
    stream.write_segment(&mut exitcode)?;
    stream.end_line()?;
    stream.reset()?;
    write!(stream, " ")?;
    Ok(())
}
