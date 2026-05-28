//! AUDIT-001 (NFR-002): the release binary links only against libc,
//! libm, libpthread, libdl, ld-linux, and similar baseline dynamic deps —
//! NO project-private `.so` files. Linux-only.

#![cfg(target_os = "linux")]

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

/// Allowlist of `ldd` library basenames considered "system baseline".
/// Anything else is flagged as a project-private dynamic dependency.
const ALLOWED_LIBS: &[&str] = &[
    "linux-vdso.so.1",
    "libc.so.6",
    "libm.so.6",
    "libpthread.so.0",
    "libdl.so.2",
    "libgcc_s.so.1",
    "librt.so.1",
    "libutil.so.1",
    "libresolv.so.2",
    "ld-linux-x86-64.so.2",
    "ld-linux-aarch64.so.1",
];

fn is_allowed(lib_name: &str) -> bool {
    ALLOWED_LIBS.contains(&lib_name) || lib_name.starts_with("ld-linux-")
}

#[test]
fn binary_links_only_baseline_libs() {
    // Resolve the binary path via the same mechanism `assert_cmd` uses.
    let cmd = Command::cargo_bin("quire").expect("quire binary built");
    let bin = cmd.get_program().to_os_string();

    let out = Command::new("ldd")
        .arg(&bin)
        .output()
        .expect("run ldd on the built binary");
    let stdout = String::from_utf8_lossy(&out.stdout);

    let mut offenders = Vec::new();
    for line in stdout.lines() {
        // `ldd` output forms:
        //   linux-vdso.so.1 (0x...)
        //   libc.so.6 => /lib/.../libc.so.6 (0x...)
        //   /lib64/ld-linux-x86-64.so.2 (0x...)
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Pull the first whitespace-delimited token, then keep only its
        // basename.
        let head = trimmed.split_whitespace().next().unwrap_or("");
        let name = head.rsplit('/').next().unwrap_or(head);
        if !is_allowed(name) {
            offenders.push(name.to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "AUDIT-001 violation: binary depends on non-baseline shared libraries: {offenders:?}\n\
         full ldd output:\n{stdout}"
    );
}
