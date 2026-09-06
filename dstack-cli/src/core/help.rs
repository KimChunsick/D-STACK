// core/help.rs
// dstack help: the roster rendered exactly as cmd_help prints it.

use crate::core::out::Out;
use crate::core::registry::ROSTER;

pub fn render(out: &mut Out) {
    out.say("dstack — machine state of the pipeline. Usage: dstack <noun> <verb> [args]");
    for (name, summary) in ROSTER.iter() {
        out.say(&format!("  {name:<22} {summary}"));
    }
    out.say(&format!("verbs: {}", ROSTER.len()));
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__columns_match_the_printf_format() {
        let mut out = Out::new();
        out.begin_capture();
        render(&mut out);
        let (stdout, _) = out.end_capture();
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(
            lines[0],
            "dstack — machine state of the pipeline. Usage: dstack <noun> <verb> [args]"
        );
        assert_eq!(lines[1], "  init                   bootstrap the .dstack store in this repository (never expands cases)");
        assert_eq!(lines[lines.len() - 1], "verbs: 70");
        assert_eq!(lines.len(), ROSTER.len() + 2);
    }
}
