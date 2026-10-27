package ai.ntranch.ardor.qa

/**
 * Android channel readiness, device side.
 *
 * For every prepared handoff in the bundle's releases document
 * that targets one of the registry's Android channels (channels
 * whose platform list includes Android — ardor/src/channels.rs:
 * nutaku, itch_io, fdroid_self_hosted), this checks whether the
 * handoff's artifact expectations hold here on the device:
 *
 * - the channel expects an Android package among the artifacts
 *   (all three channels' recorded artifact requirements demand a
 *   signed APK / Android APK upload), and
 * - every recorded artifact is present, and its bytes hash to
 *   the SHA-256 recorded in the handoff.
 *
 * This PREPARES evidence for slipway: it reports whether the
 * artifacts match the prepared handoff record. It issues no
 * certification and forks none of slipway's publish gates.
 */
object ChannelReadiness {
    /** Registry channels whose platform list includes Android. */
    val ANDROID_CHANNELS: Set<String> = setOf("nutaku", "itch_io", "fdroid_self_hosted")

    data class ArtifactResult(
        val name: String,
        val kind: String,
        val present: Boolean,
        val digestMatches: Boolean,
        val recordedSha256: String,
        val actualSha256: String?,
    ) {
        val ok: Boolean get() = present && digestMatches

        fun toJson(): JsonObj = JsonObj(
            mapOf(
                "name" to JsonStr(name),
                "kind" to JsonStr(kind),
                "present" to JsonBool(present),
                "digest_matches" to JsonBool(digestMatches),
                "recorded_sha256" to JsonStr(recordedSha256),
                "actual_sha256" to (actualSha256?.let { JsonStr(it) } ?: JsonNull),
            ),
        )
    }

    data class HandoffResult(
        val handoffId: String,
        val channel: String,
        val executor: String,
        val expectsAndroidPackage: Boolean,
        val hasAndroidPackage: Boolean,
        val artifacts: List<ArtifactResult>,
    ) {
        val ok: Boolean
            get() = (!expectsAndroidPackage || hasAndroidPackage) && artifacts.all { it.ok }

        fun toJson(): JsonObj = JsonObj(
            mapOf(
                "handoff" to JsonStr(handoffId),
                "channel" to JsonStr(channel),
                "executor" to JsonStr(executor),
                "expects_android_package" to JsonBool(expectsAndroidPackage),
                "has_android_package" to JsonBool(hasAndroidPackage),
                "artifacts" to JsonArr(artifacts.map { it.toJson() }),
                "ok" to JsonBool(ok),
            ),
        )
    }

    data class Result(
        val handoffs: List<HandoffResult>,
    ) {
        /** True when every Android-channel handoff holds (vacuously true with none). */
        val ok: Boolean get() = handoffs.all { it.ok }

        fun toJson(): JsonObj = JsonObj(
            mapOf(
                "ok" to JsonBool(ok),
                "handoffs" to JsonArr(handoffs.map { it.toJson() }),
            ),
        )
    }

    /**
     * @param artifactBytes resolves an artifact name to its bytes
     * on this device, or null when the artifact is absent.
     */
    fun check(bundle: ProjectBundle, artifactBytes: (String) -> ByteArray?): Result {
        val handoffs = bundle.doc("releases").arr("handoffs")
        val results = ArrayList<HandoffResult>()
        for (raw in handoffs) {
            val handoff = raw.asObjOrEmpty()
            val channel = handoff.str("channel") ?: continue
            if (channel !in ANDROID_CHANNELS) continue
            val artifacts = handoff.arr("artifacts").map { rawArtifact ->
                val artifact = rawArtifact.asObjOrEmpty()
                val name = artifact.str("name") ?: ""
                val recorded = artifact.str("sha256") ?: ""
                val bytes = if (name.isEmpty()) null else artifactBytes(name)
                val actual = bytes?.let { sha256Hex(it) }
                ArtifactResult(
                    name = name,
                    kind = artifact.str("kind") ?: "",
                    present = bytes != null,
                    digestMatches = actual != null && actual == recorded.lowercase(),
                    recordedSha256 = recorded,
                    actualSha256 = actual,
                )
            }
            results.add(
                HandoffResult(
                    handoffId = handoff.str("id") ?: "",
                    channel = channel,
                    executor = handoff.str("executor") ?: "",
                    expectsAndroidPackage = true,
                    hasAndroidPackage = artifacts.any { it.name.endsWith(".apk", ignoreCase = true) },
                    artifacts = artifacts,
                ),
            )
        }
        return Result(results)
    }
}
