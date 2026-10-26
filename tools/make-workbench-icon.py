#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Generate Workbench's application icon and in-app marks from CIB's globe.

Workbench's icon is CIB's round globe (`assets/brand/cib/cib-globe-round.png`,
CIB's own file, never redrawn, recoloured or covered) on a plate of CIB's
blue, #0C529A — the plate of CIB's own logo lockups — with a
white tools badge at the bottom right: a wrench crossed with a screwdriver.

The badge is what makes it a developer tool rather than the bank: the globe
alone on CIB blue is CIB's own mobile-app icon, which Workbench must not be
mistaken for (the author, 2026-10-26). So the globe is moved up and left and
made smaller, and the badge sits clear of it; nothing is drawn over the globe.
The plate keeps the handoff's corner (39 px on a 168 px plate).

This replaces `tools/make-icons.py` and the favicon part of
`tools/make-brand.py` for Workbench; both draw Spagitty's mark.

Writes:
    assets/brand/cib/workbench-icon-1024.png   the source for `bun tauri icon`
    assets/brand/cib/workbench-icon-64.png     the in-app mark (BrandMark)

Then run `bun tauri icon assets/brand/cib/workbench-icon-1024.png -o src-tauri/icons`.

    python3 tools/make-workbench-icon.py           # writes the files
    python3 tools/make-workbench-icon.py --check   # verifies the committed pixels match

Requires Pillow.
"""

from __future__ import annotations

import io
import pathlib
import sys

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = pathlib.Path(__file__).resolve().parent.parent
CIB = ROOT / "assets" / "brand" / "cib"
GLOBE = CIB / "cib-globe-round.png"

# CIB blue, the plate of CIB's logo lockups.
PLATE = (0x0C, 0x52, 0x9A, 255)
BADGE = (255, 255, 255, 255)

# Fractions of the plate. The handoff's corner: radius 39 on a 168 plate.
RADIUS = 39 / 168
# The globe, up and left; the badge, bottom right, 0.03 clear of it.
GLOBE_CENTRE, GLOBE_SIZE = 0.42, 0.50
BADGE_CENTRE, BADGE_SIZE = 0.745, 0.36

SUPERSAMPLE = 4


def wrench(d: int) -> Image.Image:
    """An open-ended wrench with a ring at the other end, lying flat, as a mask."""
    m = Image.new("L", (d, d), 0)
    g = ImageDraw.Draw(m)
    c = d / 2
    u = lambda f: f * d  # noqa: E731
    g.rounded_rectangle((c - u(0.30), c - u(0.07), c + u(0.24), c + u(0.07)), radius=u(0.07), fill=255)
    g.ellipse((c - u(0.45), c - u(0.15), c - u(0.15), c + u(0.15)), fill=255)
    g.rectangle((c - u(0.47), c - u(0.06), c - u(0.32), c + u(0.06)), fill=0)
    g.ellipse((c + u(0.17), c - u(0.105), c + u(0.38), c + u(0.105)), fill=255)
    g.ellipse((c + u(0.225), c - u(0.05), c + u(0.325), c + u(0.05)), fill=0)
    return m


def screwdriver(d: int) -> Image.Image:
    """A flat screwdriver lying flat, handle to the right, as a mask."""
    m = Image.new("L", (d, d), 0)
    g = ImageDraw.Draw(m)
    c = d / 2
    u = lambda f: f * d  # noqa: E731
    g.rounded_rectangle((c + u(0.07), c - u(0.095), c + u(0.41), c + u(0.095)), radius=u(0.07), fill=255)
    g.rectangle((c + u(0.03), c - u(0.06), c + u(0.07), c + u(0.06)), fill=255)
    g.rectangle((c - u(0.31), c - u(0.034), c + u(0.03), c + u(0.034)), fill=255)
    g.polygon(
        [(c - u(0.31), c - u(0.034)), (c - u(0.39), c - u(0.016)), (c - u(0.39), c + u(0.016)), (c - u(0.31), c + u(0.034))],
        fill=255,
    )
    return m


def badge(d: int) -> Image.Image:
    """The white disc with the two tools crossed on it, in CIB blue."""
    out = Image.new("RGBA", (d, d), (0, 0, 0, 0))
    ImageDraw.Draw(out).ellipse((0, 0, d - 1, d - 1), fill=BADGE)
    blue = Image.new("RGBA", (d, d), PLATE)
    white = Image.new("RGBA", (d, d), BADGE)
    tools = round(d * 0.98)
    at = ((d - tools) // 2, (d - tools) // 2)
    under = screwdriver(tools).rotate(-45, resample=Image.BICUBIC)
    over = wrench(tools).rotate(45, resample=Image.BICUBIC)
    # A white gap round the wrench where it crosses the screwdriver.
    gap = over.filter(ImageFilter.MaxFilter(2 * max(1, d // 90) + 1))
    for mask, colour in ((under, blue), (gap, white), (over, blue)):
        layer = Image.new("L", (d, d), 0)
        layer.paste(mask, at)
        out.paste(colour, (0, 0), layer)
    return out


def render(size: int) -> Image.Image:
    big = size * SUPERSAMPLE
    plate = Image.new("RGBA", (big, big), (0, 0, 0, 0))
    ImageDraw.Draw(plate).rounded_rectangle(
        (0, 0, big - 1, big - 1), radius=round(big * RADIUS), fill=PLATE
    )
    globe = Image.open(GLOBE).convert("RGBA")
    side = round(big * GLOBE_SIZE)
    globe = globe.resize((side, round(side * globe.height / globe.width)), Image.LANCZOS)
    centre = round(big * GLOBE_CENTRE)
    plate.alpha_composite(globe, (centre - globe.width // 2, centre - globe.height // 2))
    disc = badge(round(big * BADGE_SIZE))
    centre = round(big * BADGE_CENTRE)
    plate.alpha_composite(disc, (centre - disc.width // 2, centre - disc.height // 2))
    return plate.resize((size, size), Image.LANCZOS)


def png(image: Image.Image) -> bytes:
    out = io.BytesIO()
    image.save(out, format="PNG", optimize=True)
    return out.getvalue()


def outputs() -> dict[pathlib.Path, Image.Image]:
    return {
        CIB / "workbench-icon-1024.png": render(1024),
        CIB / "workbench-icon-64.png": render(64),
    }


# How far a channel may differ between this machine's drawing and the file's.
# The check compares pixels, not bytes: Pillow's wheels compress PNGs
# differently on Windows and Linux, so the same drawing is not the same file.
TOLERANCE = 2


def matches(path: pathlib.Path, image: Image.Image) -> bool:
    if not path.exists():
        return False
    committed = Image.open(path).convert("RGBA")
    if committed.size != image.size:
        return False
    diff = ImageChops.difference(committed, image).getextrema()
    return all(high <= TOLERANCE for _, high in diff)


def main(argv: list[str]) -> int:
    check = "--check" in argv
    stale = []
    for path, image in outputs().items():
        if check:
            if not matches(path, image):
                stale.append(path.relative_to(ROOT).as_posix())
        else:
            path.write_bytes(png(image))
            print(path.relative_to(ROOT).as_posix())
    if stale:
        print("out of date, run python3 tools/make-workbench-icon.py:", *stale, sep="\n  ")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
