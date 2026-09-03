// core/error.rs
// The failure modes of every verb: Failed exits 1, CannotDecide exits 2, Exit carries a code.

use std::fmt;

/// The exit-code contract of design.md: 0 pass, 1 a checked condition failed (fail() in
/// common.sh), 2 the command cannot decide (die() — the hooks block on this code). Exit(n) is
/// the shell's plain `exit n`: end the process with that code and print nothing, because the
/// reason is already on stdout (a refusal) or belongs to a child process (exec).
#[derive(Debug)]
pub enum Error {
    Failed(String),
    CannotDecide(String),
    Exit(i32),
}

impl Error {
    pub fn failed<S: Into<String>>(message: S) -> Error {
        Error::Failed(message.into())
    }

    pub fn cannot_decide<S: Into<String>>(message: S) -> Error {
        Error::CannotDecide(message.into())
    }

    pub fn message(&self) -> &str {
        match self {
            Error::Failed(m) | Error::CannotDecide(m) => m,
            Error::Exit(_) => "",
        }
    }

    pub fn code(&self) -> i32 {
        match self {
            Error::Failed(_) => 1,
            Error::CannotDecide(_) => 2,
            Error::Exit(code) => *code,
        }
    }
}

/// The one line fail() and die() print on stderr; main and Context::call are the only printers.
/// Exit renders as nothing at all, so a caller that prints this line prints an empty stream.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Exit(_) => Ok(()),
            _ => write!(f, "dstack: {}", self.message()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__exit_codes_and_prefix() {
        assert_eq!(Error::failed("boom").code(), 1);
        assert_eq!(Error::cannot_decide("boom").code(), 2);
        assert_eq!(Error::failed("boom").to_string(), "dstack: boom");
    }

    #[test]
    fn r13_exit_variant_prints_nothing() {
        assert_eq!(Error::Exit(3).code(), 3);
        assert_eq!(Error::Exit(3).message(), "");
        assert_eq!(Error::Exit(3).to_string(), "");
    }
}
