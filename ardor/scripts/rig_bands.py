# rig_bands.py — ardor MCP rig_measure_bands instrument.
# Consolidated from band_at_row.py (moondeep-party-rigs build/):
# the L-15/L-27 SUPPLEMENTARY Step 0 band measurement. The fleet
# battery recovers z_scale as height/1.68, which is biased x1.0119
# (the artefact's true max z is the hair-cap pole), so its band
# centres land high and, for z_scale >~ 1.02, the bust band misses
# the bust row entirely and reads 0. This instrument replicates
# measure_battery's band_circ EXACTLY (same torso-vertex population
# -- dominant bone in {Hips, Spine, Spine1, Spine2} at weight > 0.45
# on the rest mesh -- same +-0.015 window, same Ramanujan ellipse)
# with band centres placed at the artefact's true section rows:
# row_z (unscaled frame) x the build record's z_scale, supplied by
# the caller. The fleet battery itself is NOT modified (L-15); the
# z_scale repair is referred to the canonical-base programme (L-27).
# Generalization over the lane script: arbitrary named rows, plus
# the L-44 distribution probe per row -- population counts in the
# band window and in the two adjacent windows, and the nearest
# populated row centre -- so an empty band reads as an instrument
# miss with evidence, never as a circumference of 0.
# Run: blender -b -Y --factory-startup --offline-mode -noaudio
#      -P rig_bands.py -- <glb> <z_scale> <rows.json> <out.json>
# rows.json: [{"name": "bust", "row_z": 1.235}, ...] (unscaled frame)
import bpy, json, math, sys
from mathutils import Vector

argv = sys.argv[sys.argv.index("--") + 1:]
GLB, ZS, ROWS_P, OUT = argv[0], float(argv[1]), argv[2], argv[3]
ROWS = json.load(open(ROWS_P))

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=GLB)
meshes = [o for o in bpy.data.objects if o.type == 'MESH']
body = max((o for o in meshes if o.vertex_groups), key=lambda o: len(o.data.vertices))
dg = bpy.context.evaluated_depsgraph_get()

def eval_verts(obj):
    ev = obj.evaluated_get(dg)
    me = ev.to_mesh()
    vs = [Vector(v.co) for v in me.vertices]
    ev.to_mesh_clear()
    return vs

names = [g.name for g in body.vertex_groups]
TORSO_BONES = {'mixamorig:Hips', 'mixamorig:Spine', 'mixamorig:Spine1', 'mixamorig:Spine2'}
dom = {}
for v in body.data.vertices:
    best, bw = None, 0.45
    for g in v.groups:
        if g.weight > bw:
            bw, best = g.weight, names[g.group]
    if best:
        dom.setdefault(best, []).append(v.index)
REST_V = eval_verts(body)
TORSO_V = [REST_V[i] for b in TORSO_BONES for i in dom.get(b, [])]

def band_circ(verts, zc, half=0.015):
    xs = [v.x for v in verts if abs(v.z - zc) <= half]
    ys = [v.y for v in verts if abs(v.z - zc) <= half]
    if not xs:
        return None
    a = (max(xs) - min(xs)) / 2.0
    b = (max(ys) - min(ys)) / 2.0
    h = ((a - b) / (a + b)) ** 2
    return math.pi * (a + b) * (1 + 3 * h / (10 + math.sqrt(4 - 3 * h)))

def window_count(verts, lo, hi):
    return sum(1 for v in verts if lo <= v.z <= hi)

# z histogram of the torso population (5 mm bins) for the L-44 probe
torso_zs = sorted(v.z for v in TORSO_V)
def nearest_populated(zc):
    if not torso_zs:
        return None
    best = min(torso_zs, key=lambda z: abs(z - zc))
    return round(best, 4)

res = {"glb": GLB, "z_scale_build_record": ZS, "rows": {}}
for row in ROWS:
    name, row_z = row["name"], float(row["row_z"])
    zc = row_z * ZS
    circ = band_circ(TORSO_V, zc)
    entry = {
        "row_z_unscaled": row_z,
        "centre_z": round(zc, 4),
        "circ_m": round(circ, 4) if circ is not None else None,
        "band_empty": circ is None,
        "population_in_window": window_count(TORSO_V, zc - 0.015, zc + 0.015),
        "population_below_window": window_count(TORSO_V, zc - 0.045, zc - 0.015),
        "population_above_window": window_count(TORSO_V, zc + 0.015, zc + 0.045),
        "nearest_populated_z": nearest_populated(zc),
    }
    res["rows"][name] = entry
    # record-comparable headline keys for the Fleet-A section rows
    if name in ("bust", "waist", "hip"):
        res[name + "_circ_at_row_m"] = round(circ or 0, 4)
with open(OUT, "w") as f:
    json.dump(res, f, indent=1)
print("BAND-AT-ROW DONE", OUT)
