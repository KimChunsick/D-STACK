// core/args.rs
// The option loop of the shell verbs: --name value, --name=value, and the unknown-option error.

use crate::core::error::{Error, Result};

/// Matches `--name value` or `--name=value` at one position and says how many arguments it ate.
/// `--name` with nothing behind it is the shell's `shift 2` on a single argument: under `set -e`
/// the command ends right there with 1 and prints nothing, so the operand is never invented.
pub fn opt(arg: &str, next: Option<&str>, name: &str) -> Result<Option<(String, usize)>> {
    if arg == format!("--{name}") {
        let value = next.ok_or(Error::Exit(1))?;
        return Ok(Some((value.to_string(), 2)));
    }
    Ok(arg
        .strip_prefix(&format!("--{name}="))
        .map(|value| (value.to_string(), 1)))
}

/// The shell's `-*)` arm: anything starting with a dash that no option arm claimed.
pub fn is_option(arg: &str) -> bool {
    arg.starts_with('-')
}

pub fn unknown_option(arg: &str) -> Error {
    Error::failed(format!("unknown option: {arg}"))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__both_option_forms_parse() {
        let parsed = |arg, next| opt(arg, next, "run").expect("parses");
        assert_eq!(parsed("--run", Some("R1")), Some(("R1".to_string(), 2)));
        assert_eq!(parsed("--run=R1", None), Some(("R1".to_string(), 1)));
        assert_eq!(parsed("--quick", Some("slug")), None);
        assert!(is_option("-x"));
        assert!(!is_option("plain"));
        assert_eq!(
            unknown_option("-x").to_string(),
            "dstack: unknown option: -x"
        );
    }

    /// The shell's `shift 2` with nothing left to shift: under `set -e` the command ends there
    /// with 1 and prints nothing, so a missing operand is Error::Exit(1), never an empty value.
    #[test]
    fn r11__a_missing_operand_is_the_silent_exit() {
        let error = opt("--run", None, "run").expect_err("the shell exits here");
        assert_eq!(error.code(), 1);
        assert_eq!(error.to_string(), "");
        // `--run=` keeps the empty string the shell's ${1#--run=} produces.
        assert_eq!(
            opt("--run=", None, "run").expect("parses"),
            Some((String::new(), 1))
        );
    }
}
