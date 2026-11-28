# rig_battery.py — ardor MCP rig_bend_battery instrument.
# Consolidated from the lane instruments measure_battery.py
# (torogao-rig-engine tools/ + lana-rung1/2 build/ copies): Step 0
# measures + weight-leak + the A10.1 bend battery on an exported GLB.
# The numeric core is byte-faithful to the lane instrument; the one
# addition is the L-29 direction column: evaluated joint world
# positions are recorded at rest and under every battery pose so the
# caller can read fold DIRECTIONS off coordinates, never off the
# battery's magnitudes alone (the sign law itself is per-rig).
# Contract (rig-lessons ledger): L-01 skeletal instrument is primary,
# centroid is reported and never gated (it is damped by construction
# on blended rigs); L-02 flexion is reported anatomically
# (180 - posed included angle) with rest included angles stated;
# L-27 the z_scale recovered here (height/1.68) is a diagnostic only
# -- band centres derived from it can miss on tall builds; the
# build-record z_scale governs (see rig_measure_bands).
# Run: blender -b -Y --factory-startup --offline-mode -noaudio
#      -P rig_battery.py -- <glb> <out.json> <frames_dir|none> <full|measure>
import bpy, json, math, sys
from mathutils import Vector

argv = sys.argv[sys.argv.index("--") + 1:]
GLB, OUT, FRAMES, MODE = argv[0], argv[1], argv[2], argv[3]

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=GLB)
arm = next(o for o in bpy.data.objects if o.type == 'ARMATURE')
meshes = [o for o in bpy.data.objects if o.type == 'MESH']
body = max((o for o in meshes if o.vertex_groups), key=lambda o: len(o.data.vertices))
hair = next((o for o in meshes if 'Hair' in o.name), None)
dg = bpy.context.evaluated_depsgraph_get()

def eval_verts(obj):
    ev = obj.evaluated_get(dg)
    me = ev.to_mesh()
    vs = [Vector(v.co) for v in me.vertices]
    ev.to_mesh_clear()
    return vs

def group_index(obj):
    """vert index -> dominant bone name (weight>0.45), from the REST mesh."""
    names = [g.name for g in obj.vertex_groups]
    dom = {}
    for v in obj.data.vertices:
        best, bw = None, 0.45
        for g in v.groups:
            if g.weight > bw:
                bw, best = g.weight, names[g.group]
        if best:
            dom.setdefault(best, []).append(v.index)
    return dom

DOM = group_index(body)
REST_V = eval_verts(body)

def centroid(idxs, verts):
    c = Vector((0, 0, 0))
    for i in idxs:
        c += verts[i]
    return c / max(1, len(idxs))

def band_circ(verts, zc, half=0.015):
    xs = [v.x for v in verts if abs(v.z - zc) <= half]
    ys = [v.y for v in verts if abs(v.z - zc) <= half]
    if not xs:
        return None
    a = (max(xs) - min(xs)) / 2.0
    b = (max(ys) - min(ys)) / 2.0
    h = ((a - b) / (a + b)) ** 2
    return math.pi * (a + b) * (1 + 3 * h / (10 + math.sqrt(4 - 3 * h)))

res = {"glb": GLB}
TORSO_BONES = {'mixamorig:Hips', 'mixamorig:Spine', 'mixamorig:Spine1', 'mixamorig:Spine2'}
TORSO_V = [REST_V[i] for b in TORSO_BONES for i in DOM.get(b, [])]

# ---------- Step 0 measures (rest) ----------
allv = []
for o in meshes:
    allv += eval_verts(o)
height = max(v.z for v in allv)
head_i = DOM.get('mixamorig:Head', [])
hv = [REST_V[i] for i in head_i]
crown = max(v.z for v in hv)
centre = [v.z for v in hv if abs(v.x) < 0.035]
chin = min(centre) if centre else min(v.z for v in hv)
head_h = crown - chin
nose = min((v.y for v in hv if abs(v.x) < 0.02 and v.z > chin + 0.6 * head_h), default=0.0)
front_neg_y = nose < 0
zs = height / 1.68  # per-girl z_scale, recovered from the artefact (L-27: diagnostic only)
res["step0"] = {
    "height_m": round(height, 4),
    "z_scale_recovered": round(zs, 4),
    "head_height_m": round(head_h, 4),
    "heads_tall": round(height / head_h, 3),
    "bust_circ_m": round(band_circ(TORSO_V, 1.235 * zs) or 0, 4),
    "waist_circ_m": round(band_circ(TORSO_V, 1.045 * zs) or 0, 4),
    "hip_circ_m": round(band_circ(TORSO_V, 0.905 * zs) or 0, 4),
    "front_is_negative_y": front_neg_y,
}
if hair is not None:
    hverts = eval_verts(hair)
    res["step0"]["hair_bottom_z"] = round(min(v.z for v in hverts), 4)
    res["step0"]["hair_below_shoulder_m"] = round(1.415 * zs - min(v.z for v in hverts), 4)
# ---------- weight leak: arm-bone weight on belly-region body verts ----------
belly = [v for v in REST_V if 1.00 * zs <= v.z <= 1.17 * zs and abs(v.x) < 0.11
         and (v.y < -0.03 if front_neg_y else v.y > 0.03)]
idx_of = {tuple(round(c, 5) for c in v): i for i, v in enumerate(REST_V)}
arm_bones = {'mixamorig:LeftArm', 'mixamorig:RightArm', 'mixamorig:LeftForeArm',
             'mixamorig:RightForeArm', 'mixamorig:LeftShoulder', 'mixamorig:RightShoulder'}
names = [g.name for g in body.vertex_groups]
tot, leak = 0.0, 0.0
for v in body.data.vertices:
    co = REST_V[v.index]
    if 1.00 * zs <= co.z <= 1.17 * zs and abs(co.x) < 0.11 and (co.y < -0.03 if front_neg_y else co.y > 0.03):
        wsum = sum(g.weight for g in v.groups)
        tot += wsum
        leak += sum(g.weight for g in v.groups if names[g.group] in arm_bones)
res["leak"] = {"belly_verts": len(belly), "arm_weight_share_pct": round(100.0 * leak / tot, 3) if tot else None}

if MODE == "full":
    # ---------- A10.1 battery ----------
    def reset_pose():
        for pb in arm.pose.bones:
            pb.rotation_mode = 'XYZ'
            pb.rotation_euler = (0, 0, 0)
            pb.location = (0, 0, 0)
        bpy.context.view_layer.update()

    def joint(name):
        return arm.pose.bones[name].matrix.translation.copy()

    def angle(a, b):
        if a.length == 0 or b.length == 0:
            return None
        return math.degrees(a.angle(b))

    def included(seg_a_idxs, seg_b_idxs, joint_name, verts):
        j = joint(joint_name)
        return angle(centroid(seg_a_idxs, verts) - j, centroid(seg_b_idxs, verts) - j)

    def seg_dir(joint_name, idxs, verts):
        return centroid(idxs, verts) - joint(joint_name)

    # L-29 direction column: evaluated joint world positions, rest + posed.
    DIR_JOINTS = ['mixamorig:Hips', 'mixamorig:LeftUpLeg', 'mixamorig:LeftLeg',
                  'mixamorig:LeftFoot', 'mixamorig:LeftShoulder', 'mixamorig:LeftArm',
                  'mixamorig:LeftForeArm', 'mixamorig:LeftHand', 'mixamorig:Head']
    def snap_joints():
        out = {}
        for n in DIR_JOINTS:
            if n in arm.pose.bones:
                t = arm.pose.bones[n].matrix.translation
                out[n] = [round(t.x, 5), round(t.y, 5), round(t.z, 5)]
        return out

    reset_pose()
    bpy.context.view_layer.update()
    rv = eval_verts(body)
    joints = {"rest": snap_joints(), "posed": {}}
    def J(name):
        return arm.pose.bones[name].matrix.translation.copy()
    def skel_inc(prox_bone, mid_bone, dist_bone):
        a, m, d = J(prox_bone), J(mid_bone), J(dist_bone)
        return angle(a - m, d - m)
    def skel_dir(b1, b2):
        return J(b2) - J(b1)
    rest = {
        "elbow_inc": included(DOM['mixamorig:LeftArm'], DOM['mixamorig:LeftForeArm'], 'mixamorig:LeftForeArm', rv),
        "knee_inc": included(DOM['mixamorig:LeftUpLeg'], DOM['mixamorig:LeftLeg'], 'mixamorig:LeftLeg', rv),
        "thigh_dir": seg_dir('mixamorig:Hips', DOM['mixamorig:LeftUpLeg'], rv),
        "arm_dir": seg_dir('mixamorig:LeftShoulder', DOM['mixamorig:LeftArm'], rv),
        "spine_dir": centroid(DOM['mixamorig:Head'], rv) - joint('mixamorig:Hips'),
        "elbow_skel": skel_inc('mixamorig:LeftShoulder', 'mixamorig:LeftForeArm', 'mixamorig:LeftHand'),
        "knee_skel": skel_inc('mixamorig:Hips', 'mixamorig:LeftLeg', 'mixamorig:LeftFoot'),
        "thigh_skel": skel_dir('mixamorig:LeftUpLeg', 'mixamorig:LeftLeg'),
        "arm_skel": skel_dir('mixamorig:LeftArm', 'mixamorig:LeftForeArm'),
        "spine_skel": skel_dir('mixamorig:Hips', 'mixamorig:Head'),
        "elbow_dist": angle(J('mixamorig:LeftShoulder') - J('mixamorig:LeftForeArm'), centroid(DOM['mixamorig:LeftHand'], rv) - J('mixamorig:LeftForeArm')),
        "head_q": arm.pose.bones['mixamorig:Head'].matrix.to_quaternion(),
        "knee_dist": angle(J('mixamorig:Hips') - J('mixamorig:LeftLeg'), centroid(DOM['mixamorig:LeftFoot'], rv) - J('mixamorig:LeftLeg')),
    }
    bat = {}
    def pose(bone, deg):
        pb = arm.pose.bones[bone]
        pb.rotation_euler = (math.radians(deg), 0, 0)
        bpy.context.view_layer.update()

    pose('mixamorig:LeftForeArm', -110)
    pv = eval_verts(body)
    joints["posed"]["elbow_flexion"] = snap_joints()
    inc = included(DOM['mixamorig:LeftArm'], DOM['mixamorig:LeftForeArm'], 'mixamorig:LeftForeArm', pv)
    sk = skel_inc('mixamorig:LeftShoulder', 'mixamorig:LeftForeArm', 'mixamorig:LeftHand')
    di = angle(J('mixamorig:LeftShoulder') - J('mixamorig:LeftForeArm'), centroid(DOM['mixamorig:LeftHand'], pv) - J('mixamorig:LeftForeArm'))
    bat["elbow_flexion"] = {"applied_deg": 110, "rest_included_deg": round(rest["elbow_skel"], 2),
        "achieved_skeletal_deg": round(180.0 - sk, 2),
        "achieved_distal_deg": round(180.0 - di, 2),
        "achieved_centroid_deg": round(rest["elbow_inc"] - inc, 2)}
    reset_pose()
    pose('mixamorig:LeftLeg', -100)
    pv = eval_verts(body)
    joints["posed"]["knee_flexion"] = snap_joints()
    inc = included(DOM['mixamorig:LeftUpLeg'], DOM['mixamorig:LeftLeg'], 'mixamorig:LeftLeg', pv)
    sk = skel_inc('mixamorig:Hips', 'mixamorig:LeftLeg', 'mixamorig:LeftFoot')
    di = angle(J('mixamorig:Hips') - J('mixamorig:LeftLeg'), centroid(DOM['mixamorig:LeftFoot'], pv) - J('mixamorig:LeftLeg'))
    bat["knee_flexion"] = {"applied_deg": 100, "rest_included_deg": round(rest["knee_skel"], 2),
        "achieved_skeletal_deg": round(180.0 - sk, 2),
        "achieved_distal_deg": round(180.0 - di, 2),
        "achieved_centroid_deg": round(rest["knee_inc"] - inc, 2)}
    reset_pose()
    pose('mixamorig:LeftUpLeg', -90)
    pv = eval_verts(body)
    joints["posed"]["hip_flexion"] = snap_joints()
    bat["hip_flexion"] = {"applied_deg": 90,
        "achieved_centroid_deg": round(angle(rest["thigh_dir"], seg_dir('mixamorig:Hips', DOM['mixamorig:LeftUpLeg'], pv)), 2),
        "achieved_skeletal_deg": round(angle(rest["thigh_skel"], skel_dir('mixamorig:LeftUpLeg', 'mixamorig:LeftLeg')), 2)}
    reset_pose()
    pose('mixamorig:LeftArm', -165)
    pv = eval_verts(body)
    joints["posed"]["shoulder_overhead"] = snap_joints()
    bat["shoulder_overhead"] = {"applied_deg": 165,
        "achieved_centroid_deg": round(angle(rest["arm_dir"], seg_dir('mixamorig:LeftShoulder', DOM['mixamorig:LeftArm'], pv)), 2),
        "achieved_skeletal_deg": round(angle(rest["arm_skel"], skel_dir('mixamorig:LeftArm', 'mixamorig:LeftForeArm')), 2)}
    reset_pose()
    pose('mixamorig:Spine', -15)
    pose('mixamorig:Spine2', -15)
    pv = eval_verts(body)
    joints["posed"]["spine_bend"] = snap_joints()
    hq = arm.pose.bones['mixamorig:Head'].matrix.to_quaternion()
    dq = rest["head_q"].inverted() @ hq
    bat["spine_bend"] = {"applied_deg": 30,
        "achieved_headframe_deg": round(math.degrees(2 * math.acos(max(-1.0, min(1.0, abs(dq.w))))), 2),
        "achieved_chord_deg": round(angle(rest["spine_skel"], skel_dir('mixamorig:Hips', 'mixamorig:Head')), 2),
        "achieved_centroid_deg": round(angle(rest["spine_dir"], centroid(DOM['mixamorig:Head'], pv) - joint('mixamorig:Hips')), 2)}
    reset_pose()
    res["battery"] = bat
    res["joints"] = joints

    # ---------- renders: rest + battery poses ----------
    if FRAMES != "none":
        import os
        os.makedirs(FRAMES, exist_ok=True)
        sc = bpy.context.scene
        sc.render.engine = 'BLENDER_EEVEE'
        sc.render.resolution_x, sc.render.resolution_y = 640, 768
        sc.render.film_transparent = False
        cam_d = bpy.data.cameras.new("cam")
        cam = bpy.data.objects.new("cam", cam_d)
        sc.collection.objects.link(cam)
        cam.location = (0, -2.55, 1.02)
        cam.rotation_euler = (Vector((0, 0, 0.94)) - cam.location).to_track_quat('-Z', 'Y').to_euler()
        sc.camera = cam
        for nm, loc, rot in (("sun", (1.5, -2.0, 3.0), (math.radians(55), 0, math.radians(-25))),
                             ("fill", (-2.0, -1.0, 1.5), (math.radians(70), 0, math.radians(140)))):
            ld = bpy.data.lights.new(nm, 'SUN')
            ld.energy = 3.0 if nm == "sun" else 1.2
            lo = bpy.data.objects.new(nm, ld)
            lo.rotation_euler = rot
            sc.collection.objects.link(lo)
        def shot(name):
            sc.render.filepath = os.path.join(FRAMES, name + ".png")
            bpy.ops.render.render(write_still=True)
        shot("rest_front")
        pose('mixamorig:LeftForeArm', -110); shot("pose_elbow110")
        reset_pose(); pose('mixamorig:LeftLeg', -100); shot("pose_knee100")
        reset_pose(); pose('mixamorig:LeftUpLeg', -90); shot("pose_hip90")
        reset_pose(); pose('mixamorig:LeftArm', -165); shot("pose_shoulder165")
        reset_pose(); pose('mixamorig:Spine', -15); pose('mixamorig:Spine2', -15); shot("pose_spine30")
        reset_pose()

with open(OUT, "w") as f:
    json.dump(res, f, indent=1)
print("MEASURE DONE", OUT)
