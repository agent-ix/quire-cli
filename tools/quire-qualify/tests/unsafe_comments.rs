use std::fs;

use ix_trace_rs::trace;

fn source(root: &std::path::Path, value: &str) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join("src/lib.rs"), value).unwrap();
}

#[trace("TC-837", "FR-025-AC-5")]
#[test]
fn unsafe_policy_covers_comments_exact_baselines_staleness_and_explicit_update() {
    let dir = tempfile::tempdir().unwrap();
    source(
        dir.path(),
        "fn f() {\n    // SAFETY: test-only empty unsafe block.\n    unsafe {}\n}\n",
    );
    quire_qualify::unsafe_comments::check(dir.path()).expect("documented unsafe block");

    source(dir.path(), "fn f() {\n    unsafe {}\n}\n");
    let error =
        quire_qualify::unsafe_comments::check(dir.path()).expect_err("undocumented unsafe block");
    assert!(matches!(
        error,
        quire_qualify::Error::UnsafeBaseline { unreviewed, stale }
            if unreviewed == ["src/lib.rs:2"] && stale.is_empty()
    ));

    source(
        dir.path(),
        "fn f() {\n    let _ = r#\"// SAFETY: string data is not a comment\"#;\n    unsafe {}\n}\n",
    );
    assert!(matches!(
        quire_qualify::unsafe_comments::check(dir.path()),
        Err(quire_qualify::Error::UnsafeBaseline { unreviewed, .. })
            if unreviewed == ["src/lib.rs:3"]
    ));

    source(dir.path(), "fn f() {\n    unsafe {}\n}\n");

    fs::write(
        dir.path().join("scripts/unsafe_comment_baseline.txt"),
        "src/lib.rs:2\n",
    )
    .unwrap();
    quire_qualify::unsafe_comments::check(dir.path()).expect("exact reviewed locus");

    fs::write(
        dir.path().join("scripts/unsafe_comment_baseline.txt"),
        "src/lib.rs:3\n",
    )
    .unwrap();
    let error = quire_qualify::unsafe_comments::check(dir.path())
        .expect_err("wrong and stale baseline locus");
    assert!(matches!(
        error,
        quire_qualify::Error::UnsafeBaseline { unreviewed, stale }
            if unreviewed == ["src/lib.rs:2"] && stale == ["src/lib.rs:3"]
    ));

    assert_eq!(
        quire_qualify::unsafe_comments::update_baseline(dir.path()).unwrap(),
        1
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("scripts/unsafe_comment_baseline.txt")).unwrap(),
        "src/lib.rs:2\n"
    );
    quire_qualify::unsafe_comments::check(dir.path()).expect("updated exact baseline");

    source(
        dir.path(),
        "fn f() {\n    // SAFETY: now documented.\n    unsafe {}\n}\n",
    );
    let error = quire_qualify::unsafe_comments::check(dir.path())
        .expect_err("obsolete exemption must be rejected");
    assert!(matches!(
        error,
        quire_qualify::Error::UnsafeBaseline { unreviewed, stale }
            if unreviewed.is_empty() && stale == ["src/lib.rs:2"]
    ));

    assert_eq!(
        quire_qualify::unsafe_comments::update_baseline(dir.path()).unwrap(),
        0
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("scripts/unsafe_comment_baseline.txt")).unwrap(),
        ""
    );
}

#[cfg(unix)]
#[trace("TC-837", "FR-025-AC-5")]
#[test]
fn baseline_update_refuses_symlink_redirection() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().unwrap();
    source(dir.path(), "fn f() {\n    unsafe {}\n}\n");
    let outside = tempfile::NamedTempFile::new().unwrap();
    fs::write(outside.path(), "do not replace\n").unwrap();
    symlink(
        outside.path(),
        dir.path().join("scripts/unsafe_comment_baseline.txt"),
    )
    .unwrap();

    assert!(matches!(
        quire_qualify::unsafe_comments::check(dir.path()),
        Err(quire_qualify::Error::SymlinkBaseline { .. })
    ));
    assert!(matches!(
        quire_qualify::unsafe_comments::update_baseline(dir.path()),
        Err(quire_qualify::Error::SymlinkBaseline { .. })
    ));
    assert_eq!(
        fs::read_to_string(outside.path()).unwrap(),
        "do not replace\n"
    );

    fs::remove_file(dir.path().join("scripts/unsafe_comment_baseline.txt")).unwrap();
    symlink(
        dir.path().join("missing-baseline-target"),
        dir.path().join("scripts/unsafe_comment_baseline.txt"),
    )
    .unwrap();
    assert!(matches!(
        quire_qualify::unsafe_comments::check(dir.path()),
        Err(quire_qualify::Error::SymlinkBaseline { .. })
    ));
    assert!(matches!(
        quire_qualify::unsafe_comments::update_baseline(dir.path()),
        Err(quire_qualify::Error::SymlinkBaseline { .. })
    ));
}
