import { execSync, spawn } from "node:child_process";
import { homedir } from "node:os";
import { join } from "node:path";

const cargoBin = join(homedir(), ".cargo", "bin");
const sep = process.platform === "win32" ? ";" : ":";
process.env.PATH = `${cargoBin}${sep}${process.env.PATH ?? ""}`;

const raw = process.argv.slice(2);
const { channel, args } = takeChannel(raw);

if (channel) {
  process.env.LUMEN_CHANNEL = channel;
}

if (process.platform === "win32" && args[0] === "dev") {
  try {
    execSync(
      `powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \\"Name='octra.exe'\\" | Where-Object { $_.ExecutablePath -match 'target\\\\debug' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }"`,
      { stdio: "ignore", windowsHide: true },
    );
  } catch {
    // nothing running
  }
}

const child = spawn("tauri", args, {
  stdio: "inherit",
  shell: true,
  env: process.env,
});
child.on("exit", (code) => process.exit(code ?? 1));

function takeChannel(argv) {
  const args = [];
  let channel = (process.env.LUMEN_CHANNEL || "").trim();
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--channel" && argv[i + 1]) {
      channel = argv[++i];
      continue;
    }
    if (a.startsWith("--channel=")) {
      channel = a.slice("--channel=".length);
      continue;
    }
    args.push(a);
  }
  if (args[0] === "dev" && !channel) {
    channel = "dev";
  }
  return { channel, args };
}
