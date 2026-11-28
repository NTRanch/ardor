#!/usr/bin/env python3
"""rig_census_glb.py — ardor MCP rig_zone_census GLB instrument.
Consolidated from census_glb.py (torogao-rig-engine tools/ +
moondeep-party-rigs build/): A10.2 structural census of a GLB, and
the one-body sameness proof for a pair. Plain-python GLB parse (no
Blender): joint lists, morph target names per mesh, per-mesh
accessor hashes. For the Body mesh of a pair: POSITION / JOINTS_0 /
WEIGHTS_0 byte-hash equality, and the COLOR_0/COLOR_1 difference
localised (differing-vertex count + bbox).
Contract (rig-lessons ledger): L-25 — colour differences are
compared as RAW BYTES per vertex; a float reinterpretation is
invalid (0xFFFFFFFF is a NaN and NaN != NaN would report every
vertex as differing). L-37 — structural identity is judged on
accessor bytes; renders are judged on pixels, never file md5
(this instrument emits no render verdicts).
Run: python3 rig_census_glb.py <a.glb> <out.json>
     python3 rig_census_glb.py <a.glb> <b.glb> <out.json>"""
import json, struct, sys, hashlib

def load(path):
    data = open(path, "rb").read()
    assert data[:4] == b"glTF", "not a GLB: " + path
    jlen = struct.unpack("<I", data[12:16])[0]
    gltf = json.loads(data[20:20 + jlen])
    off = 20 + jlen
    blen = struct.unpack("<I", data[off:off + 4])[0]
    binchunk = data[off + 8:off + 8 + blen]
    return gltf, binchunk

def accessor_bytes(gltf, binchunk, idx):
    acc = gltf["accessors"][idx]
    bv = gltf["bufferViews"][acc["bufferView"]]
    start = bv.get("byteOffset", 0) + acc.get("byteOffset", 0)
    comp = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}[acc["componentType"]]
    ncomp = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}[acc["type"]]
    n = acc["count"] * comp * ncomp
    return binchunk[start:start + n], acc

def census(path):
    gltf, binchunk = load(path)
    nodes = gltf["nodes"]
    joints = [nodes[j]["name"] for j in gltf["skins"][0]["joints"]]
    meshes = {}
    for m in gltf["meshes"]:
        prim = m["primitives"][0]
        entry = {"targets": (m.get("extras") or {}).get("targetNames"),
                 "verts": gltf["accessors"][prim["attributes"]["POSITION"]]["count"]}
        for attr in ("POSITION", "JOINTS_0", "WEIGHTS_0", "COLOR_0"):
            if attr in prim["attributes"]:
                raw, _ = accessor_bytes(gltf, binchunk, prim["attributes"][attr])
                entry[attr + "_sha256"] = hashlib.sha256(raw).hexdigest()
        meshes[m["name"]] = entry
    return {"path": path, "joints": joints, "meshes": meshes,
            "_raw": (gltf, binchunk)}

if len(sys.argv) == 3:
    a = census(sys.argv[1])
    del a["_raw"]
    out = {"a": a}
    with open(sys.argv[2], "w") as f:
        json.dump(out, f, indent=1)
    print(json.dumps({"joints": len(a["joints"]), "meshes": list(a["meshes"])}))
    sys.exit(0)

a = census(sys.argv[1])
b = census(sys.argv[2])

def body(g):
    for name, m in g["meshes"].items():
        if name.endswith("_Body"):
            return name, m
    raise KeyError("no _Body mesh")

an, am = body(a)
bn, bm = body(b)
proof = {
    "joint_sets_equal": a["joints"] == b["joints"],
    "joint_count": len(a["joints"]),
    "body_mesh": [an, bn],
    "body_verts": [am["verts"], bm["verts"]],
    "position_equal": am["POSITION_sha256"] == bm["POSITION_sha256"],
    "joints0_equal": am["JOINTS_0_sha256"] == bm["JOINTS_0_sha256"],
    "weights0_equal": am["WEIGHTS_0_sha256"] == bm["WEIGHTS_0_sha256"],
    "morph_targets_a": am["targets"],
    "morph_targets_b": bm["targets"],
    "morph_sets_equal": am["targets"] == bm["targets"],
}
# localise colour differences on the Body mesh — RAW BYTE comparison per
# vertex (a float reinterpretation is invalid: 0xFFFFFFFF is a NaN and
# NaN != NaN would report every vertex as differing).
ga, ba = a["_raw"]
gb, bb = b["_raw"]
def prim_of(gltf, name):
    for m in gltf["meshes"]:
        if m["name"] == name:
            return m["primitives"][0]
pa, pb = prim_of(ga, an), prim_of(gb, bn)
pos, pacc = accessor_bytes(ga, ba, pa["attributes"]["POSITION"])
import array
fp = array.array("f"); fp.frombytes(pos)
for cname in ("COLOR_0", "COLOR_1"):
    if cname not in pa["attributes"]:
        continue
    ca, cacc = accessor_bytes(ga, ba, pa["attributes"][cname])
    cb, _ = accessor_bytes(gb, bb, pb["attributes"][cname])
    comp = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}[cacc["componentType"]]
    ncomp = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}[cacc["type"]]
    stride = comp * ncomp
    diff_idx = [i for i in range(cacc["count"])
                if ca[i * stride:(i + 1) * stride] != cb[i * stride:(i + 1) * stride]]
    key = cname.lower() + "_diff"
    if diff_idx:
        # GLB space: y is up
        xs = [fp[i * 3] for i in diff_idx]; ys = [fp[i * 3 + 1] for i in diff_idx]
        zs = [fp[i * 3 + 2] for i in diff_idx]
        proof[key] = {"verts": len(diff_idx),
                      "bbox_glb_xyz": {"x": [min(xs), max(xs)], "up_y": [min(ys), max(ys)],
                                       "z": [min(zs), max(zs)]}}
    else:
        proof[key] = {"verts": 0}
for g in (a, b):
    del g["_raw"]
out = {"a": a, "b": b, "one_body_proof": proof}
with open(sys.argv[3], "w") as f:
    json.dump(out, f, indent=1)
print(json.dumps(proof, indent=1))
