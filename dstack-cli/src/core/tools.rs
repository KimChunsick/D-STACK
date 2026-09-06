// core/tools.rs
// What the machine has and what the repository declares: deps.tsv, team style, PROJECT.md policy.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::mode::Mode;
use crate::core::roots::{git_out, Home};

pub const NO_TEAM_STYLE_LINE: &str =
    "No team style — in this repository existing code wins";

pub fn deps_file(home: &Home) -> PathBuf {
    match std::env::var("DSTACK_DEPS") {
        Ok(path) if !path.is_empty() => PathBuf::from(path),
        _ => home.repo.join("deps.tsv"),
    }
}

/// The two probe forms deps.tsv uses, read natively. The shell ran the probe through eval, which
/// would run arbitrary text; the port refuses everything but these two forms, so git stays the
/// only executable the CLI spawns (R01).
pub fn tool_present(probe: &str) -> Result<bool> {
    let probe = probe.trim();
    if let Some(name) = probe.strip_prefix("command -v ") {
        return Ok(on_path(name.trim()));
    }
    if let Some(path) = probe.strip_prefix("test -x ") {
        return Ok(is_executable(&probe_path(path.trim())));
    }
    Err(Error::cannot_decide(format!(
        "deps.tsv probe not supported: {probe} (use `command -v <name>` or `test -x <path>`)"
    )))
}

/// command -v <name>: the name resolves to an executable file, either directly when it carries a
/// slash or through one of the PATH entries.
fn on_path(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.contains('/') {
        return is_executable(Path::new(name));
    }
    let path = match std::env::var("PATH") {
        Ok(path) => path,
        Err(_) => return false,
    };
    path.split(':').any(|dir| {
        let dir = if dir.is_empty() { "." } else { dir };
        is_executable(&Path::new(dir).join(name))
    })
}

/// The argument of test -x: surrounding double quotes come off and a leading $HOME/ or ~/ becomes
/// the home directory.
fn probe_path(raw: &str) -> PathBuf {
    let raw = raw.trim_matches('"');
    let home = std::env::var("HOME").unwrap_or_default();
    for prefix in ["$HOME/", "~/"] {
        if let Some(rest) = raw.strip_prefix(prefix) {
            return PathBuf::from(&home).join(rest);
        }
    }
    PathBuf::from(raw)
}

fn is_executable(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(meta) => meta.is_file() && meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

/// tool_check(): the table and the return code of the shell — 0 all present, 1 a goal-closing
/// tool is missing, 2 there is no deps.tsv to check against. A probe form the port does not
/// support stops the check instead of reading as "missing".
pub fn tool_check(ctx: &mut Context, fields: &[String]) -> Result<i32> {
    let path = deps_file(&ctx.home);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => {
            ctx.out.say(&format!(
                "tools: deps.tsv missing at {} — cannot check (R13)",
                path.display()
            ));
            return Ok(2);
        }
    };
    let mut missing = 0;
    let mut checked = 0;
    let mut groups_needed: Vec<String> = Vec::new();
    let mut groups_met: Vec<String> = Vec::new();
    ctx.out.say("tools required by this request:");
    for line in text.lines() {
        let column: Vec<&str> = line.split('\t').collect();
        let name = column.first().copied().unwrap_or("");
        if name.is_empty() || name.starts_with('#') || name == "name" {
            continue;
        }
        let probe = column.get(1).copied().unwrap_or("");
        let install = column.get(2).copied().unwrap_or("");
        let when = column.get(5).copied().unwrap_or("");
        let reqby = column.get(6).copied().unwrap_or("");
        let group = column.get(7).copied().unwrap_or("");
        if when != "goal-closing" || !reqby_matches(reqby, fields) {
            continue;
        }
        checked += 1;
        if tool_present(probe)? {
            ctx.out.say(&format!("  ok      {name}"));
            if !group.is_empty() {
                groups_met.push(group.to_string());
            }
        } else if !group.is_empty() {
            groups_needed.push(group.to_string());
            ctx.out.say(&format!(
                "  missing {name} (alternative in group '{group}'; install: {install})"
            ));
        } else {
            missing += 1;
            ctx.out.say(&format!(
                "  MISSING {name} — install: {install} — or turn off: {reqby}"
            ));
        }
    }
    for group in &groups_needed {
        if !groups_met.contains(group) {
            missing += 1;
            ctx.out
                .say(&format!("  MISSING every tool in group '{group}'"));
        }
    }
    ctx.out.say(&format!(
        "  checked {checked} goal-closing tools, missing {missing}"
    ));
    Ok(if missing == 0 { 0 } else { 1 })
}

/// Provider requirements follow the selected main and the roles this target actually runs.
pub fn tool_check_for_mode(
    ctx: &mut Context, fields: &[String], mode: &Mode, need_sub: bool,
) -> Result<i32> {
    let mut selected = fields.to_vec();
    selected.push(format!("provider={}", mode.main));
    if need_sub && mode.sub != mode.main {
        selected.push(format!("provider={}", mode.sub));
    }
    tool_check(ctx, &selected)
}

/// required_by is a field expression: always | e2e=capture | visual=design,regression.
pub fn reqby_matches(expr: &str, fields: &[String]) -> bool {
    if expr == "always" {
        return true;
    }
    let key = expr.split('=').next().unwrap_or("");
    let values = match expr.find('=') {
        Some(at) => &expr[at + 1..],
        None => expr,
    };
    for field in fields {
        let (name, value) = match field.find('=') {
            Some(at) => (&field[..at], &field[at + 1..]),
            None => (field.as_str(), field.as_str()),
        };
        if name != key {
            continue;
        }
        if values
            .split(',')
            .any(|want| !want.is_empty() && want == value)
        {
            return true;
        }
    }
    false
}

/// The organisation of the origin remote, read the way the shell reads it.
pub fn org_of_remote(wt_root: &Path) -> Option<String> {
    let url = git_out(Some(wt_root), &["remote", "get-url", "origin"])?;
    let mut url = url.as_str();
    if let Some(stripped) = url.strip_suffix(".git") {
        url = stripped;
    }
    if let Some(stripped) = url.strip_suffix('/') {
        url = stripped;
    }
    let mut url = url.to_string();
    if let Some(at) = url.find(':') {
        if url[at..].contains('/') {
            url = url[at + 1..].to_string();
        }
    }
    if let Some(at) = url.find("://") {
        url = url[at + 3..].to_string();
    }
    if let Some(at) = url.find('/') {
        url = url[at + 1..].to_string();
    }
    let org = url.split('/').next().unwrap_or("").to_string();
    if org.is_empty() {
        None
    } else {
        Some(org)
    }
}

/// team_style_lookup() (R52): the worktree file, then PROJECT.md, then the organisation file.
pub fn team_style_lookup(ctx: &mut Context) -> Option<PathBuf> {
    let roots = ctx.roots().ok()?;
    let in_worktree = roots.wt_root.join(".claude/style/team.md");
    if in_worktree.is_file() {
        return Some(in_worktree);
    }
    let project = roots.store.join("project/PROJECT.md");
    if let Ok(text) = std::fs::read_to_string(&project) {
        if let Some(line) = text.lines().find(|line| line.starts_with("team_style:")) {
            let declared = line["team_style:".len()..].trim_start_matches([' ', '\t']);
            if !declared.is_empty() {
                let path = match declared.starts_with('/') {
                    true => PathBuf::from(declared),
                    false => roots.wt_root.join(declared),
                };
                if path.is_file() {
                    return Some(path);
                }
            }
        }
    }
    let org = org_of_remote(&roots.wt_root)?;
    let path = PathBuf::from(std::env::var("HOME").ok()?)
        .join(".claude/style")
        .join(format!("{org}.md"));
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

/// policy_get() (R35): one key of the "## Verification policy" block of PROJECT.md.
pub fn policy_get(store: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(store.join("project/PROJECT.md")).ok()?;
    let mut in_block = false;
    for line in text.lines() {
        if line.starts_with("## Verification policy") {
            in_block = true;
            continue;
        }
        if in_block && line.starts_with("## ") {
            in_block = false;
        }
        if !in_block {
            continue;
        }
        if let Some(at) = line.find(':') {
            if &line[..at] == key {
                return Some(line[at + 1..].trim_matches([' ', '\t']).to_string());
            }
        }
    }
    None
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__reqby_expressions() {
        let fields: Vec<String> = vec!["e2e=capture".to_string(), "review=on".to_string()];
        assert!(reqby_matches("always", &fields));
        assert!(reqby_matches("e2e=capture", &fields));
        assert!(reqby_matches("e2e=cli,capture", &fields));
        assert!(!reqby_matches("e2e=cli", &fields));
        assert!(!reqby_matches("visual=design", &fields));
    }

    #[test]
    fn r01__probe_command_v_finds_git() {
        assert!(tool_present("command -v git").expect("supported form"));
        assert!(tool_present("command -v /bin/sh").expect("supported form"));
    }

    #[test]
    fn r01__probe_missing_tool_is_false() {
        assert!(!tool_present("command -v dstack-no-such-tool-xyz").expect("supported form"));
        assert!(!tool_present("test -x /usr/bin/dstack-no-such-tool-xyz").expect("supported form"));
        assert!(
            !tool_present("command -v /etc").expect("supported form"),
            "a directory is not a tool"
        );
    }

    #[test]
    fn r01__probe_test_x_expands_home() {
        let home = PathBuf::from(std::env::var("HOME").expect("HOME"));
        assert_eq!(probe_path("/usr/bin/true"), PathBuf::from("/usr/bin/true"));
        assert!(tool_present("test -x \"/bin/sh\"").expect("supported form"));
        assert!(!tool_present("test -x \"$HOME/dstack-no-such-tool-xyz\"").expect("supported form"));
    }

    #[test]
    fn r01__probe_unknown_form_cannot_decide() {
        let refused = tool_present("ls | grep git").expect_err("no shell runs this");
        assert_eq!(refused.code(), 2);
        assert_eq!(
            refused.message(),
            "deps.tsv probe not supported: ls | grep git (use `command -v <name>` or `test -x <path>`)"
        );
    }
}
