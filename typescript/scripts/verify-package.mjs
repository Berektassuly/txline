import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npmExecPath = process.env.npm_execpath;
const npm = npmExecPath ? process.execPath : process.platform === "win32" ? "npm.cmd" : "npm";
const npmArgsPrefix = npmExecPath ? [npmExecPath] : [];
const useShell = !npmExecPath && process.platform === "win32";

function npmJson(args, options = {}) {
  const stdout = execFileSync(npm, [...npmArgsPrefix, ...args], {
    cwd: packageRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
    shell: useShell,
    ...options,
  });
  return JSON.parse(stdout);
}

const dryRun = npmJson(["pack", "--dry-run", "--json"])[0];
const files = new Set(dryRun.files.map((entry) => entry.path));

for (const required of [
  "package.json",
  "README.md",
  "NOTES.md",
  "dist/index.js",
  "dist/index.d.ts",
]) {
  if (!files.has(required)) {
    throw new Error(`package is missing ${required}`);
  }
}

for (const forbiddenPrefix of ["src/", "tests/", "coverage/", "node_modules/"]) {
  const leaked = [...files].find((file) => file.startsWith(forbiddenPrefix));
  if (leaked) {
    throw new Error(`package includes non-published source artifact ${leaked}`);
  }
}

const tempDir = mkdtempSync(join(tmpdir(), "txline-npm-pack-"));

try {
  const packed = npmJson(["pack", "--json", "--pack-destination", tempDir])[0];
  const tarball = join(tempDir, packed.filename);
  const smokeDir = join(tempDir, "smoke");

  writeFileSync(
    join(tempDir, "package.json"),
    JSON.stringify({ private: true, type: "module" }, null, 2),
  );

  execFileSync(
    npm,
    [
      ...npmArgsPrefix,
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--package-lock=false",
      tarball,
    ],
    { cwd: tempDir, stdio: "inherit", shell: useShell },
  );

  rmSync(smokeDir, { recursive: true, force: true });
  writeFileSync(
    join(tempDir, "smoke.mjs"),
    [
      'import { TxlineClient, devnetConfig } from "@beriktassuly/txline";',
      "const client = new TxlineClient({ config: devnetConfig() });",
      "if (!client) throw new Error('failed to construct client');",
      "",
    ].join("\n"),
  );
  execFileSync(process.execPath, [join(tempDir, "smoke.mjs")], {
    stdio: "inherit",
  });
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}
