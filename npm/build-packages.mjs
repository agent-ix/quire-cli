// Generate the per-platform npm packages from prebuilt binaries.
//
// Reads built binaries from <ARTIFACTS_DIR>/<rust-target>/quire[.exe] and emits
// ready-to-publish packages under npm/dist/<name>/. The CI release job downloads
// the build matrix artifacts into that layout, runs this, then `npm publish`es
// each dist/* package plus npm/quire-cli (the launcher).
//
// Usage:  node npm/build-packages.mjs <version>
//   env   ARTIFACTS_DIR (default: ./artifacts)

import { mkdirSync, copyFileSync, writeFileSync, chmodSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(here, "..");

// rust target triple -> npm (process.platform, process.arch)
const TARGETS = [
  { rust: "x86_64-unknown-linux-musl", platform: "linux", arch: "x64", windows: false },
  { rust: "aarch64-unknown-linux-musl", platform: "linux", arch: "arm64", windows: false },
  { rust: "x86_64-apple-darwin", platform: "darwin", arch: "x64", windows: false },
  { rust: "aarch64-apple-darwin", platform: "darwin", arch: "arm64", windows: false },
  { rust: "x86_64-pc-windows-msvc", platform: "win32", arch: "x64", windows: true },
];

const version = process.env.QUIRE_VERSION || process.argv[2];
if (!version) {
  console.error("usage: node npm/build-packages.mjs <version>  (or set QUIRE_VERSION)");
  process.exit(1);
}

const artifactsRoot = process.env.ARTIFACTS_DIR || join(repoRoot, "artifacts");
const outRoot = join(here, "dist");

let built = 0;
for (const t of TARGETS) {
  const binName = t.windows ? "quire.exe" : "quire";
  const src = join(artifactsRoot, t.rust, binName);
  if (!existsSync(src)) {
    console.error(`missing binary for ${t.rust}: ${src}`);
    process.exit(1);
  }

  const pkgName = `quire-cli-${t.platform}-${t.arch}`;
  const pkgDir = join(outRoot, pkgName);
  mkdirSync(join(pkgDir, "bin"), { recursive: true });

  const dest = join(pkgDir, "bin", binName);
  copyFileSync(src, dest);
  if (!t.windows) chmodSync(dest, 0o755);

  const pkgJson = {
    name: `@agent-ix/${pkgName}`,
    version,
    description: `Prebuilt quire binary for ${t.platform}-${t.arch}.`,
    homepage: "https://github.com/agent-ix/quire-cli#readme",
    repository: { type: "git", url: "git+https://github.com/agent-ix/quire-cli.git" },
    license: "MIT",
    os: [t.platform],
    cpu: [t.arch],
    files: ["bin/"],
    publishConfig: { registry: "https://npm.pkg.github.com" },
  };
  writeFileSync(join(pkgDir, "package.json"), JSON.stringify(pkgJson, null, 2) + "\n");
  console.log(`built @agent-ix/${pkgName}@${version}`);
  built++;
}

console.log(`\n${built} platform package(s) written under ${outRoot}`);
