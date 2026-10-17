#!/usr/bin/env bash
set -euo pipefail

root="${RUNNER_TEMP}/skinstrike-native-visual"
library="${root}/library"
mkdir -p "${library}/Favorites" "${library}/Studies"
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER=llvmpipe
export XDG_CONFIG_HOME="${root}/config"

python3 - "$library" <<'PY'
import pathlib
import struct
import sys
import zlib

root = pathlib.Path(sys.argv[1])
palette = [(191, 89, 54), (45, 112, 178), (58, 149, 117), (213, 151, 54)]
def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)
for index, base in enumerate(palette, 1):
    rows = bytearray()
    for y in range(64):
        rows.append(0)
        for x in range(64):
            rows.extend((min(255, base[0] + x // 3), min(255, base[1] + y // 3), base[2], 255))
    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", 64, 64, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(rows))) + chunk(b"IEND", b"")
    (root / ("Favorites" if index < 3 else "Studies") / f"base-{index}.png").write_bytes(png)
PY

cli="./target/debug/skinstrike"
native="./target/debug/skinstrike-ui"
for index in 1 2 3 4; do
  if (( index < 3 )); then collection="Favorites"; else collection="Studies"; fi
  folder="${library}/${collection}"
  draft="${folder}/draft-${index}.json"
  "$cli" new "${folder}/base-${index}.png" "$draft"
  if [[ "$index" == 1 ]]; then "$cli" add-color "$draft" "Copper wash" 220 114 61 110; fi
  project_id=$(python3 - "$draft" <<'PY'
import json
import sys
print(json.load(open(sys.argv[1], encoding="utf-8"))["project_id"][:16])
PY
)
  title=("Copper Study" "Azure Study" "Verdant Study" "Golden Study")
  project="${folder}/${title[index-1]} -- ${project_id}.skinstrike.json"
  cp "$draft" "$project"
  if [[ "$index" == 1 ]]; then editor_project="$project"; fi
done

timeout 50s xvfb-run -a -s '-screen 0 1600x1000x24' "$native" --capture-library "${root}/gallery.png" "$library"
timeout 50s xvfb-run -a -s '-screen 0 1600x1000x24' "$native" --capture-project "${root}/editor.png" "$editor_project"

python3 - "$root" <<'PY'
import hashlib
import pathlib
import struct
import sys
import zlib

for name in ("gallery", "editor"):
    path = pathlib.Path(sys.argv[1]) / f"{name}.png"
    data = path.read_bytes()
    if len(data) < 5000 or not data.startswith(b"\x89PNG\r\n\x1a\n"):
        raise SystemExit(f"{name} native capture is missing or incomplete")
    width, height = struct.unpack(">II", data[16:24])
    if width < 960 or height < 620:
        raise SystemExit(f"{name} native capture has unexpected dimensions: {width}x{height}")
    offset = 8
    compressed = bytearray()
    while offset < len(data):
        length = struct.unpack(">I", data[offset:offset + 4])[0]
        kind = data[offset + 4:offset + 8]
        if kind == b"IDAT": compressed.extend(data[offset + 8:offset + 8 + length])
        offset += length + 12
    if len(set(zlib.decompress(compressed))) < 24:
        raise SystemExit(f"{name} native capture lacks visible content")
    print(f"{name}: {width}x{height}, {len(data)} bytes, SHA-256 {hashlib.sha256(data).hexdigest().upper()}")
PY
