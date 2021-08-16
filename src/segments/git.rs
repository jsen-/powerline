use crate::{Color, ColoredStream, Segment};
use git2::{Branch, BranchType, ErrorClass, ErrorCode, Repository, Status};
use std::io::Write as _;

pub struct Git {
    repo: Result<Repo, git2::Error>,
}

struct Upstream {
    ahead: u8,
    behind: u8,
}

enum RepoState {
    Detached(git2::Buf),
    OnBranch {
        name: String,
        upstream: Option<Upstream>,
    },
    Empty,
}

#[derive(Debug)]
struct Statuses {
    conflicted: u8,
    untracked: u8,
    not_staged: u8,
    staged: u8,
}

struct Repo {
    state: RepoState,
    repo: git2::Repository,
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

fn get_state(repo: &git2::Repository) -> Result<RepoState, git2::Error> {
    if repo.head_detached()? {
        let head = repo.head()?;
        let target = head.peel(git2::ObjectType::Any)?;
        let short_id = target.short_id()?;
        Ok(RepoState::Detached(short_id))
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
        Ok(RepoState::OnBranch {
            name: String::from_utf8_lossy(branch.name_bytes()?).into_owned(),
            upstream,
        })
    } else {
        Ok(RepoState::Empty)
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
        conflicted,
        untracked,
        not_staged,
        staged,
    })
}

impl Git {
    pub fn new() -> Option<Self> {
        let repo = match git2::Repository::discover(".") {
            Err(_) => return None,
            Ok(repo) => repo,
        };

        let repo = get_state(&repo).and_then(|state| Ok(Repo { state, repo }));
        Some(Git { repo })
    }
}

impl Segment for Git {
    fn bg(&mut self) -> Color {
        let repo = match self.repo {
            Ok(ref state) => state,
            Err(_) => return Color::from_rgb(255, 0, 0),
        };
        match repo.state {
            RepoState::Detached(_) => Color::from_rgb(0, 0, 180),
            RepoState::OnBranch { upstream: None, .. } => Color::from_rgb(180, 0, 0),
            RepoState::OnBranch {
                upstream: Some(_), ..
            } => Color::from_rgb(0, 180, 0),
            RepoState::Empty => Color::from_rgb(255, 255, 255),
        }
    }

    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        w.set_fg(Color::from_rgb(230, 230, 230))?;
        let repo = match self.repo {
            Ok(ref state) => state,
            Err(_) => return write!(w, " ☠ "),
        };
        match repo.state {
            RepoState::Detached(ref short_id) => {
                write!(w, " 📤 {} ", String::from_utf8_lossy(&*short_id))
            }
            RepoState::OnBranch {
                upstream: None,
                ref name,
            } => write!(w, " ⭠ {} ", name),
            RepoState::OnBranch {
                upstream: Some(Upstream { ahead, behind, .. }),
                ref name,
            } => write!(w, " ⭠ {} {}⬆/{}⬇ ", name, ahead, behind),
            RepoState::Empty => write!(w, " ∅  no commits "),
        }
    }
}

pub struct GitStatus {
    statuses: Result<Statuses, git2::Error>,
}

impl GitStatus {
    pub fn new(git: &Git) -> Option<Self> {
        match git.repo {
            Ok(ref repo) => Some(Self {
                statuses: get_statuses(&repo.repo),
            }),
            Err(_) => None,
        }
    }
}

// '🏷' tag
// '🫂' merge
impl Segment for GitStatus {
    fn bg(&mut self) -> Color {
        match self.statuses {
            Ok(ref state) => state,
            Err(_) => return Color::from_rgb(255, 0, 0),
        };
        Color::from_rgb(0, 0, 0)
    }

    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        let statuses = match self.statuses {
            Ok(ref state) => state,
            Err(_) => return write!(w, " ☠ "),
        };
        w.set_fg(Color::from_rgb(230, 230, 230))?;
        write!(
            w,
            " {}✅  {}🖍  {}❓ {}💔 ",
            statuses.staged, statuses.not_staged, statuses.untracked, statuses.conflicted
        )
    }
}
