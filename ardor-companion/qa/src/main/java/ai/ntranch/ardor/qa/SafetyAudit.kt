package ai.ntranch.ardor.qa

/** One named check with its outcome and the server's exact detail wording. */
data class QaCheck(
    val name: String,
    val ok: Boolean,
    val detail: String,
) {
    fun toJson(): JsonObj = JsonObj(
        mapOf(
            "check" to JsonStr(name),
            "ok" to JsonBool(ok),
            "detail" to JsonStr(detail),
        ),
    )
}

/**
 * The safety audit — a port of the Rust server's `safety_audit`
 * (ardor/src/gate.rs), which is the spec of record: all five
 * safety surfaces present, enabled, and ungated; an adult-rated
 * project carries an age-gate pattern. Check names and detail
 * strings match the server's audit output exactly, so a device
 * report and a server report can be diffed line by line.
 */
object SafetyAudit {
    /** The five surfaces, in the server's SAFETY_SURFACES order. */
    val SURFACES: List<String> = listOf("accessibility", "aftercare", "purge", "safeword", "stop")

    data class Result(
        val ok: Boolean,
        val checks: List<QaCheck>,
    ) {
        fun toJson(): JsonObj = JsonObj(
            mapOf(
                "ok" to JsonBool(ok),
                "checks" to JsonArr(checks.map { it.toJson() }),
            ),
        )
    }

    fun audit(bundle: ProjectBundle): Result {
        val checks = ArrayList<QaCheck>()
        val surfaces = bundle.doc("safety").arr("surfaces")
        for (name in SURFACES) {
            val found = surfaces.firstOrNull { (it as? JsonObj)?.str("id") == name } as? JsonObj
            val (ok, detail) = when {
                found == null -> false to "surface missing"
                (found["enabled"] as? JsonBool)?.value != true -> false to "surface disabled"
                (found["progression_gated"] as? JsonBool)?.value == true ->
                    false to "surface progression-gated"
                else -> true to "present, enabled, ungated"
            }
            checks.add(QaCheck("safety_surface:$name", ok, detail))
        }
        ageGateCheck(bundle)?.let { checks.add(it) }
        return Result(checks.all { it.ok }, checks)
    }

    /**
     * The age-gate check on its own — null when the project is
     * not adult-rated (the server only runs it then). The rating
     * defaults to "adult" when absent, as on the server.
     */
    fun ageGateCheck(bundle: ProjectBundle): QaCheck? {
        val rating = bundle.meta.str("rating") ?: "adult"
        if (rating != "adult") return null
        val ageGate = bundle.meta.str("age_gate") ?: ""
        return QaCheck(
            name = "age_gate_present",
            ok = ageGate.isNotEmpty(),
            detail = if (ageGate.isEmpty()) {
                "adult-rated project has no age-gate pattern"
            } else {
                "age-gate pattern recorded"
            },
        )
    }
}
