#!/usr/bin/env bash
# Capture the unmodified, pinned active candidate, never the preserved root 2D app.
set -euo pipefail
root=$(git rev-parse --show-toplevel)
source="$root/foundation/subcraft"
out="${RUNNER_TEMP:?}/skinstrike-active-3d"
mkdir -p "$out"
export SKINSTRIKE_CANDIDATE_HOME="$out/isolated-home"
export SUBCRAFT_DATA="$SKINSTRIKE_CANDIDATE_HOME/data"
export SUBCRAFT_LIBRARY="$SKINSTRIKE_CANDIDATE_HOME/library"
export XDG_CONFIG_HOME="$out/config"
export XDG_RUNTIME_DIR="$out/runtime"
mkdir -p "$SUBCRAFT_DATA" "$SUBCRAFT_LIBRARY" "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
export WGPU_BACKEND=vulkan
export VK_ICD_FILENAMES="$(find /usr/share/vulkan/icd.d -name '*lvp*' -print -quit)"
test -f "$VK_ICD_FILENAMES"
export LIBGL_ALWAYS_SOFTWARE=1
export RAYON_NUM_THREADS=1
binary="$root/.candidate/capture-target/debug/subcraft"
test -x "$binary"
test "$(git rev-parse HEAD)" = "${EXPECTED_HEAD:?}"
python3 - "$root" "$source" <<'PY'
import json,subprocess,sys
from pathlib import Path
root,source=map(Path,sys.argv[1:])
contract=json.loads((root/'foundation/candidate.json').read_text(encoding='utf-8'))
actual=subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD'],text=True).strip()
assert actual==contract['candidate_commit']=='85d35ed6ad74f98aa79f5c2d931ded104ebc05df', 'Unexpected active source'
assert not subprocess.check_output(['git','-C',str(source),'status','--porcelain'],text=True).strip(), 'Dirty candidate source'
PY
# This is the app's existing native viewport screenshot hook and its built-in procedural sample.
timeout 180s xvfb-run -a -s '-screen 0 1700x1100x24' "$binary" --demo --screenshot "$out/skinstrike-active-3d-demo.png" > "$out/capture.log" 2>&1
python3 - "$root" "$source" "$binary" "$out" <<'PY'
from pathlib import Path
from hashlib import sha256
import json,os,subprocess,sys
from PIL import Image
root,source,binary,out=map(Path,sys.argv[1:])
image_path=out/'skinstrike-active-3d-demo.png'
with Image.open(image_path) as image:
    image.load()
    assert image.width>=1200 and image.height>=700, 'Unexpected capture size'
    assert len(image.convert('RGB').getcolors(image.width*image.height) or [])>64, 'Capture lacks visible content'
    dimensions=list(image.size)
record={'parent_commit':subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip(),'foundation_commit':subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD'],text=True).strip(),'binary_sha256':sha256(binary.read_bytes()).hexdigest(),'screenshot':image_path.name,'screenshot_sha256':sha256(image_path.read_bytes()).hexdigest(),'dimensions':dimensions,'command':['subcraft','--demo','--screenshot',image_path.name],'feature':'skinstrike-candidate','renderer':'Mesa software Vulkan under Xvfb','scene':'Unmodified built-in procedural demo on the sample crate; not CS2 game assets','workflow_run_id':os.environ.get('GITHUB_RUN_ID'),'native_ui_visual_review':'pending; numeric checks do not establish visual acceptance'}
(out/'provenance.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print(json.dumps(record,indent=2))
PY
