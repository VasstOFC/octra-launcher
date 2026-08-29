import fs from "fs";
import path from "path";

const VERSION = "1.21.1";
const BASE = `https://assets.mcasset.cloud/${VERSION}/assets/minecraft/textures`;
const OUT = path.join("public", "mc-textures");

/** id → ścieżka tekstury w jarze Minecraft */
const ICONS = {
  grass: "block/grass_block_top.png",
  dirt: "block/dirt.png",
  stone: "block/stone.png",
  cobblestone: "block/cobblestone.png",
  oak_log: "block/oak_log.png",
  sand: "block/sand.png",
  netherrack: "block/netherrack.png",
  obsidian: "block/obsidian.png",
  end_stone: "block/end_stone.png",
  tnt: "block/tnt_side.png",
  chest: "entity/chest/normal.png",
  crafting_table: "block/crafting_table_top.png",
  beacon: "block/beacon.png",
  diamond: "item/diamond.png",
  emerald: "item/emerald.png",
  gold: "item/gold_ingot.png",
  iron: "item/iron_ingot.png",
  redstone: "item/redstone.png",
  netherite: "item/netherite_ingot.png",
  lapis: "item/lapis_lazuli.png",
  cobblemon: "item/turtle_egg.png",
  pixelmon: "item/golden_apple.png",
  aged: "item/brick.png",
  create: "item/copper_ingot.png",
  botania: "block/poppy.png",
  twilight: "item/ender_eye.png",
  atm: "item/nether_star.png",
  skyblock: "block/oak_sapling.png",
  vault_hunters: "item/trial_key.png",
  ftb: "item/compass_16.png",
  better_mc: "item/diamond_sword.png",
  rlcraft: "item/rotten_flesh.png",
};

fs.mkdirSync(OUT, { recursive: true });

let ok = 0;
let fail = 0;

for (const [id, rel] of Object.entries(ICONS)) {
  const url = `${BASE}/${rel}`;
  const dest = path.join(OUT, `${id}.png`);
  try {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const buf = Buffer.from(await res.arrayBuffer());
    fs.writeFileSync(dest, buf);
    ok++;
    console.log(`✓ ${id}`);
  } catch (e) {
    fail++;
    console.error(`✗ ${id}: ${e}`);
  }
}

console.log(`\nDone: ${ok} ok, ${fail} failed → ${OUT}`);
