# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "pillow",
# ]
# ///


# Written by Chatgpt because I ceebsed doing this manually

import os
from PIL import Image

# List of colors to process
COLORS = ["blue", "green", "orange", "pink", "purple", "red", "yellow"]

SRC_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUTLINE_DIR = SRC_DIR


def make_outline(color):
    src_path = os.path.join(SRC_DIR, f"tile_{color}.png")
    out_path = os.path.join(OUTLINE_DIR, f"outline_{color}.png")
    img = Image.open(src_path).convert("RGBA")
    w, h = img.size
    pixels = img.load()
    outline_img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    outline_pixels = outline_img.load()
    for y in range(h):
        for x in range(w):
            if pixels[x, y][3] == 0:
                continue  # transparent, skip
            # Check if pixel is on the edge of the shape
            edge = False
            for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                nx, ny = x + dx, y + dy
                if nx < 0 or nx >= w or ny < 0 or ny >= h or pixels[nx, ny][3] == 0:
                    edge = True
                    break
            if edge:
                outline_pixels[x, y] = pixels[x, y]
    outline_img.save(out_path)
    print(f"Saved outline for {color} to {out_path}")


if __name__ == "__main__":
    for color in COLORS:
        make_outline(color)
