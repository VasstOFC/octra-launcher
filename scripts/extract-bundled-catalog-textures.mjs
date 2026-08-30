import fs from "fs";

const srcPath =
  process.argv[2] ||
  "C:/Users/mrcra/.cursor/projects/g-Octra-Launcher/agent-tools/a8565d9f-bb8a-4d53-9dbe-73636d343ce5.txt";
const dest = new URL("../src-tauri/resources/catalog_bundled_textures.json", import.meta.url);

const src = fs.readFileSync(srcPath, "utf8");
const keys = [
  "890044fb07cbca79bb9ffec4d2f15cdd1053e4b554e9a02469e9d0b271f3fdfa",
  "8d0484011053097a9809f14c0301166981369b3a660150afea1e753ae7e54685",
  "b240795e214270b5b864cea3cbbcbac2fae60abed5de10229a7567510713355b",
  "fa4d0a00cfabcad04659b991176e2b7872c661d54e85808bc12aa59e10dde326",
  "bfa251327ce1fc617dc90879dbfb77dabf151381d5d40f261c7f16e2d147d942",
  "2d89c01ed54a6fa08d534c07200ccddda426183e642d17cef648d90716f4aa92",
];

const out = {};
for (const key of keys) {
  const marker = `texture_key: Arc::from("${key}")`;
  const idx = src.indexOf(marker);
  if (idx < 0) {
    console.error("missing", key.slice(0, 12));
    continue;
  }
  const chunk = src.slice(idx, idx + 12000);
  const m = chunk.match(/data:image\/png;base64,([A-Za-z0-9+/=]+)/);
  if (!m) {
    console.error("no b64 for", key.slice(0, 12));
    continue;
  }
  out[key] = m[1];
}

fs.writeFileSync(dest, JSON.stringify(out));
console.log(`Wrote ${Object.keys(out).length} bundled textures`);
