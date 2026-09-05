// verbs/request/new.rs
// dstack request new: the request file copied from the work_type template (R40, R41).

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::meta::meta_get;
use crate::core::target::resolve_target;
use crate::store::request::{field_default, req_enum, RequestDoc, REQ_FIELDS};

use super::{request_file, rowfile, take};

pub fn new(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut work_type, mut title) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        if let Some((value, eaten)) = take(&rest, i, "type")? {
            work_type = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "title")? {
            title = value;
            i += eaten;
        } else if rest[i].starts_with('-') {
            fail!("unknown option: {}", rest[i]);
        } else {
            fail!("unexpected argument: {}", rest[i]);
        }
    }
    let file = request_file(&target);
    if file.is_file() {
        fail!(
            "request.md already exists: {} (dstack request show, or req add to extend it)",
            file.display()
        );
    }
    let types = req_enum("work_type").join(" ");
    if work_type.is_empty() {
        work_type = meta_get(&target.dir, "work_type")?.unwrap_or_default();
    }
    if work_type.is_empty() {
        fail!("--type is required: one of {types}");
    }
    if !format!(" {types} ").contains(&format!(" {work_type} ")) {
        fail!("--type must be one of: {types} (got '{work_type}')");
    }
    let template = ctx
        .home
        .home
        .join(format!("templates/request/{work_type}.md"));
    if !template.is_file() {
        return Err(Error::cannot_decide(format!(
            "no template for work_type={work_type} at {}",
            template.display()
        )));
    }
    if title.is_empty() {
        let slug = meta_get(&target.dir, "slug")?.filter(|slug| !slug.is_empty());
        title = format!("요청서: {}", slug.as_deref().unwrap_or(&target.id));
    }
    let text = std::fs::read_to_string(&template)
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", template.display())))?;
    rowfile::write(&file, &fill(&text, &title, &work_type))?;

    let doc = RequestDoc::load(&file)?;
    say!(ctx, "request: {}", file.display());
    say!(ctx, "  from template: {}", template.display());
    say!(ctx, "  title: {title}");
    for key in REQ_FIELDS {
        let value = doc.field(key).unwrap_or_default();
        say!(ctx, "  {:<18} {value}", format!("{key}:"));
    }
    say!(
        ctx,
        "  fields {}, rows 0, lines {}",
        REQ_FIELDS.len(),
        doc.line_count()
    );
    ctx.out.say(
        "  next: dstack req add \"<line>\" --accept \"<criterion>\", then dstack request open",
    );
    Ok(())
}

/// The awk pass over the template: every known frontmatter key is rewritten from field_default
/// (the table is the authority, not the template, so a stale template cannot ship a value that
/// `check request` would reject) and the title heading takes the run's title.
fn fill(template: &str, title: &str, work_type: &str) -> String {
    let value = |key: &str| match key {
        "work_type" => work_type.to_string(),
        other => field_default(work_type, other).to_string(),
    };
    let mut out = String::new();
    let mut frontmatter = false;
    for (index, line) in rowfile::lines(template).iter().enumerate() {
        let rewritten = if index == 0 && *line == "---" {
            frontmatter = true;
            line.to_string()
        } else if frontmatter && *line == "---" {
            frontmatter = false;
            line.to_string()
        } else if frontmatter {
            match line.find(':') {
                Some(at) if REQ_FIELDS.contains(&&line[..at]) => {
                    format!("{}: {}", &line[..at], value(&line[..at]))
                }
                _ => line.to_string(),
            }
        } else if *line == "# {{TITLE}}" {
            format!("# {title}")
        } else {
            line.to_string()
        };
        out.push_str(&rewritten);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_the_template_fill_rewrites_the_frontmatter_and_the_title() {
        let template =
            "---\nwork_type: web-ui\nunit_tests: off\nkept: as is\n---\n# {{TITLE}}\n\nbody\n";
        assert_eq!(
            fill(template, "a title", "cli"),
            "---\nwork_type: cli\nunit_tests: on\nkept: as is\n---\n# a title\n\nbody\n"
        );
    }

    #[test]
    fn r13_a_template_without_frontmatter_only_takes_the_title() {
        assert_eq!(fill("# {{TITLE}}\nx: y\n", "t", "cli"), "# t\nx: y\n");
        assert_eq!(fill("", "t", "cli"), "");
    }
}
