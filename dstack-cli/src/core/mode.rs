// Independent main/sub providers, project defaults and immutable-by-default task snapshots.
use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::error::{Error, Result};
use crate::core::fsx::atomic_write;
use crate::core::paths::is_plain_name;
use crate::core::roots::Roots;
use crate::core::target::{Target, TargetKind};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Claude,
    Codex,
}

impl Provider {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            _ => Err(Error::failed(format!(
                "provider must be claude or codex (got '{value}')"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Mode {
    pub main: Provider,
    pub sub: Provider,
}

impl Default for Mode {
    fn default() -> Self {
        Self {
            main: Provider::Claude,
            sub: Provider::Codex,
        }
    }
}

impl Mode {
    pub fn project(roots: &Roots) -> Result<Self> {
        Ok(read(&roots.store.join("project/mode.json"))?.unwrap_or_default())
    }

    pub fn effective(roots: &Roots) -> Result<Self> {
        Ok(selected(roots, None)?.mode)
    }

    /// An old run or quick task keeps the historical pair, independent of project changes.
    pub fn for_run(_roots: &Roots, dir: &Path) -> Result<Self> {
        require_directory(dir)?;
        Ok(read(&dir.join("mode.json"))?.unwrap_or_default())
    }

    pub fn snapshot(&self, dir: &Path) -> Result<()> {
        require_directory(dir)?;
        self.write(&dir.join("mode.json"))
    }

    pub fn write_project(&self, roots: &Roots) -> Result<()> {
        roots.require_store()?;
        let path = roots.store.join("project/mode.json");
        read(&path)?;
        fs::create_dir_all(path.parent().expect("project directory"))
            .map_err(|error| cannot("create mode directory", &path, error))?;
        self.write(&path)
    }

    fn write(&self, path: &Path) -> Result<()> {
        // Validate the existing file even when replacing both values. A damaged snapshot is
        // not silently repaired by an unrelated adoption or settings command.
        read(path)?;
        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| Error::cannot_decide(format!("cannot encode mode: {error}")))?;
        bytes.push(b'\n');
        atomic_write(path, &bytes).map_err(|error| cannot("write", path, error))
    }
}

pub struct Selection {
    pub mode: Mode,
    pub source: &'static str,
    pub target: Option<Target>,
}

/// Shared read-only target selection for mode show and exec, including exec --dry-run.
/// Unlike resolve_target this never updates an owner heartbeat or creates metadata.
pub fn selected(roots: &Roots, selector: Option<(&str, &str)>) -> Result<Selection> {
    let current;
    let selector = match selector {
        Some(selector) => Some(selector),
        None => {
            current = roots.current_run_id()?;
            current.as_deref().map(|id| ("run", id))
        }
    };
    let Some((kind, id)) = selector else {
        let project = read(&roots.store.join("project/mode.json"))?;
        return Ok(Selection {
            mode: project.unwrap_or_default(),
            source: if project.is_some() {
                "project"
            } else {
                "default"
            },
            target: None,
        });
    };
    if !is_plain_name(id) {
        return Err(Error::failed(format!(
            "{kind} id must be a plain name (got '{id}')"
        )));
    }
    let (kind, dir) = match kind {
        "run" => (TargetKind::Run, roots.runs.join(id)),
        "quick" => (TargetKind::Quick, roots.quick.join(id)),
        _ => return Err(Error::failed("mode target must be run or quick")),
    };
    require_directory(&dir)?;
    let snapshot = read(&dir.join("mode.json"))?;
    let source = match (kind, snapshot.is_some()) {
        (TargetKind::Run, true) => "run",
        (TargetKind::Run, false) => "legacy-run",
        (TargetKind::Quick, true) => "quick",
        (TargetKind::Quick, false) => "legacy-quick",
    };
    Ok(Selection {
        mode: snapshot.unwrap_or_default(),
        source,
        target: Some(Target {
            kind,
            id: id.to_string(),
            dir,
        }),
    })
}

fn require_directory(dir: &Path) -> Result<()> {
    let metadata = fs::metadata(dir).map_err(|error| cannot("read mode target", dir, error))?;
    if !metadata.is_dir() {
        return Err(Error::cannot_decide(format!(
            "mode target is not a directory: {}",
            dir.display()
        )));
    }
    Ok(())
}

fn read(path: &Path) -> Result<Option<Mode>> {
    // symlink_metadata distinguishes a missing entry from an existing dangling symlink.
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(cannot("read", path, error)),
        Ok(_) => (),
    }
    let text = fs::read_to_string(path).map_err(|error| cannot("read", path, error))?;
    serde_json::from_str(&text).map(Some).map_err(|error| {
        Error::cannot_decide(format!("invalid mode file {}: {error}", path.display()))
    })
}

fn cannot(action: &str, path: &Path, error: std::io::Error) -> Error {
    Error::cannot_decide(format!("cannot {action} {}: {error}", path.display()))
}
