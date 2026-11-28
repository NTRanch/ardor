# rig_zone_truth_blend.py — ardor MCP rig_zone_census blend instrument.
# Consolidated unchanged from zone_truth_blend.py
# (moondeep-lip-cure build/): builder-truth zone census from a .blend.
# Run: blender -b -Y --factory-startup --offline-mode -noaudio
#        -P rig_zone_truth_blend.py -- <blend> <spec.json> <out.json>
# The builder's sculpt displaces y only, so a Body vert is a head-loft
# ring vert iff its (x, z) lies on a spec section ring's slot grid
# (exact test, no sculpt replication). Zone membership is read from
# the shape-key deltas the builder itself wrote:
#   LIP    : Fac_Mth_Aa delta at the scale-1.0 mouth pattern
#            (MOUTH_IN is x0.55, TEETH x0.25 -- magnitudes separate)
#   LID_U  : Fac_Eye_Angry dz != 0 (put on every upper-set vert)
#   LID_L  : Fac_Eye_LowlidUp dz != 0 (put on every lower-set vert)
# Lash verts share the lid witnesses but are not on ring slots, so
# the ring restriction separates LID from LASH exactly.
# Contract (rig-lessons ledger): L-34/L-41 — zones derive from the
# artefact's own rings; on cured artefacts THIS reading (the
# artefact's own morph deltas, exact ring-slot membership) is the
# zone truth, never a GLB position replication.
import bpy, json, math, sys

argv = sys.argv[sys.argv.index("--") + 1:]
BLEND, SPEC_P, OUT = argv[0], argv[1], argv[2]
spec = json.load(open(SPEC_P))
ZS = spec.get("z_scale", 1.0)
RS = spec.get("radial_scale", 1.0)
fc = spec["face"]
ex = fc["eye_x"] * RS
ez = fc["eye_z"] * ZS
mz = fc["mouth_z"] * ZS

bpy.ops.wm.open_mainfile(filepath=BLEND)
body = next(o for o in bpy.data.objects
            if o.type == 'MESH' and o.name.endswith('_Body'))
me = body.data
keys = {kb.name: kb for kb in me.shape_keys.key_blocks}
basis = keys['Basis'].data

def delta(name, i):
    return keys[name].data[i].co - basis[i].co

def sstep(a, b, x):
    if x <= a: return 0.0
    if x >= b: return 1.0
    t = (x - a) / (b - a)
    return t * t * (3 - 2 * t)

# ring slot grid: (z, {rounded x values}) per section
rings = []
for sec in spec["head_sections"]:
    rz = sec[0] * ZS
    rx = sec[1] * RS
    n = sec[4] if len(sec) > 4 else 24
    xs = {round(rx * math.cos(2.0 * math.pi * k / n), 6) for k in range(n)}
    rings.append((rz, xs))

def ring_of(v):
    for rz, xs in rings:
        if abs(v.z - rz) < 1e-6 and round(v.x, 6) in xs:
            return rz
    return None

res = {"blend": BLEND, "body_verts": len(me.vertices),
       "LIP": 0, "lip_pattern_ok": 0,
       "LID_U_L": 0, "LID_L_L": 0, "LID_U_R": 0, "LID_L_R": 0,
       "ring_verts": 0}
for i, v in enumerate(basis):
    co = v.co
    if ring_of(co) is None:
        continue
    res["ring_verts"] += 1
    aa = delta('Fac_Mth_Aa', i)
    if aa.length > 1e-7:
        cf = sstep(0.006, 0.016, abs(co.x))
        upper = co.z > mz
        exp_dz = 0.0026 if upper else -0.0092 * (1 - 0.45 * cf)
        if abs(aa.z - exp_dz) < 1e-5 and abs(aa.x) < 1e-5 and abs(aa.y) < 1e-5:
            res["LIP"] += 1
            res["lip_pattern_ok"] += 1
        else:
            res.setdefault("lip_pattern_mismatch", []).append(
                [round(co.x, 4), round(co.y, 4), round(co.z, 4),
                 round(aa.z, 5), round(exp_dz, 5)])
    side = 'L' if co.x > 0 else 'R'
    if abs(delta('Fac_Eye_Angry', i).z) > 1e-6:
        res['LID_U_' + side] += 1
    if abs(delta('Fac_Eye_LowlidUp', i).z) > 1e-6:
        res['LID_L_' + side] += 1
json.dump(res, open(OUT, 'w'), indent=1)
print("ZONE_TRUTH", json.dumps(res))
