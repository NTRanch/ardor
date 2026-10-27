package ai.ntranch.ardor.qa

/**
 * The on-device QA battery over one exported Ardor project
 * bundle. Runs the safety audit, the age-gate item, the
 * deterministic replay (checked against the run the server
 * recorded in qa.json, when one exists), and the Android
 * channel-readiness evidence — and returns one structured
 * report. No check here mutates anything.
 */
object QaRunner {
    data class Report(
        val project: String,
        val safety: SafetyAudit.Result,
        val ageGate: QaCheck?,
        val replay: ReplayOutcome,
        val readiness: ChannelReadiness.Result,
    ) {
        val ok: Boolean
            get() = safety.ok &&
                (ageGate?.ok ?: true) &&
                replay.matchesRecorded != false &&
                readiness.ok

        fun summaryLines(): List<String> {
            val lines = ArrayList<String>()
            for (check in safety.checks) {
                lines.add("${if (check.ok) "PASS" else "FAIL"} ${check.name} — ${check.detail}")
            }
            if (ageGate != null) {
                lines.add("${if (ageGate.ok) "PASS" else "FAIL"} age_gate (reported item) — ${ageGate.detail}")
            }
            when (replay.matchesRecorded) {
                true -> lines.add("PASS replay_recorded_hash — ${replay.result.hash}")
                false -> lines.add(
                    "FAIL replay_recorded_hash — recomputed ${replay.result.hash}, " +
                        "recorded ${replay.recordedHash}",
                )
                null -> lines.add(
                    "PASS replay_deterministic — ${replay.result.hash} (no recorded run in bundle)",
                )
            }
            if (readiness.handoffs.isEmpty()) {
                lines.add("PASS channel_readiness — no Android-channel handoffs in bundle")
            } else {
                for (handoff in readiness.handoffs) {
                    lines.add(
                        "${if (handoff.ok) "PASS" else "FAIL"} channel_readiness " +
                            "${handoff.channel} (${handoff.handoffId})",
                    )
                    for (artifact in handoff.artifacts) {
                        val state = when {
                            !artifact.present -> "missing"
                            !artifact.digestMatches -> "digest mismatch"
                            else -> "present, digest matches"
                        }
                        lines.add(
                            "${if (artifact.ok) "PASS" else "FAIL"} artifact " +
                                "${handoff.channel}/${artifact.name} — $state",
                        )
                    }
                }
            }
            return lines
        }

        fun toJson(): JsonObj = JsonObj(
            mapOf(
                "project" to JsonStr(project),
                "ok" to JsonBool(ok),
                "safety_audit" to safety.toJson(),
                "age_gate" to (ageGate?.toJson() ?: JsonNull),
                "replay" to JsonObj(
                    mapOf(
                        "seed" to JsonULong(replay.result.seed),
                        "start" to JsonStr(replay.result.start),
                        "steps_count" to JsonLong(replay.result.stepsCount.toLong()),
                        "stop_reason" to JsonStr(replay.result.stopReason),
                        "truncated" to JsonBool(replay.result.truncated),
                        "hash" to JsonStr(replay.result.hash),
                        "recorded_hash" to (replay.recordedHash?.let { JsonStr(it) } ?: JsonNull),
                        "matches_recorded" to (
                            replay.matchesRecorded?.let { JsonBool(it) } ?: JsonNull
                            ),
                    ),
                ),
                "channel_readiness" to readiness.toJson(),
            ),
        )
    }

    data class ReplayOutcome(
        val result: PlaytestReplay.Result,
        val recordedHash: String?,
        /** null when the bundle records no run to compare against. */
        val matchesRecorded: Boolean?,
    )

    /**
     * @param artifactBytes resolves handoff artifact names to
     * bytes on this device (null = absent), for readiness.
     */
    fun run(bundle: ProjectBundle, artifactBytes: (String) -> ByteArray?): Report {
        val safety = SafetyAudit.audit(bundle)
        val ageGate = SafetyAudit.ageGateCheck(bundle)

        // Replay the latest run the server recorded, if any, so
        // the on-device hash is compared against the server's own.
        val recordedRun = bundle.doc("qa").arr("runs")
            .lastOrNull()?.asObjOrEmpty()
        val seed = (recordedRun?.get("seed") as? JsonLong)?.value?.toULong() ?: 7uL
        val maxSteps = (recordedRun?.get("max_steps") as? JsonLong)?.value?.toInt()
            ?: PlaytestReplay.DEFAULT_STEPS
        val replay = PlaytestReplay.generate(bundle, seed, maxSteps)
        val recordedHash = recordedRun?.str("hash")
        val outcome = ReplayOutcome(
            result = replay,
            recordedHash = recordedHash,
            matchesRecorded = recordedHash?.let { it == replay.hash },
        )

        val readiness = ChannelReadiness.check(bundle, artifactBytes)
        return Report(
            project = bundle.meta.str("slug") ?: "",
            safety = safety,
            ageGate = ageGate,
            replay = outcome,
            readiness = readiness,
        )
    }
}
