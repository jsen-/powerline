use crate::{Color, ColoredStream, Segment};
use git2::{Branch, BranchType, ErrorClass, ErrorCode, Repository, Status};
use std::io::Write as _;

pub struct Git(Option<GitInner>);

pub struct GitInner {
    state: Result<State, git2::Error>,
    statuses: Result<Statuses, git2::Error>,
}

#[derive(Debug, Clone, Copy)]
struct Upstream {
    ahead: u8,
    behind: u8,
}

enum State {
    Detached(git2::Buf),
    OnBranch {
        name: String,
        upstream: Option<Upstream>,
    },
    Empty,
}

macro_rules! impl_status {
    ($name:ident, $r:literal, $g:literal, $b:literal, $format:literal) => {
        #[derive(Debug)]
        struct $name(u8);
        impl Segment for $name {
            fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
                if self.0 > 0 {
                    w.set_fg(Color::from_rgb(0, 0, 0))?;
                    // w.set_fg(Color::from_rgb($r, $g, $b))?;
                    if self.0 < 99 {
                        write!(w, $format, self.0)?;
                    } else {
                        write!(w, $format, "99+")?;
                    }
                }
                Ok(())
            }
        }
    };
}

impl_status!(Staged, 0, 100, 0, " {}✓ ");
impl_status!(NotStaged, 0, 0, 255, " {}* ");
impl_status!(Untracked, 0, 0, 0, " {}⁺ ");
impl_status!(Conflicted, 180, 0, 0, " {}💔 ");

#[derive(Debug)]
struct Statuses {
    staged: Staged,
    not_staged: NotStaged,
    untracked: Untracked,
    conflicted: Conflicted,
}

impl Segment for Statuses {
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        if self.staged.0 > 0
            || self.not_staged.0 > 0
            || self.untracked.0 > 0
            || self.conflicted.0 > 0
        {
            w.start_segment(Color::from_rgb(200, 200, 200))?;
            if self.staged.0 > 0 {
                self.staged.write(w)?;
            }
            if self.not_staged.0 > 0 {
                self.not_staged.write(w)?;
            }
            if self.untracked.0 > 0 {
                self.untracked.write(w)?;
            }
            if self.conflicted.0 > 0 {
                self.conflicted.write(w)?;
            }
        }
        Ok(())
    }
}

pub fn get_head_branch(repo: &git2::Repository) -> Result<Option<Branch>, git2::Error> {
    let branches = repo.branches(Some(BranchType::Local))?;
    for branch in branches {
        let (branch, _ /*branch_type*/) = branch?;
        if branch.is_head() {
            return Ok(Some(branch));
        }
    }
    Ok(None)
}

fn get_state(repo: &git2::Repository) -> Result<State, git2::Error> {
    if repo.head_detached()? {
        let head = repo.head()?;
        let target = head.peel(git2::ObjectType::Any)?;
        let short_id = target.short_id()?;
        Ok(State::Detached(short_id))
    } else if let Some(branch) = get_head_branch(&repo)? {
        let local = branch.get().target().ok_or(git2::Error::new(
            ErrorCode::NotFound,
            ErrorClass::Object,
            "Target for local branch not found",
        ))?;
        let upstream = branch.upstream().ok().and_then(|b| b.get().target());
        let upstream = if let Some(upstream) = upstream {
            let (ahead, behind) = repo.graph_ahead_behind(local, upstream)?;
            Some(Upstream {
                ahead: ahead as u8,
                behind: behind as u8,
            })
        } else {
            None
        };
        Ok(State::OnBranch {
            name: String::from_utf8_lossy(branch.name_bytes()?).into_owned(),
            upstream,
        })
    } else {
        Ok(State::Empty)
    }
}

fn get_statuses(git: &Repository) -> Result<Statuses, git2::Error> {
    let mut staged = 0;
    let mut not_staged = 0;
    let mut untracked = 0;
    let mut conflicted = 0;

    let statuses = git.statuses(Some(
        git2::StatusOptions::new()
            .show(git2::StatusShow::IndexAndWorkdir)
            .include_untracked(true)
            .renames_from_rewrites(true)
            .renames_head_to_index(true),
    ))?;
    for entry in statuses.iter() {
        let status = entry.status();
        if status.contains(Status::INDEX_NEW)
            || status.contains(Status::INDEX_MODIFIED)
            || status.contains(Status::INDEX_TYPECHANGE)
            || status.contains(Status::INDEX_RENAMED)
            || status.contains(Status::INDEX_DELETED)
        {
            staged += 1;
        }
        if status.contains(Status::WT_MODIFIED)
            || status.contains(Status::WT_TYPECHANGE)
            || status.contains(Status::WT_DELETED)
        {
            not_staged += 1;
        }
        if status.contains(Status::WT_NEW) {
            untracked += 1;
        }
        if status.contains(Status::CONFLICTED) {
            conflicted += 1;
        }
    }
    Ok(Statuses {
        conflicted: Conflicted(conflicted),
        untracked: Untracked(untracked),
        not_staged: NotStaged(not_staged),
        staged: Staged(staged),
    })
}

impl Git {
    pub fn new() -> Self {
        let repo = match git2::Repository::discover(".") {
            Err(_) => return Self(None),
            Ok(repo) => repo,
        };

        let state = get_state(&repo);
        let statuses = get_statuses(&repo);
        Self(Some(GitInner { state, statuses }))
    }
}

impl Segment for Git {
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        let inner = match self.0 {
            None => return Ok(()),
            Some(ref mut inner) => inner,
        };
        let state = match inner.state {
            Ok(ref state) => state,
            Err(_) => {
                w.set_bg(Color::from_rgb(255, 0, 0))?;
                return write!(w, " ☠ ");
            }
        };
        match state {
            State::Detached(ref short_id) => {
                w.set_bg(Color::from_rgb(0, 0, 180))?;
                w.set_fg(Color::from_rgb(230, 230, 230))?;
                write!(w, " 📤 {} ", String::from_utf8_lossy(&*short_id))?;
            }
            &State::OnBranch { upstream, ref name } => {
                w.set_bg(Color::from_rgb(30, 180, 30))?;
                w.set_fg(Color::from_rgb(0, 0, 0))?;
                write!(w, " ⭠ {} ", name)?;
                if let Some(Upstream { ahead, behind }) = upstream {
                    if behind > 0 {
                        w.start_segment(Color::from_rgb(120, 30, 30))?;
                    } else if ahead > 0 {
                        w.start_segment(Color::from_rgb(120, 30, 120))?;
                    } else {
                        w.start_segment(Color::from_rgb(30, 30, 30))?;
                    }
                    w.set_fg(Color::from_rgb(230, 230, 230))?;
                    write!(w, " {}🔺 {}🔻", ahead, behind)?;
                }
            }
            State::Empty => {
                w.set_bg(Color::from_rgb(255, 255, 255))?;
                w.set_fg(Color::from_rgb(0, 0, 0))?;
                write!(w, " ∅  no commits ")?;
            }
        }
        match inner.statuses {
            Ok(ref mut statuses) => {
                statuses.write(w)?;
            }
            Err(_) => {
                w.set_bg(Color::from_rgb(255, 0, 0))?;
                write!(w, " ☠ ")?;
            }
        }
        Ok(())
    }
}

// '🏷' tag
// '🫂' merge
// write!(
//     w,
//     " {}✅  {}🖍  {}❓ {}💔 ",
//     statuses.staged, statuses.not_staged, statuses.untracked, statuses.conflicted
// )
