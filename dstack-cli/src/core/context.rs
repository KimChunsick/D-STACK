// core/context.rs
// The Context every verb gets: home, lazy roots, the output sink and the in-process self-call.

use std::path::PathBuf;
use std::rc::Rc;

use crate::core::error::{Error, Result};
use crate::core::out::Out;
use crate::core::registry::Registry;
use crate::core::roots::{Home, Roots};

/// What a self-call returned: the shell read these three from a subprocess.
pub struct Called {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct Context {
    pub home: Home,
    pub session_id: String,
    pub parent_pid: u32,
    pub self_exe: PathBuf,
    pub out: Out,
    registry: Rc<Registry>,
    roots: Option<Roots>,
}

impl Context {
    pub fn new(home: Home, self_exe: PathBuf, registry: Rc<Registry>) -> Context {
        Context {
            home,
            session_id: ["DSTACK_SESSION_ID", "CLAUDE_CODE_SESSION_ID", "CODEX_THREAD_ID", "CODEX_SESSION_ID"]
                .iter()
                .find_map(|key| std::env::var(key).ok().filter(|value| !value.trim().is_empty()))
                .unwrap_or_default(),
            parent_pid: std::os::unix::process::parent_id(),
            self_exe,
            out: Out::new(),
            registry,
            roots: None,
        }
    }

    /// resolve_roots() once per process; the roots are a handful of paths, so callers own a copy
    /// and stay free to print while they hold it.
    pub fn roots(&mut self) -> Result<Roots> {
        if self.roots.is_none() {
            self.roots = Some(Roots::resolve()?);
        }
        Ok(self.roots.clone().expect("just resolved"))
    }

    /// `[ -n "${WT_ROOT:-}" ]`: the roots this process already resolved, and no resolution of its
    /// own. A verb that also runs outside a repository asks this before it spawns its own git.
    pub fn resolved_roots(&self) -> Option<Roots> {
        self.roots.clone()
    }

    /// Roots the test decides instead of the ones the environment resolves.
    #[cfg(test)]
    pub fn set_roots(&mut self, roots: Roots) {
        self.roots = Some(roots);
    }

    /// Whether some handler answers this roster entry (the doctor's verb sweep asks).
    pub fn handled(&self, name: &str) -> bool {
        self.registry.has_handler(name)
    }

    /// Where the shell spawned "$DSTACK_SELF", the port calls the handler in process and captures
    /// what it printed. A refusal inside the call is the captured "dstack: ..." line and its code.
    pub fn call(&mut self, name: &str, args: &[String]) -> Called {
        let registry = Rc::clone(&self.registry);
        self.out.begin_capture();
        let result = registry.invoke(self, name, args);
        let (stdout, mut stderr) = self.out.end_capture();
        match result {
            Ok(()) => Called {
                code: 0,
                stdout,
                stderr,
            },
            // Error::Exit prints nothing, so the caller reads a bare code, as it read the exit
            // status of the subprocess the shell spawned here.
            Err(error) => {
                if !matches!(error, Error::Exit(_)) {
                    stderr.push_str(&format!("{error}\n"));
                }
                Called {
                    code: error.code(),
                    stdout,
                    stderr,
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::core::error::Error;
    use crate::core::verb::Verb;

    /// A verb that ends the process the way `exec` does: its own output, then a bare exit code.
    struct Quits;
    impl Verb for Quits {
        fn name(&self) -> &'static str {
            "status"
        }
        fn run(&self, ctx: &mut Context, _args: &[String]) -> Result<()> {
            ctx.out.say("what it printed before it quit");
            Err(Error::Exit(7))
        }
    }

    #[test]
    fn r13_call_returns_the_exit_code_silently() {
        let registry = Rc::new(Registry::new(vec![Box::new(Quits) as Box<dyn Verb>]));
        let home = Home::resolve().expect("the repository of this test binary");
        let mut ctx = Context::new(home, PathBuf::new(), Rc::clone(&registry));
        let called = ctx.call("status", &[]);
        assert_eq!(called.code, 7);
        assert_eq!(called.stdout, "what it printed before it quit\n");
        assert_eq!(called.stderr, "");
    }
}
