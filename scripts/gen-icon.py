#!/usr/bin/env python3
"""Geometric O mark on Beyond Black — Octra experimental icon."""
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
BG = (6, 3, 5, 255)
PURPLE = (160, 81, 162, 255)
LIGHT = (210, 210, 210, 255)


def mark(size: int) -> Image.Image:
    im = Image.new("RGBA", (size, size), BG)
    d = ImageDraw.Draw(im)
    pad = size * 0.14
    d.rounded_rectangle(
        [pad * 0.35, pad * 0.35, size - pad * 0.35, size - pad * 0.35],
        radius=size * 0.22,
        fill=(18, 12, 20, 255),
        outline=PURPLE,
        width=max(1, size // 48),
    )
    outer = size * 0.31
    inner = size * 0.175
    box = [size / 2 - outer, size / 2 - outer, size / 2 + outer, size / 2 + outer]
    d.ellipse(box, outline=PURPLE, width=max(2, size // 14))
    hole = [size / 2 - inner, size / 2 - inner, size / 2 + inner, size / 2 + inner]
    d.ellipse(hole, fill=(18, 12, 20, 255))
    return im


def main() -> None:
    icons = ROOT / "src-tauri" / "icons"
    icons.mkdir(parents=True, exist_ok=True)
    for s in (32, 64, 128, 256, 512):
        mark(s).save(icons / f"{s}x{s}.png")
    mark(256).save(icons / "128x128@2x.png")
    mark(128).save(icons / "128x128.png")
    mark(32).save(icons / "32x32.png")
    mark(256).save(icons / "icon.png")
    # Pillow writes a valid multi-size ICO from the largest raster.
    mark(256).save(
        icons / "icon.ico",
        format="ICO",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )
    # icns optional; Tauri on Windows does not need it
    (ROOT / "public").mkdir(exist_ok=True)
    svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
<rect width="1024" height="1024" rx="220" fill="#060305"/>
<rect x="70" y="70" width="884" height="884" rx="180" fill="#120c14" stroke="#a051a2" stroke-width="10"/>
<circle cx="512" cy="512" r="250" fill="none" stroke="#a051a2" stroke-width="92"/>
</svg>
"""
    (ROOT / "public" / "favicon.svg").write_text(svg)
    (ROOT / "app-icon.svg").write_text(svg)
    print("icons ok")


if __name__ == "__main__":
    main()
