use crate::support::{Repo, ROW};
use std::fs;

#[test]
fn R21__large_renamed_range_fits_with_complete_patch_provenance_and_physical_paths() {
    let r = Repo::new();
    let text: String = (0..7000)
        .map(|line| format!("line {line:05}: original content retained through the move\n"))
        .collect();
    r.s.write("allowed/large.txt", &text);
    let mut binary: Vec<u8> = (0..16000).map(|byte| (byte % 251) as u8).collect();
    fs::write(r.s.0.join("allowed/binary.dat"), &binary).unwrap();
    r.s.write("allowed/run.sh", "#!/bin/sh\nprintf 'fixture\\n'\n");
    let base = r.commit();

    r.git(&["mv", "allowed/large.txt", "allowed/moved text.txt"]);
    r.s.write("allowed/moved text.txt", &format!("{text}TEXT_TAIL\n"));
    r.git(&["mv", "allowed/binary.dat", "allowed/moved.dat"]);
    let first = r.commit();
    binary[8000] = 255;
    fs::write(r.s.0.join("allowed/moved.dat"), &binary).unwrap();
    r.git(&["update-index", "--chmod=+x", "allowed/run.sh"]);
    // Preserve the index mode even on a filesystem that does not expose executable bits.
    r.git(&["config", "core.filemode", "false"]);
    let head = r.commit();
    r.plan(&[&first, &head]);
    r.git(&["config", "diff.renames", "false"]);

    let expanded = r.git(&["diff", "--binary", "--no-renames", &base, &head]);
    assert!(
        expanded.len() > 512000,
        "fixture must exceed the old ceiling"
    );
    let bundle = r.checked();
    assert!(bundle.len() < 512000);
    assert!(bundle.starts_with(&format!("=== REQUEST (frozen) ===\n{ROW}\n")));
    for line in [
        "mode: committed-plan".to_owned(),
        format!("base: {base}"),
        format!("head: {head}"),
        format!("task: T1 commit: {first} recorded: {first}"),
        format!("task: T2 commit: {head} recorded: {head}"),
    ] {
        assert_eq!(bundle.lines().filter(|one| *one == line).count(), 1);
    }
    let paths: Vec<_> = bundle
        .lines()
        .filter_map(|line| line.strip_prefix("--- file: "))
        .collect();
    assert_eq!(
        paths,
        [
            "allowed/binary.dat",
            "allowed/large.txt",
            "allowed/moved text.txt",
            "allowed/moved.dat",
            "allowed/run.sh",
        ]
    );
    for evidence in [
        "rename from allowed/large.txt\nrename to allowed/moved text.txt",
        "+TEXT_TAIL\n",
        "rename from allowed/binary.dat\nrename to allowed/moved.dat",
        "GIT binary patch",
        "old mode 100644\nnew mode 100755",
    ] {
        assert!(bundle.contains(evidence), "missing {evidence}");
    }
    let start = bundle.find("diff --git ").unwrap();
    let end = bundle.rfind("\n=== CONTRACT ===\n").unwrap();
    r.s.write("bundle.patch.txt", &bundle[start..end]);
    let expected_tree = r.git(&["rev-parse", &format!("{head}^{{tree}}")]);
    r.git(&["checkout", "--detach", &base]);
    r.git(&["apply", "--index", "--binary", "bundle.patch.txt"]);
    assert_eq!(r.git(&["write-tree"]), expected_tree);
    eprintln!(
        "R21 complete rename bundle: {} bytes; expanded patch: {} bytes; final tree: {expected_tree}",
        bundle.len(),
        expanded.len()
    );
}

#[test]
fn R21__intermediate_undeclared_move_is_refused_even_when_final_rename_is_declared() {
    let r = Repo::new();
    r.git(&["mv", "allowed/a.txt", "outside-moved.txt"]);
    let first = r.commit();
    r.git(&["mv", "outside-moved.txt", "allowed/final.txt"]);
    let head = r.commit();
    r.plan(&[&first, &head]);
    let final_paths = r.git(&[
        "diff",
        "--name-status",
        "--find-renames=50%",
        &r.base,
        &head,
    ]);
    assert_eq!(final_paths, "R100\tallowed/a.txt\tallowed/final.txt");
    r.refused("undeclared path: outside-moved.txt");
}
