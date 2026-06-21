import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

function assertSemver(version) {
  // Accepts stable SemVer only for release tags (no prerelease/build metadata).
  const semverPattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
  if (!semverPattern.test(version)) {
    throw new Error(`Invalid SemVer version: ${version}`);
  }
}

async function updatePackageJson(version) {
  const packagePath = resolve("package.json");
  const raw = await readFile(packagePath, "utf8");
  const data = JSON.parse(raw);
  data.version = version;
  await writeFile(packagePath, `${JSON.stringify(data, null, 2)}\n`, "utf8");
}

async function updateCargoToml(version) {
  const cargoPath = resolve("src-tauri", "Cargo.toml");
  const raw = await readFile(cargoPath, "utf8");
  // Update only the package version, not dependency inline table versions.
  const packageVersionPattern = /(^\[package\][\s\S]*?^\s*version\s*=\s*")[^"]+("\s*$)/m;
  const updated = raw.replace(packageVersionPattern, `$1${version}$2`);

  if (updated === raw) {
    throw new Error("Could not find version field in src-tauri/Cargo.toml");
  }

  await writeFile(cargoPath, updated, "utf8");
}

async function updateTauriConfig(version) {
  const tauriPath = resolve("src-tauri", "tauri.conf.json");
  const raw = await readFile(tauriPath, "utf8");
  const data = JSON.parse(raw);

  if (!data.package) {
    data.package = {};
  }

  data.package.version = version;
  await writeFile(tauriPath, `${JSON.stringify(data, null, 2)}\n`, "utf8");
}

async function main() {
  const version = process.argv[2];

  if (!version) {
    throw new Error("Missing version argument. Usage: bun run release:sync-version <x.y.z>");
  }

  assertSemver(version);

  await Promise.all([
    updatePackageJson(version),
    updateCargoToml(version),
    updateTauriConfig(version)
  ]);

  process.stdout.write(`Synchronized app version to ${version}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error.message}\n`);
  process.exit(1);
});
