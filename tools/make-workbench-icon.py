#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Generate Workbench's application icon and in-app marks from CIB's globe.

Workbench's icon is CIB's round globe (`assets/brand/cib/cib-globe-round.png`,
CIB's own file, never redrawn or recoloured) on a plate of CIB's primary blue,
#1858A8 — the colour cibeg.com names `primary-blue`. The proportions are the
handoff's (`design_handoff_cib_workbench/source/Mark.dc.html`): a 168 px
plate with a 39 px corner radius holds a 96 px globe.

This replaces `tools/make-icons.py` and the favicon part of
`tools/make-brand.py` for Workbench; both draw Spagitty's mark.

Writes:
    assets/brand/cib/workbench-icon-1024.png   the source for `bun tauri icon`
    assets/brand/cib/workbench-icon-64.png     the in-app mark (BrandMark)

Then run `bun tauri icon assets/brand/cib/workbench-icon-1024.png -o src-tauri/icons`.

    python3 tools/make-workbench-icon.py           # writes the files
    python3 tools/make-workbench-icon.py --check   # verifies the committed files match

Requires Pillow.
"""

from __future__ import annotations

import io
import pathlib
import sys

from PIL import Image, ImageDraw

ROOT = pathlib.Path(__file__).resolve().parent.parent
CIB = ROOT / "assets" / "brand" / "cib"
GLOBE = CIB / "cib-globe-round.png"

# CIB's primary blue, from cibeg.com's own stylesheet (`.bg-primary-blue`).
PLATE = (0x18, 0x58, 0xA8, 255)

# The handoff's icon: plate 168, radius 39, globe 96.
RADIUS = 39 / 168
GLOBE_SIZE = 96 / 168

SUPERSAMPLE = 4


def render(size: int) -> Image.Image:
    big = size * SUPERSAMPLE
    plate = Image.new("RGBA", (big, big), (0, 0, 0, 0))
    ImageDraw.Draw(plate).rounded_rectangle(
        (0, 0, big - 1, big - 1), radius=round(big * RADIUS), fill=PLATE
    )
    globe = Image.open(GLOBE).convert("RGBA")
    side = round(big * GLOBE_SIZE)
    globe = globe.resize((side, round(side * globe.height / globe.width)), Image.LANCZOS)
    plate.alpha_composite(globe, ((big - globe.width) // 2, (big - globe.height) // 2))
    return plate.resize((size, size), Image.LANCZOS)


def png(image: Image.Image) -> bytes:
    out = io.BytesIO()
    image.save(out, format="PNG", optimize=True)
    return out.getvalue()


def outputs() -> dict[pathlib.Path, bytes]:
    return {
        CIB / "workbench-icon-1024.png": png(render(1024)),
        CIB / "workbench-icon-64.png": png(render(64)),
    }


def main(argv: list[str]) -> int:
    check = "--check" in argv
    stale = []
    for path, data in outputs().items():
        if check:
            if not path.exists() or path.read_bytes() != data:
                stale.append(path.relative_to(ROOT).as_posix())
        else:
            path.write_bytes(data)
            print(path.relative_to(ROOT).as_posix())
    if stale:
        print("out of date, run python3 tools/make-workbench-icon.py:", *stale, sep="\n  ")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
