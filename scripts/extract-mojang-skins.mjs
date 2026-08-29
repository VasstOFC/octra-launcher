import fs from "fs";

const text = fs.readFileSync(
  process.argv[2] ||
    "C:/Users/mrcra/.cursor/projects/g-Octra-Launcher/agent-tools/85e6dafb-52f3-4a57-a356-5e6297e6d20a.txt",
  "utf8",
);

const skins = [];
const blockRe =
  /texture_key: Arc::from\("([^"]+)"\),\s*name: Some\(Arc::from\("([^"]+)"\)\),\s*section: Some\(Arc::from\(([^)]+)\)\),\s*variant: MinecraftSkinVariant::(Classic|Slim)/g;

const sectionMap = {
  DEFAULT_SKINS_SECTION: "Default skins",
  MINECON_EARTH_2017_SKIN_PACK_SECTION: "MINECON Earth 2017",
  BUILDERS_AND_BIOMES_SKIN_PACK_SECTION: "Builders & Biomes",
  STRIDING_HERO_SKIN_PACK_SECTION: "Striding Hero",
  THE_GARDEN_AWAKENS_SKIN_PACK_SECTION: "The Garden Awakens",
  CHASE_THE_SKIES_SKIN_PACK_SECTION: "Chase the Skies",
  THE_COPPER_AGE_SKIN_PACK_SECTION: "The Copper Age",
  MOUNTS_OF_MAYHEM_SKIN_PACK_SECTION: "Mounts of Mayhem",
  TINY_TAKEOVER_SKIN_PACK_SECTION: "Tiny Takeover",
  CHAOS_CUBED_SKIN_PACK_SECTION: "Chaos Cubed",
};

let m;
while ((m = blockRe.exec(text))) {
  const textureKey = m[1];
  if (textureKey.startsWith("local-")) continue;
  const sectionRef = m[3].trim();
  const section =
    Object.entries(sectionMap).find(([k]) => sectionRef.includes(k))?.[1] ??
    "Other";
  skins.push({
    textureKey,
    name: m[2],
    section,
    variant: m[4].toLowerCase(),
  });
}

const groups = new Map();
for (const s of skins) {
  if (!groups.has(s.section)) groups.set(s.section, []);
  groups.get(s.section).push({
    id: s.textureKey.slice(0, 12),
    name: s.name,
    textureKey: s.textureKey,
    variant: s.variant,
  });
}

const order = [
  "Default skins",
  "MINECON Earth 2017",
  "Builders & Biomes",
  "Striding Hero",
  "The Garden Awakens",
  "Chase the Skies",
  "The Copper Age",
  "Mounts of Mayhem",
  "Tiny Takeover",
  "Chaos Cubed",
];

const out = order
  .filter((title) => groups.has(title))
  .map((title) => ({
    id: title.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
    title,
    skins: groups.get(title),
  }));

const dest = new URL("../src-tauri/resources/mojang_skins.json", import.meta.url);
fs.mkdirSync(new URL("../src-tauri/resources/", import.meta.url), {
  recursive: true,
});
fs.writeFileSync(dest, JSON.stringify(out, null, 2));
console.log(`Wrote ${skins.length} skins in ${out.length} groups`);
