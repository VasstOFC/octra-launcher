import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const version = pkg.version;
const flags = new Set(process.argv.slice(2));
const buildLauncher = flags.has("--build-launcher");
const sign = flags.has("--sign");

const cargoBin = join(homedir(), ".cargo", "bin");
const sep = process.platform === "win32" ? ";" : ":";
process.env.PATH = `${cargoBin}${sep}${process.env.PATH ?? ""}`;

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, {
    stdio: "inherit",
    shell: true,
    cwd: root,
    env: process.env,
    ...opts,
  });
  if ((r.status ?? 1) !== 0) {
    process.exit(r.status ?? 1);
  }
}

function runExe(exe, args) {
  const r = spawnSync(exe, args, {
    stdio: "inherit",
    shell: false,
    cwd: root,
    env: process.env,
  });
  if ((r.status ?? 1) !== 0) {
    process.exit(r.status ?? 1);
  }
}

const lumenExe = join(root, "src-tauri", "target", "release", "octra.exe");

if (buildLauncher) {
  console.log("Budowanie launchera…");
  run("npm", ["run", "tauri", "--", "build", "--no-bundle"]);
} else if (!existsSync(lumenExe)) {
  console.error(`Brak ${lumenExe}`);
  console.error("Najpierw: npm run tauri build");
  console.error("Albo od razu: npm run installer:pack -- --build-launcher");
  process.exit(1);
}

const stage = join(root, "installer", ".payload-stage");
rmSync(stage, { recursive: true, force: true });
mkdirSync(join(stage, "packs"), { recursive: true });
copyFileSync(lumenExe, join(stage, "octra.exe"));

const packsDir = join(root, "packs");
if (existsSync(packsDir)) {
  for (const name of readdirSync(packsDir)) {
    if (name.toLowerCase().endsWith(".mrpack")) {
      copyFileSync(join(packsDir, name), join(stage, "packs", name));
    }
  }
}

console.log("Budowanie okna instalatora…");
run("cargo", ["build", "--release", "--manifest-path", "installer/Cargo.toml"]);

const stub = join(root, "installer", "target", "release", "octra-installer.exe");
if (!existsSync(stub)) {
  console.error("Nie powstał octra-installer.exe");
  process.exit(1);
}

const dist = join(root, "dist-installer");
mkdirSync(dist, { recursive: true });
const stable = join(dist, "Octra-setup.exe");

console.log("Pakowanie payloadu do instalatora…");
runExe(stub, [
  "--make-sfx",
  "--stub",
  stub,
  "--payload-dir",
  stage,
  "--out",
  stable,
]);
console.log(`Instalator: ${stable}`);

if (sign) {
  if (!process.env.TAURI_SIGNING_PRIVATE_KEY) {
    console.error("Brak TAURI_SIGNING_PRIVATE_KEY — nie można podpisać instalatora.");
    process.exit(1);
  }
  run("npx", ["tauri", "signer", "sign", stable]);
}

if (sign || flags.has("--write-manifest")) {
  writeLatestJson(stable, version);
}

function writeLatestJson(exePath, ver) {
  const sigPath = `${exePath}.sig`;
  if (!existsSync(sigPath)) {
    console.error(`Brak podpisu: ${sigPath}`);
    process.exit(1);
  }
  const tag = `v${ver}`;
  const latest = {
    version: ver,
    notes: process.env.OCTRA_RELEASE_NOTES || "Nowa wersja Octra (dev).",
    pub_date: new Date().toISOString(),
    platforms: {
      "windows-x86_64": {
        signature: readFileSync(sigPath, "utf8").trim(),
        url: `https://github.com/VasstOFC/octra-launcher/releases/download/${tag}/Octra-setup.exe`,
      },
    },
  };
  const out = join(root, "dist-installer", "latest.json");
  writeFileSync(out, `${JSON.stringify(latest, null, 2)}\n`);
  console.log(`Manifest: ${out}`);
}
