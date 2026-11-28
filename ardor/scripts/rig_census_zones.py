#!/usr/bin/env python3
"""rig_census_zones.py — ardor MCP rig_zone_census GLB zone instrument.
Consolidated unchanged from census_zones.py (fleet-zone-census
build/): the Fleet-A face-zone census from shipped GLBs.

Replicates the fleet builder's position-derived zone assignment
(build_character*.py build_head): zones are tested on the head-loft
ring verts AFTER sculpt, in scaled space, against RAW-frame literals:
  LIP:    |x|<=0.0195 and 1.5065<=z<=1.5225 and y<-0.042
  LID_*:  ellipse ((x -/+ ex)/0.020, (z-ez)/0.017) < 1.35 and y<-0.038,
          ez = scaled eye_z (builder scales face.eye_z by ZS at load),
          U if z>=ez else L
Expected populations derive from the artefact's own rings: the two
loft rings bracketing scaled mouth_z (front verts passing the LIP
x/y gates) and the rings bracketing scaled eye_z (verts passing the
lid ellipse) -- the rung-1 cure's derivation (L-34/L-40).
Contract (rig-lessons ledger): L-41 — this instrument replicates
the builder's position frame and inherits it; on CURED artefacts
(zones ring-derived at build) it can misreport (it read LIP 0 on
the cured Sable witness). Zone truth on a cured artefact is the
blend-side shape-key reading (rig_zone_truth_blend.py); this GLB
reading is the citable frame for shipped/uncured artefacts and
carries +/-1-vert COLOR-dust against blend-side truth.
GLB frame -> Blender frame: bx=gx, by=-gz, bz=gy.
Run: python3 rig_census_zones.py <glb> <spec.json>  (JSON to stdout)
"""
import json, struct, sys, math

def read_glb(path):
    data = open(path, 'rb').read()
    assert data[:4] == b'glTF'
    jlen = struct.unpack('<I', data[12:16])[0]
    js = json.loads(data[20:20+jlen])
    off = 20 + jlen
    blen = struct.unpack('<I', data[off:off+4])[0]
    binchunk = data[off+8:off+8+blen]
    return js, binchunk

def accessor(js, binchunk, idx):
    acc = js['accessors'][idx]
    bv = js['bufferViews'][acc['bufferView']]
    start = bv.get('byteOffset', 0) + acc.get('byteOffset', 0)
    n = acc['count']
    t = acc['type']
    comp = {'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4}[t]
    fmt = {5126:'f',5123:'H',5125:'I',5121:'B'}[acc['componentType']]
    stride = bv.get('byteStride')
    out = []
    esz = struct.calcsize(fmt)*comp
    for i in range(n):
        o = start + i*(stride if stride else esz)
        out.append(struct.unpack_from('<'+fmt*comp, binchunk, o))
    return out

def body_positions(js, binchunk):
    # Body mesh = the primitive carrying morph targets (fleet exports)
    for mesh in js['meshes']:
        for prim in mesh['primitives']:
            if 'targets' in prim:
                return accessor(js, binchunk, prim['attributes']['POSITION']), mesh.get('name','?')
    # fallback: largest position accessor
    best = None
    for mesh in js['meshes']:
        for prim in mesh['primitives']:
            pos = accessor(js, binchunk, prim['attributes']['POSITION'])
            if best is None or len(pos) > len(best[0]):
                best = (pos, mesh.get('name','?'))
    return best

def census(glb_path, spec_path):
    spec = json.load(open(spec_path))
    ZS = spec.get('z_scale', 1.0); RS = spec.get('radial_scale', 1.0)
    fc = spec['face']
    ex = fc['eye_x']*RS; ez = fc['eye_z']*ZS; mz = fc['mouth_z']*ZS
    js, binchunk = read_glb(glb_path)
    pos, meshname = body_positions(js, binchunk)
    raw = [(gx, -gz, gy) for (gx, gy, gz) in pos]  # -> Blender (x, y, z)
    # GLB export splits verts per face/material: dedupe by position
    seen = {}
    for v in raw:
        seen[(round(v[0],5), round(v[1],5), round(v[2],5))] = v
    verts = list(seen.values())
    # head loft rings from the spec's own sections (scaled)
    rings = []
    for sec in spec['head_sections']:
        rz = sec[0]*ZS; rx = sec[1]*RS; ry = sec[2]*RS; cy = sec[3]*RS
        n = sec[4] if len(sec) > 4 else 24
        def gauss(x, mu, sig):
            return math.exp(-0.5*((x-mu)/sig)**2)
        ring = []
        for v in verts:
            if abs(v[2]-rz) >= 0.0012:
                continue
            for k in range(n):
                th = 2.0*math.pi*k/n
                if abs(v[0]-rx*math.cos(th)) >= 0.0004:
                    continue
                thw = math.atan2(math.sin(th), math.cos(th))
                front = max(0.0, -math.sin(thw))
                y_ell = cy + ry*math.sin(th)
                dy = 0.0
                if front >= 0.25:
                    dy -= 0.0115*gauss(v[2], 1.554, 0.0095)*gauss(thw, -math.pi/2, 0.17)
                    dy -= 0.0035*gauss(v[2], 1.568, 0.010)*gauss(thw, -math.pi/2, 0.13)
                    dy -= 0.0038*gauss(v[2], mz, 0.006)*gauss(thw, -math.pi/2, 0.30)
                    dy -= 0.0042*gauss(v[2], 1.480, 0.006)*gauss(thw, -math.pi/2, 0.26)
                    for sd in (1.0, -1.0):
                        dd = math.hypot(v[0]-sd*ex, v[2]-ez)
                        dy += 0.0038*gauss(dd, 0.0, 0.014)*front
                if abs(v[1]-(y_ell+dy)) < 0.0012:
                    ring.append(v)
                    break
        rings.append((rz, n, ring))
    ringverts = [v for (_, _, r) in rings for v in r]
    zones = {'LIP':0, 'LID_U_L':0, 'LID_L_L':0, 'LID_U_R':0, 'LID_L_R':0, 'SKIN':0}
    for (x, y, z) in ringverts:
        zone = 'SKIN'
        for s, nm in ((1.0,'L'), (-1.0,'R')):
            dd = math.hypot((x - s*ex)/0.020, (z - ez)/0.017)
            if dd < 1.35 and y < -0.038:
                zone = ('LID_U_'+nm) if z >= ez else ('LID_L_'+nm)
        if abs(x) <= 0.0195 and 1.5065 <= z <= 1.5225 and y < -0.042:
            zone = 'LIP'
        zones[zone] += 1
    # expected: rings bracketing scaled mouth_z / eye_z
    def bracketing(target):
        below = [r for r in rings if r[0] <= target]
        above = [r for r in rings if r[0] > target]
        out = []
        if below: out.append(below[-1])
        if above: out.append(above[0])
        return out
    exp_lip = 0
    for (_, _, ring) in bracketing(mz):
        exp_lip += sum(1 for (x, y, z) in ring if abs(x) <= 0.0195 and y < -0.042)
    exp_lid = {'U':0, 'L':0}
    for (_, _, ring) in bracketing(ez):
        for (x, y, z) in ring:
            for s in (1.0, -1.0):
                dd = math.hypot((x - s*ex)/0.020, (z - ez)/0.017)
                if dd < 1.35 and y < -0.038:
                    exp_lid['U' if z >= ez else 'L'] += 1
    ringinfo = [(round(rz,4), n, len(r)) for (rz, n, r) in rings]
    return {'ZS': ZS, 'RS': RS, 'mesh': meshname, 'body_verts': len(verts),
            'ring_verts': len(ringverts), 'zones': zones,
            'exp_lip': exp_lip, 'exp_lid_per_side': {k: v//2 for k, v in exp_lid.items()},
            'mouth_z_scaled': round(mz,4), 'eye_z_scaled': round(ez,4), 'rings': ringinfo}

if __name__ == '__main__':
    print(json.dumps(census(sys.argv[1], sys.argv[2]), indent=1))
