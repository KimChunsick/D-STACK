// verbs/issue/run.rs
// How one filing of a fixture ended, and what it printed on the way (R06).

use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

/// What one filing did: how it ended, and the last line it printed — which is the only thing
/// that says why when the ending alone cannot be read.
pub(super) struct Run {
    /// The exit code, or None when no code was returned: only a code decides a verdict.
    pub code: Option<i32>,
    /// How the child ended, in the words the message uses.
    pub ended: String,
    pub said: String,
}

impl Run {
    /// A status carries either an exit code or the signal that ended the child, and a child a
    /// signal ended returned no code at all. Standing an invented 2 in for it — the exit code of a
    /// verb that decided it could not decide — would report a decision on the one row a reader has
    /// to diagnose the crash from, so a signal is named as one. exec.rs reads the same accessor to
    /// give a killed command the shell's 128 + signal.
    pub(super) fn from_status(status: ExitStatus, said: String) -> Run {
        let ended = match (status.code(), status.signal()) {
            (Some(code), _) => format!("exited {code}"),
            (None, Some(signal)) => format!("was terminated by signal {signal}"),
            (None, None) => "ended with neither an exit code nor a signal".to_string(),
        };
        Run {
            code: status.code(),
            ended,
            said,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    /// A child a signal ended returned no exit code at all, and the row a reader diagnoses the
    /// crash from must not be given one: `status.code().unwrap_or(2)` reports an exit 2 the child
    /// never made, which reads as the verb having decided it could not decide.
    ///
    /// The child is the port's own binary, named by its path: R01 holds the crate to spawning git
    /// alone, and r01__spawns_only_git reads every spawn whose program is a quoted name under
    /// src/, #[cfg(test)] modules included. `hook <event>` waits for its payload on stdin, so a
    /// child whose stdin stays open and empty is still there to be killed.
    #[test]
    fn r06__a_child_a_signal_ended_is_not_an_exit_code() {
        let exe = crate::core::roots::Home::resolve()
            .expect("the repository of this test binary")
            .repo
            .join("dstack-cli/target/debug/dstack");
        let mut child = std::process::Command::new(&exe)
            .args(["hook", "stop"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("a child to kill at {}: {e}", exe.display()));
        child.kill().expect("kill the child");
        let status = child.wait().expect("the status of the killed child");
        assert_eq!(status.code(), None, "a signal leaves no exit code");
        let run = Run::from_status(status, String::new());
        assert_eq!(run.code, None, "no code is invented for it");
        assert_eq!(run.ended, "was terminated by signal 9");
    }
}
