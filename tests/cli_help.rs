//! IT-032 / NFR-006-AC-2: `quire --help` snapshot pinned at
//! `tests/snapshots/help.txt`. Regenerate by running the binary and
//! committing the new file — no insta dep.

mod common;

use common::quire;

#[test]
fn it_032_help_output_matches_pinned_snapshot() {
    let snapshot = include_str!("snapshots/help.txt");
    let out = quire().arg("--help").output().expect("help runs");
    assert!(out.status.success(), "--help should exit 0");
    let actual = String::from_utf8(out.stdout).expect("help output is UTF-8");
    assert_eq!(
        actual.trim_end(),
        snapshot.trim_end(),
        "--help output drifted from snapshot at tests/snapshots/help.txt; \
         to update, run `target/release/quire --help > tests/snapshots/help.txt`."
    );
}
