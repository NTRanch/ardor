package ai.ntranch.ardor.qa

/**
 * Deterministic playtest replay — a line-for-line port of the
 * Rust server's `generate_playtest` (ardor/src/playtest.rs), which
 * is the spec of record. For the same project documents + seed +
 * step budget it produces the same step list and the same run
 * hash: SHA-256 over the canonical JSON of {seed, start, steps}.
 *
 * The walk: scenes follow `next`; dialogues with choices draw
 * one choice from the SplitMix64 stream. Bounded by the step
 * budget and a per-node visit cap; the stop reason records why
 * the walk ended.
 */
object PlaytestReplay {
    const val MAX_STEPS: Int = 10_000
    const val DEFAULT_STEPS: Int = 512
    const val NODE_VISIT_CAP: Int = 8
    const val TERMINAL: String = "END"

    data class Result(
        val seed: ULong,
        val start: String,
        val steps: List<JsonObj>,
        val truncated: Boolean,
        val stopReason: String,
        val hash: String,
    ) {
        val stepsCount: Int get() = steps.size
    }

    fun generate(bundle: ProjectBundle, seed: ULong, maxSteps: Int = DEFAULT_STEPS): Result {
        val budget = maxSteps.coerceIn(1, MAX_STEPS)
        val graph = bundle.doc("graph")
        val start = bundle.meta.str("start_node") ?: ""
        val nodes = graph.arr("nodes")
        val rng = SplitMix64(seed)
        val steps = ArrayList<JsonObj>()
        val visits = HashMap<String, Int>()
        var current = start
        var stopReason = "terminal"

        walk@ while (true) {
            if (steps.size >= budget) {
                stopReason = "step_budget"
                break
            }
            if (current == TERMINAL || current.isEmpty()) {
                stopReason = "terminal"
                break
            }
            val node = nodes.firstOrNull {
                (it as? JsonObj)?.str("id") == current
            } as? JsonObj
            if (node == null) {
                stopReason = "missing_node"
                break
            }
            val count = (visits[current] ?: 0) + 1
            visits[current] = count
            if (count > NODE_VISIT_CAP) {
                stopReason = "loop_guard"
                break
            }
            val kind = node.str("kind") ?: "scene"
            val stepIndex = steps.size
            var next: String? = null
            var choice: JsonValue = JsonNull
            if (kind == "dialogue") {
                val choices = node.arr("choices")
                if (choices.isEmpty()) {
                    next = node.str("next")
                } else {
                    val pick = rng.below(choices.size)
                    val chosen = choices[pick].asObjOrEmpty()
                    next = chosen.str("next")
                    choice = JsonObj(
                        mapOf(
                            "index" to JsonLong(pick.toLong()),
                            "label" to JsonStr(chosen.str("label") ?: ""),
                        ),
                    )
                }
            } else {
                next = node.str("next")
            }
            steps.add(
                JsonObj(
                    mapOf(
                        "i" to JsonLong(stepIndex.toLong()),
                        "node" to JsonStr(current),
                        "kind" to JsonStr(kind),
                        "choice" to choice,
                        "next" to (next?.let { JsonStr(it) } ?: JsonNull),
                    ),
                ),
            )
            if (next == null) {
                stopReason = "terminal"
                break@walk
            }
            current = next
        }

        val truncated = stopReason != "terminal"
        val hash = sha256Hex(
            canonicalJson(
                JsonObj(
                    mapOf(
                        "seed" to JsonULong(seed),
                        "start" to JsonStr(start),
                        "steps" to JsonArr(steps),
                    ),
                ),
            ),
        )
        return Result(
            seed = seed,
            start = start,
            steps = steps,
            truncated = truncated,
            stopReason = stopReason,
            hash = hash,
        )
    }
}
