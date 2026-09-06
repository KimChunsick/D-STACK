// Explicit ownership transfer after immutable-packet and current-state checks.
use super::{packet, snapshot};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, utc_now, with_lock};
use crate::core::mode::{Mode, Provider};
use crate::core::roots::Roots;
use crate::core::target::Target;
use std::fs;
use std::path::Path;

pub fn apply(
    ctx: &mut Context,
    roots: &Roots,
    target: &Target,
    dir: &Path,
    host: Provider,
    stopped: bool,
) -> Result<()> {
    if !stopped {
        return Err(Error::failed(
            "confirm the source session and every native worker are stopped with --source-stopped",
        ));
    }
    let _lock = with_lock(&roots.local)?;
    packet::regular_dir(&target.dir.join("handoffs"))?;
    let data = packet::load(dir)?;
    if data.to != host {
        return Err(Error::failed(format!(
            "handoff destination is {}; start that host",
            data.to
        )));
    }
    if ctx.session_id.trim().is_empty()
        || ctx.session_id == data.history.session
        || ctx.session_id == data.snapshot.owner_session
        || ctx.session_id.contains(['\n', '\r', '\t'])
    {
        return Err(Error::failed(
            "handoff resume requires a distinct nonempty destination session id",
        ));
    }
    for name in ["consumed", "resuming"] {
        if fs::symlink_metadata(dir.join(name)).is_ok() {
            return Err(Error::failed(format!(
                "handoff already {name}; inspect its receipt before any further takeover"
            )));
        }
    }
    packet::verify_history(&data)?;
    snapshot::verify(&data.snapshot, roots, target)?;
    snapshot::check_idle(roots, &data.snapshot)?;
    let meta_path = target.dir.join("meta.tsv");
    let old_meta = packet::read(&meta_path, 256 * 1024)?;
    let mode_path = target.dir.join("mode.json");
    let old_mode = optional(&mode_path)?;
    let current_path = roots.local.join("CURRENT");
    let old_current = optional(&current_path)?;
    let changes = [
        ("owner_session", ctx.session_id.clone()),
        ("owner_pid", ctx.parent_pid.to_string()),
        ("owner_ts", utc_now()),
        ("status", "open".into()),
        ("handoff_id", data.id.clone()),
        ("handoff_from_session", data.history.session.clone()),
        ("transcript_path", String::new()),
    ];
    let mut new_meta = old_meta
        .lines()
        .filter(|line| {
            !changes
                .iter()
                .any(|(key, _)| line.split('\t').next() == Some(key))
        })
        .map(|l| format!("{l}\n"))
        .collect::<String>();
    for (key, value) in &changes {
        new_meta.push_str(&format!("{key}\t{value}\n"));
    }
    let mode = Mode {
        main: data.to,
        sub: data.snapshot.mode.sub,
    };
    let new_mode = serde_json::to_vec_pretty(&mode).map_err(packet::io)?;
    // The journal makes interruption detectable. A failed write rolls back every prior byte;
    // a process crash leaves resuming in place and blocks another automatic takeover.
    packet::write_new(&dir.join("resuming"), &format!("{}\n", ctx.session_id))?;
    let applied = (|| {
        atomic_write(&mode_path, &new_mode).map_err(packet::io)?;
        atomic_write(&meta_path, new_meta.as_bytes()).map_err(packet::io)?;
        atomic_write(&current_path, format!("{}\n", target.id).as_bytes()).map_err(packet::io)?;
        packet::write_new(
            &dir.join("consumed"),
            &format!(
                "session={}\nhost={}\nat={}\n",
                ctx.session_id,
                host,
                utc_now()
            ),
        )
    })();
    if let Err(error) = applied {
        let rollback = (|| {
            restore(&mode_path, old_mode.as_deref())?;
            atomic_write(&meta_path, old_meta.as_bytes()).map_err(packet::io)?;
            restore(&current_path, old_current.as_deref())?;
            fs::remove_file(dir.join("resuming")).map_err(packet::io)
        })();
        return match rollback {
            Ok(()) => Err(error),
            Err(rollback) => Err(Error::cannot_decide(format!(
                "{error}; rollback incomplete: {rollback}; inspect {}",
                dir.display()
            ))),
        };
    }
    ctx.out.say(&format!(
        "handoff resumed: {} (run {}, main={}, sub={})",
        data.id, target.id, host, mode.sub
    ));
    ctx.out
        .say(&format!("read: {}", dir.join("RESUME.md").display()));
    Ok(())
}
fn optional(path: &Path) -> Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(packet::io(e)),
        Ok(_) => packet::read(path, 256 * 1024).map(Some),
    }
}
fn restore(path: &Path, value: Option<&str>) -> Result<()> {
    match value {
        Some(value) => atomic_write(path, value.as_bytes()).map_err(packet::io),
        None => match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(packet::io(e)),
        },
    }
}
