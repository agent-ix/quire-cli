#!/usr/bin/env node
"use strict";

// Thin launcher: resolve the prebuilt `quire` binary that matches this
// platform/arch (shipped as an optional dependency) and exec it transparently,
// forwarding argv, stdio, and the exit status / terminating signal.

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const packageJson = require("../package.json");

const PLATFORM = process.platform; // 'linux' | 'darwin' | 'win32' | ...
const ARCH = process.arch; // 'x64' | 'arm64' | ...
const PACKAGE_PREFIX = "@agent-ix/quire-cli-";
const OPTIONAL_DEPENDENCIES = packageJson.optionalDependencies || {};

function resolveBinary() {
  const key = `${PLATFORM}-${ARCH}`;
  const pkg = `${PACKAGE_PREFIX}${key}`;
  if (!Object.prototype.hasOwnProperty.call(OPTIONAL_DEPENDENCIES, pkg)) {
    const supported = Object.keys(OPTIONAL_DEPENDENCIES)
      .filter((name) => name.startsWith(PACKAGE_PREFIX))
      .map((name) => name.slice(PACKAGE_PREFIX.length))
      .sort();
    throw new Error(
      `quire-cli: unsupported platform "${key}".\n` +
        `Prebuilt binaries exist for: ${supported.join(", ")}.\n` +
        `Build from source instead: https://github.com/agent-ix/quire-cli`
    );
  }
  const binName = PLATFORM === "win32" ? "quire.exe" : "quire";
  try {
    return require.resolve(`${pkg}/bin/${binName}`);
  } catch (_) {
    throw new Error(
      `quire-cli: the prebuilt binary package "${pkg}" is not installed.\n` +
        `It is an optional dependency — reinstall without --no-optional, or\n` +
        `add "${pkg}" explicitly. (--ignore-optional / CPU-VM mismatches skip it.)`
    );
  }
}

let bin;
try {
  bin = resolveBinary();
} catch (err) {
  process.stderr.write(String(err.message) + "\n");
  process.exit(1);
}

// npm does not always preserve the executable bit through publish/install;
// best-effort restore it on POSIX before exec.
if (PLATFORM !== "win32") {
  try {
    fs.chmodSync(bin, 0o755);
  } catch (_) {
    /* non-fatal: the file may already be executable or read-only */
  }
}

const result = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  process.stderr.write(`quire-cli: failed to launch binary: ${result.error.message}\n`);
  process.exit(1);
}
if (result.signal) {
  // Re-raise the child's terminating signal so callers observe the same
  // signal-driven exit (e.g. SIGABRT -> 134 on a panic, per FR-007).
  process.kill(process.pid, result.signal);
}
process.exit(result.status === null ? 1 : result.status);
