# rig_census_blend.py — ardor MCP rig_zone_census blend-pair instrument.
# Consolidated unchanged from census_blend.py (moondeep-party-rigs
# build/): blend-side census for A10.2 — armature bone names + Body
# shape-key (morph) names for a pair of blends.
# Run: blender -b -Y --factory-startup --offline-mode -noaudio
#      -P rig_census_blend.py -- <a.blend> <b.blend> <out.json>
import bpy, json, sys

argv = sys.argv[sys.argv.index("--") + 1:]
res = {}
for path in argv[:2]:
    bpy.ops.wm.open_mainfile(filepath=path)
    arm = next(o for o in bpy.data.objects if o.type == 'ARMATURE')
    body = next(o for o in bpy.data.objects
                if o.type == 'MESH' and o.name.endswith('_Body'))
    keys = []
    if body.data.shape_keys:
        keys = [kb.name for kb in body.data.shape_keys.key_blocks
                if kb.name != 'Basis']
    res[path] = {"bones": sorted(b.name for b in arm.data.bones),
                 "bone_count": len(arm.data.bones),
                 "body_shape_keys": keys,
                 "objects": sorted(o.name for o in bpy.data.objects)}
with open(argv[2], "w") as f:
    json.dump(res, f, indent=1)
print("CENSUS DONE", argv[2])
