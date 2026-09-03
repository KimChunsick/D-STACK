// verbs/doctor/korules.rs
// doctor section 7: the size of the Korean rule table, by rule kind (R91).

use crate::core::context::Context;
use crate::core::error::Result;

pub fn section(ctx: &mut Context) -> Result<bool> {
    let path = ctx.home.home.join("lint/ko-rules.tsv");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => {
            say!(ctx, "ko-rules: missing {}", path.display());
            return Ok(false);
        }
    };
    let rows = |kind: &str| {
        text.lines()
            .filter(|line| {
                let column: Vec<&str> = line.split('\t').collect();
                !column[0].starts_with('#') && column.get(1).copied().unwrap_or("") == kind
            })
            .count()
    };
    say!(
        ctx,
        "ko-rules ({}): regex rows {} (checked by doctor --self), judgment rows {} (counted only)",
        path.display(),
        rows("regex"),
        rows("judgment")
    );
    Ok(true)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r91__the_rule_table_is_counted_by_kind() {
        let (held, printed) = super::super::tests::printed(section);
        assert!(held, "the rule table is missing:\n{printed}");
        let line = printed.lines().next().expect("the one line");
        assert!(line.starts_with("ko-rules ("), "unexpected line: {line}");
        assert!(
            line.contains(" (checked by doctor --self), judgment rows "),
            "unexpected line: {line}"
        );
        let regex_rows: usize = line
            .split("regex rows ")
            .nth(1)
            .and_then(|rest| rest.split(' ').next())
            .expect("the regex count")
            .parse()
            .expect("a number");
        assert!(regex_rows > 0, "the table carries regex rows");
    }
}
