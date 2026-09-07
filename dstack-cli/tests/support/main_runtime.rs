#![allow(dead_code)]
use dstack_cli::core::{context::Context, registry::Registry, roots::Home};
use dstack_cli::selftest::Verdict;
use std::{fs, path::PathBuf, rc::Rc};

pub fn repo() -> PathBuf {
    fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")).unwrap()
}

pub fn context() -> Context {
    Context::new(
        Home {
            home: repo().join("claude"),
            repo: repo(),
        },
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(dstack_cli::verbs::all_verbs())),
    )
}

pub fn fixture(name: &str, wanted: Verdict, diagnostic: &str) {
    let checkers = dstack_cli::verbs::doctor::selftests();
    let checker = checkers
        .iter()
        .find(|c| c.checker() == "main-runtime")
        .expect("main-runtime must be registered in doctor --self");
    let mut ctx = context();
    ctx.out.begin_capture();
    let path = repo().join("claude/lint/fixtures/main-runtime").join(name);
    let verdict = checker.run(&mut ctx, &path).expect("checker decides");
    let (out, err) = ctx.out.end_capture();
    assert_eq!(verdict, wanted, "{name}: {out}{err}");
    if wanted == Verdict::Reject {
        assert!(
            out.contains(diagnostic),
            "{name} must diagnose {diagnostic}: {out}"
        );
        assert!(
            out.contains(".md:"),
            "diagnostic needs document and line: {out}"
        );
    }
}
