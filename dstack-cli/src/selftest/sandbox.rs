// selftest/sandbox.rs
// The scratch repository of the shell reference's selftest.sh: its directory, git and dstack.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::context::Context;
use crate::core::error::{Error, Result};

/// A sandbox never depends on codex or ego-browser being installed on the machine.
const DEPS_TABLE: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n\
jq\tcommand -v jq\t-\t-\tno\tgoal-closing\talways\t\n";

static NEXT: AtomicU32 = AtomicU32::new(0);

/// A scratch git repository with the request, plan and artifact writers the checkers share.
/// The directory goes away when the sandbox is dropped, on every path out.
pub struct Sandbox {
    pub dir: PathBuf,
}

/// Only scratch() builds a Sandbox, and it only returns a directory this process created, so the
/// cleanup can never delete a path that was already there.
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// The scratch directory is created with fs::create_dir, which is exclusive: an existing path —
/// a directory, a file, or a symlink someone planted on shared temp storage — makes it fail
/// instead of being reused or followed. The name carries a nonce so a lost race just retries.
fn create_scratch_dir() -> Result<PathBuf> {
    let base = std::env::temp_dir();
    let mut last = String::new();
    for _ in 0..16 {
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since| since.subsec_nanos())
            .unwrap_or(0);
        let dir = base.join(format!(
            "dstack-selftest.{}-{}-{}",
            std::process::id(),
            unique,
            nonce
        ));
        match fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                last = format!("{} is taken", dir.display());
            }
            Err(e) => {
                return Err(Error::cannot_decide(format!(
                    "sandbox: cannot create {}: {e}",
                    dir.display()
                )))
            }
        }
    }
    Err(Error::cannot_decide(format!(
        "sandbox: no free scratch directory under {} ({last})",
        base.display()
    )))
}

impl Sandbox {
    /// The scratch repository and its deps table — everything selftest_sandbox does before it
    /// calls dstack init.
    pub fn scratch() -> Result<Sandbox> {
        let sandbox = Sandbox {
            dir: create_scratch_dir()?,
        };
        sandbox.write(&sandbox.dir.join(".deps.tsv"), DEPS_TABLE)?;
        sandbox.git(&["init", "-q"])?;
        sandbox.git(&[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            // The shell sandbox inherits the machine's signing setup; a fixture must not, or a
            // global commit.gpgsign turns every sandbox into a signing round that can fail.
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "init",
        ])?;
        Ok(sandbox)
    }

    /// selftest_sandbox(): the scratch repository with a store and one open run.
    pub fn new(ctx: &Context) -> Result<Sandbox> {
        let sandbox = Sandbox::scratch()?;
        sandbox.expect_ok(ctx, &["init"])?;
        sandbox.expect_ok(ctx, &["run", "new", "sandbox", "--type", "cli"])?;
        Ok(sandbox)
    }

    /// dsx(): dstack as a subprocess with cwd in the sandbox, stdout and stderr merged as 2>&1
    /// merges them — the real root resolution is exercised, so DSTACK_ROOT is never set.
    pub fn dsx(&self, ctx: &Context, args: &[&str]) -> Result<(i32, String)> {
        let log = self.dir.join(".dsx.out");
        let file =
            fs::File::create(&log).map_err(|e| Error::cannot_decide(format!("sandbox: {e}")))?;
        let merged = file
            .try_clone()
            .map_err(|e| Error::cannot_decide(format!("sandbox: {e}")))?;
        let status = Command::new(&ctx.self_exe)
            .args(args)
            .current_dir(&self.dir)
            .env("DSTACK_DEPS", self.dir.join(".deps.tsv"))
            .env_remove("DSTACK_ROOT")
            .stdin(Stdio::null())
            .stdout(Stdio::from(file))
            .stderr(Stdio::from(merged))
            .status()
            .map_err(|e| {
                Error::cannot_decide(format!(
                    "sandbox: cannot run {}: {e}",
                    ctx.self_exe.display()
                ))
            })?;
        let output = fs::read_to_string(&log).unwrap_or_default();
        let _ = fs::remove_file(&log);
        Ok((status.code().unwrap_or(2), output))
    }

    /// The current run directory of the sandbox (sandbox_run_dir).
    pub fn run_dir(&self) -> Result<PathBuf> {
        let current = self.dir.join(".dstack/local/CURRENT");
        let id = fs::read_to_string(&current).map_err(|e| {
            Error::cannot_decide(format!(
                "sandbox: no CURRENT in {}: {e}",
                self.dir.display()
            ))
        })?;
        Ok(self
            .dir
            .join(".dstack/runs")
            .join(id.trim_end_matches('\n')))
    }

    pub(super) fn write(&self, path: &Path, text: &str) -> Result<()> {
        fs::write(path, text).map_err(|e| {
            Error::cannot_decide(format!("sandbox: cannot write {}: {e}", path.display()))
        })
    }

    /// git with its own stderr in the error: a sandbox that cannot be built has to say why.
    fn git(&self, args: &[&str]) -> Result<()> {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.dir)
            .output()
            .map_err(|e| Error::cannot_decide(format!("sandbox: cannot run git: {e}")))?;
        if out.status.success() {
            return Ok(());
        }
        Err(Error::cannot_decide(format!(
            "sandbox: git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }

    fn expect_ok(&self, ctx: &Context, args: &[&str]) -> Result<()> {
        let (code, output) = self.dsx(ctx, args)?;
        if code == 0 {
            return Ok(());
        }
        Err(Error::cannot_decide(format!(
            "sandbox: dstack {} exited {code}: {output}",
            args.join(" ")
        )))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::core::registry::Registry;
    use crate::core::roots::Home;
    use std::rc::Rc;

    /// A Context whose self_exe is the debug binary cargo just built.
    fn context() -> Context {
        let home = Home::resolve().expect("repository");
        let exe = home.repo.join("dstack-cli/target/debug/dstack");
        Context::new(home, exe, Rc::new(Registry::new(crate::verbs::all_verbs())))
    }

    #[test]
    fn r05__scratch_is_a_git_repository_with_the_deps_table() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        assert!(sandbox.dir.join(".git").is_dir());
        let deps = std::fs::read_to_string(sandbox.dir.join(".deps.tsv")).expect("deps table");
        assert_eq!(deps.lines().count(), 3);
        assert!(deps
            .lines()
            .next()
            .expect("header")
            .starts_with("name\tprobe\tinstall"));
    }

    #[test]
    fn r05__scratch_never_reuses_an_existing_path() {
        let first = Sandbox::scratch().expect("first");
        let second = Sandbox::scratch().expect("second");
        assert_ne!(
            first.dir, second.dir,
            "two sandboxes never share a directory"
        );
        assert!(first.dir.is_dir() && second.dir.is_dir());
        // What the exclusive create buys: a planted symlink is refused, never followed.
        let planted =
            std::env::temp_dir().join(format!("dstack-selftest-planted-{}", std::process::id()));
        let _ = fs::remove_file(&planted);
        std::os::unix::fs::symlink(&second.dir, &planted).expect("plant a symlink");
        let refused = fs::create_dir(&planted).expect_err("an existing path is not created over");
        assert_eq!(refused.kind(), std::io::ErrorKind::AlreadyExists);
        fs::remove_file(&planted).expect("clean up");
    }

    #[test]
    fn r05__the_sandbox_directory_goes_away_with_the_guard() {
        let dir = {
            let sandbox = Sandbox::scratch().expect("scratch repository");
            sandbox.dir.clone()
        };
        assert!(!dir.exists());
    }

    // init and run new are ported in P5; until then the subprocess path cannot succeed.
    #[test]
    #[ignore = "needs dstack init and run new (P5)"]
    fn r05__sandbox_end_to_end() {
        let ctx = context();
        let sandbox = Sandbox::new(&ctx).expect("sandbox");
        let (code, output) = sandbox.dsx(&ctx, &["status"]).expect("status");
        assert_eq!(code, 0, "{output}");
    }
}
