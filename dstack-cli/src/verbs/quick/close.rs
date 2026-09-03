// verbs/quick/close.rs
// dstack quick close: the report decides, then the state row is stamped (R79, R99).

use crate::core::args::{is_option, opt};
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::fsx::utc_now;

use super::{require_dir, state};

quick_verb!(QuickClose, "quick close", close);

fn close(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (mut slug, mut why) = (String::new(), String::new());
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].clone();
        if let Some((value, eaten)) = opt(&arg, args.get(i + 1).map(String::as_str), "abandon")? {
            why = value;
            i += eaten;
            continue;
        }
        i += 1;
        match arg.as_str() {
            _ if is_option(&arg) => return Err(crate::core::args::unknown_option(&arg)),
            _ if slug.is_empty() => slug = arg,
            _ => fail!("unexpected argument: {arg}"),
        }
    }
    require_dir(&roots.quick, &slug, "close")?;
    let when = utc_now();
    let status = match why.is_empty() {
        false => "abandoned",
        // The report is the gate, and its own exit code decides — a quick task closes on the
        // same evidence rule as a Goal (R79), never on "it looked done". The shell captured the
        // subprocess with 2>&1 and printed the whole of it on stdout, refusal line included.
        true => {
            let called = ctx.call("report", &["--quick".to_string(), slug.clone()]);
            let merged = format!("{}{}", called.stdout, called.stderr);
            let printed = merged.trim_end_matches('\n').to_string();
            ctx.out.say(&printed);
            if called.code != 0 {
                fail!(
                    "dstack report --quick {slug} exited {} — quick task '{slug}' stays open",
                    called.code
                )
            }
            "done"
        }
    };
    state::ensure(&roots.quick)?;
    state::close(&roots.quick, &slug, status, &when)?;
    say!(ctx, "quick {slug}: {status} at {when}");
    if !why.is_empty() {
        say!(ctx, "  abandoned because: {why}");
    }
    say!(ctx, "  state: {}", roots.quick.join("STATE.md").display());
    say!(
        ctx,
        "  open quick tasks left: {}",
        state::open_slugs(&roots.quick)?.len()
    );
    Ok(())
}
