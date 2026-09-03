// verbs/quick/new.rs
// dstack quick new: the quick task and the request file its defaults write (R99, R105).

use std::path::Path;

use crate::core::args::{is_option, opt, unknown_option};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::read_text;
use crate::core::paths::valid_slug;
use crate::core::tools::tool_check;
use crate::store::request::{field_default, req_enum};

use super::state;

quick_verb!(QuickNew, "quick new", new);

fn new(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (mut slug, mut work_type) = (String::new(), "cli".to_string());
    let (mut discuss, mut research, mut review, mut validate) = (false, false, false, false);
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].clone();
        if let Some((value, eaten)) = opt(&arg, args.get(i + 1).map(String::as_str), "type")? {
            work_type = value;
            i += eaten;
            continue;
        }
        i += 1;
        match arg.as_str() {
            "--discuss" => discuss = true,
            "--research" => research = true,
            "--review" => review = true,
            "--validate" => validate = true,
            "--full" => {
                research = true;
                review = true;
                validate = true;
            }
            _ if is_option(&arg) => return Err(unknown_option(&arg)),
            _ if slug.is_empty() => slug = arg,
            _ => fail!("unexpected argument: {arg}"),
        }
    }
    if slug.is_empty() {
        fail!("usage: dstack quick new <slug> [--type T] [--discuss] [--research] [--review] [--validate] [--full]")
    }
    if !valid_slug(&slug) {
        fail!("slug must match [a-z0-9][a-z0-9-]* (got '{slug}')")
    }
    let types = req_enum("work_type");
    if !types.contains(&work_type.as_str()) {
        fail!("--type must be one of: {}", types.join(" "))
    }
    let dir = roots.quick.join(&slug);
    if dir.is_dir() {
        fail!(
            "quick task exists: {} (dstack quick status {slug})",
            dir.display()
        )
    }

    // Quick defaults (R99): everything that costs a model round trip is off unless a flag turns
    // it on. `--validate` restores the work_type's own e2e value rather than a fixed one, because
    // what "verified" means is a property of the work type, not of the track.
    let research_field = if research { "one-pass" } else { "none" };
    let review_field = if review { "on" } else { "off" };
    let e2e_field = match validate {
        true => field_default(&work_type, "e2e"),
        false => "none",
    };

    // R105: refuse before creating anything, so a missing tool leaves no half-open task behind.
    let fields = [
        format!("e2e={e2e_field}"),
        format!("review={review_field}"),
        "visual=none".to_string(),
        "unit_tests=off".to_string(),
    ];
    if tool_check(ctx, &fields)? != 0 {
        ctx.out
            .say("refused: a goal-closing tool is missing for this quick task (see lines above)");
        return Err(Error::Exit(1));
    }

    // Every read this verb needs happens before the first write: the work_type template and the
    // state table a cannot-decide could stop it on. A half-created directory would be refused by
    // the next attempt as an existing quick task, so a failed read has to leave nothing behind.
    let body = template_body(ctx, &work_type, &slug)?;
    state::readable(&roots.quick)?;
    let text = request_text(
        &slug,
        &work_type,
        research_field,
        review_field,
        e2e_field,
        &body,
    );

    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::cannot_decide(format!("cannot create {}: {e}", dir.display())))?;
    let request = dir.join("request.md");
    std::fs::write(&request, text)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", request.display())))?;
    state::ensure(&roots.quick)?;
    state::add(&roots.quick, &slug)?;

    say!(ctx, "quick task: {slug}");
    say!(ctx, "  dir:     {}", dir.display());
    say!(ctx, "  request: {}", request.display());
    say!(ctx, "  state:   {}", roots.quick.join("STATE.md").display());
    say!(ctx, "  fields:  work_type={work_type} route=quick external_research={research_field} risk_axes=none design_review=skip");
    say!(ctx, "           review={review_field} codex_effort=medium e2e={e2e_field} unit_tests=off visual=none korean_polish=on");
    say!(
        ctx,
        "  CURRENT untouched: {}",
        roots
            .current_run_id()?
            .unwrap_or_else(|| "(none)".to_string())
    );
    if discuss {
        ctx.out.say(
            "  --discuss: run one interview round before approving (the frontmatter is unchanged)",
        );
    }
    say!(
        ctx,
        "  next: dstack req add \"<line>\" --accept \"<criterion>\" --quick {slug}"
    );
    Ok(())
}

/// The work_type template without its own frontmatter and with the title filled the way
/// `request new` fills it (R40), so the two entry points produce the same body. The trailing
/// newlines the shell's `$(…)` drops are dropped here too.
fn template_body(ctx: &Context, work_type: &str, slug: &str) -> Result<String> {
    let path = ctx
        .home
        .home
        .join(format!("templates/request/{work_type}.md"));
    if !path.is_file() {
        return Ok(String::new());
    }
    Ok(strip_frontmatter(&read(&path)?, slug))
}

fn strip_frontmatter(template: &str, title: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    let mut done = false;
    for (index, line) in template.lines().enumerate() {
        if index == 0 && line == "---" {
            inside = true;
            continue;
        }
        if inside && !done && line == "---" {
            inside = false;
            done = true;
            continue;
        }
        if inside {
            continue;
        }
        match line == "# {{TITLE}}" {
            true => out.push_str(&format!("# {title}\n")),
            false => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    out.trim_end_matches('\n').to_string()
}

/// The frontmatter is always written here: the template carries Goal defaults, and a quick task
/// that inherited them would quietly re-enable the stages R99 exists to skip.
fn request_text(
    slug: &str,
    work_type: &str,
    research: &str,
    review: &str,
    e2e: &str,
    body: &str,
) -> String {
    let mut text = String::from("---\n");
    text.push_str(&format!("work_type: {work_type}\n"));
    text.push_str("route: quick\n");
    text.push_str(&format!("external_research: {research}\n"));
    text.push_str("risk_axes: none\n");
    text.push_str("design_review: skip\n");
    text.push_str(&format!("review: {review}\n"));
    text.push_str("codex_effort: medium\n");
    text.push_str(&format!("e2e: {e2e}\n"));
    text.push_str("unit_tests: off\n");
    text.push_str("visual: none\n");
    text.push_str("korean_polish: on\n");
    text.push_str("---\n");
    if !body.is_empty() {
        text.push_str(body);
        text.push('\n');
        return text;
    }
    text.push_str(&format!("# {slug}\n\n"));
    text.push_str("Quick task (R99). Minimum: one R row with an accept criterion, one task, one\n");
    text.push_str("evidence row, one report. Add rows with:\n\n");
    text.push_str(&format!(
        "    dstack req add \"<one line>\" --accept \"<observable criterion>\" --quick {slug}\n"
    ));
    text
}

fn read(path: &Path) -> Result<String> {
    Ok(read_text(path)?.unwrap_or_default())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__the_template_loses_its_frontmatter_and_takes_the_title() {
        let template = "---\nwork_type: cli\n---\n# {{TITLE}}\n\nbody\n\n\n";
        assert_eq!(strip_frontmatter(template, "tidy"), "# tidy\n\nbody");
        // A second `---` further down is prose, not a frontmatter fence.
        assert_eq!(strip_frontmatter("---\na: b\n---\n---\n", "t"), "---");
        assert_eq!(strip_frontmatter("no frontmatter\n", "t"), "no frontmatter");
        assert_eq!(strip_frontmatter("", "t"), "");
    }

    #[test]
    fn r13__a_template_with_nothing_but_frontmatter_falls_back_to_the_minimum() {
        let written = request_text("tidy", "cli", "none", "off", "none", "");
        assert!(written.ends_with(
            "    dstack req add \"<one line>\" --accept \"<observable criterion>\" --quick tidy\n"
        ));
        assert!(written.contains("\nroute: quick\n"));
        let with_body = request_text("tidy", "cli", "one-pass", "on", "cli", "# tidy\n\nbody");
        assert!(with_body.ends_with("---\n# tidy\n\nbody\n"));
        assert!(with_body.contains("\nexternal_research: one-pass\n"));
    }
}
