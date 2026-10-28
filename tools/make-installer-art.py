#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Generate the art Workbench's installers show, from CIB's own files.

The Windows installer and the macOS disk image are the first thing of
Workbench anybody sees, and Tauri's defaults are blank: a grey NSIS wizard and
a white Finder window. This composes CIB's files onto those surfaces. Nothing
is redrawn, recoloured or cropped: each file is scaled whole, with its own
proportions, and placed on a ground.

The sidebar's ground is Night's: black with a CIB-blue ambient, never navy.
The header strip sits above white wizard pages, and Finder labels the disk
image's icons in dark text, so both take the colour logo on white.

Writes:
    src-tauri/installer/nsis-header.bmp     150 x 57, every wizard page's header
    src-tauri/installer/nsis-sidebar.bmp    164 x 314, the welcome and finish pages
    src-tauri/installer/dmg-background.png  660 x 400, the disk image's window

    python3 tools/make-installer-art.py           # writes the files
    python3 tools/make-installer-art.py --check   # verifies the committed pixels match

Requires Pillow.
"""

from __future__ import annotations

import io
import pathlib
import sys

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = pathlib.Path(__file__).resolve().parent.parent
CIB = ROOT / "assets" / "brand" / "cib"
OUT = ROOT / "src-tauri" / "installer"

LOGO_COLOUR = CIB / "cib-logo-color.png"
LOGO_REVERSED = CIB / "cib-logo-reversed.png"
ICON = CIB / "workbench-icon-1024.png"

CIB_BLUE = (0x0C, 0x52, 0x9A)
NIGHT = (0, 0, 0)
WHITE = (255, 255, 255)

# The disk image's two icons, centred on these points. `tauri.conf.json` says
# the same numbers, and `tools/installer.test.ts` holds the two together.
DMG_SIZE = (660, 400)
DMG_APP = (180, 190)
DMG_APPLICATIONS = (480, 190)


def scaled(path: pathlib.Path, width: int | None = None, height: int | None = None) -> Image.Image:
    """CIB's file, whole, at a width or a height, keeping its proportions."""
    image = Image.open(path).convert("RGBA")
    if width is not None:
        height = round(image.height * width / image.width)
    else:
        width = round(image.width * height / image.height)
    return image.resize((width, height), Image.LANCZOS)


def night(size: tuple[int, int], centre: tuple[float, float], reach: float) -> Image.Image:
    """Black, lit from one point by CIB blue that fades out at `reach`."""
    w, h = size
    glow = Image.new("L", size, 0)
    cx, cy = centre[0] * w, centre[1] * h
    r = reach * max(w, h)
    ImageDraw.Draw(glow).ellipse((cx - r, cy - r, cx + r, cy + r), fill=150)
    glow = glow.filter(ImageFilter.GaussianBlur(r / 3))
    ground = Image.new("RGBA", size, NIGHT + (255,))
    ground.paste(Image.new("RGBA", size, CIB_BLUE + (255,)), (0, 0), glow)
    return ground


def place(ground: Image.Image, image: Image.Image, centre: tuple[int, int]) -> None:
    ground.alpha_composite(image, (centre[0] - image.width // 2, centre[1] - image.height // 2))


def header() -> Image.Image:
    ground = Image.new("RGBA", (150, 57), WHITE + (255,))
    place(ground, scaled(LOGO_COLOUR, height=41), (75, 28))
    return ground


def sidebar() -> Image.Image:
    ground = night((164, 314), (0.5, 0.36), 0.22)
    place(ground, scaled(ICON, width=88), (82, 112))
    place(ground, scaled(LOGO_REVERSED, width=112), (82, 262))
    return ground


def dmg() -> Image.Image:
    ground = Image.new("RGBA", DMG_SIZE, WHITE + (255,))
    # An arrow from the app to Applications, in place of words.
    draw = ImageDraw.Draw(ground)
    y = DMG_APP[1]
    start, end = DMG_APP[0] + 84, DMG_APPLICATIONS[0] - 84
    draw.line((start, y, end - 10, y), fill=CIB_BLUE + (255,), width=3)
    draw.polygon([(end, y), (end - 16, y - 9), (end - 16, y + 9)], fill=CIB_BLUE + (255,))
    place(ground, scaled(LOGO_COLOUR, height=44), (DMG_SIZE[0] // 2, 345))
    return ground


def bmp(image: Image.Image) -> bytes:
    # NSIS reads 24-bit bitmaps; an alpha channel makes it show nothing.
    out = io.BytesIO()
    image.convert("RGB").save(out, format="BMP")
    return out.getvalue()


def png(image: Image.Image) -> bytes:
    out = io.BytesIO()
    image.convert("RGB").save(out, format="PNG", optimize=True)
    return out.getvalue()


def outputs() -> dict[pathlib.Path, tuple[Image.Image, bytes]]:
    made = {
        OUT / "nsis-header.bmp": (header(), bmp),
        OUT / "nsis-sidebar.bmp": (sidebar(), bmp),
        OUT / "dmg-background.png": (dmg(), png),
    }
    return {path: (image, encode(image)) for path, (image, encode) in made.items()}


# How far a channel may differ between this machine's drawing and the file's,
# for the reason make-workbench-icon.py gives: the check compares pixels.
TOLERANCE = 2


def matches(path: pathlib.Path, image: Image.Image) -> bool:
    if not path.exists():
        return False
    committed = Image.open(path).convert("RGB")
    if committed.size != image.size:
        return False
    diff = ImageChops.difference(committed, image.convert("RGB")).getextrema()
    return all(high <= TOLERANCE for _, high in diff)


def main(argv: list[str]) -> int:
    check = "--check" in argv
    stale = []
    for path, (image, data) in outputs().items():
        if check:
            if not matches(path, image):
                stale.append(path.relative_to(ROOT).as_posix())
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            print(path.relative_to(ROOT).as_posix())
    if stale:
        print("out of date, run python3 tools/make-installer-art.py:", *stale, sep="\n  ")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
