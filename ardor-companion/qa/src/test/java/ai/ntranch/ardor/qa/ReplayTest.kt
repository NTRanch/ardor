package ai.ntranch.ardor.qa

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

/**
 * The cross-implementation contract: this Kotlin replay must
 * produce, for the D-116 evidence-game project, the exact run
 * hash the Rust server produced (seed 7, max_steps 32):
 * 14b271c23aef8468e568ee2785041d015be7a27a753fa9d62f0fa4aa59a088f8.
 */
class ReplayTest {
    companion object {
        const val SERVER_HASH = "14b271c23aef8468e568ee2785041d015be7a27a753fa9d62f0fa4aa59a088f8"
    }

    @Test
    fun `replay of the D-116 fixture reproduces the Rust server hash`() {
        val result = PlaytestReplay.generate(Fixtures.directoryBundle(), 7uL, 32)
        assertEquals(SERVER_HASH, result.hash)
        assertEquals("arrival", result.start)
        assertEquals(2, result.stepsCount)
        assertEquals("terminal", result.stopReason)
        assertEquals(false, result.truncated)
    }

    @Test
    fun `canonical form of the fixture run matches the server byte shape`() {
        val result = PlaytestReplay.generate(Fixtures.directoryBundle(), 7uL, 32)
        val canonical = canonicalJson(
            JsonObj(
                mapOf(
                    "seed" to JsonULong(7uL),
                    "start" to JsonStr("arrival"),
                    "steps" to JsonArr(result.steps),
                ),
            ),
        )
        assertEquals(
            """{"seed":7,"start":"arrival","steps":[""" +
                """{"choice":null,"i":0,"kind":"scene","next":"greeting","node":"arrival"},""" +
                """{"choice":{"index":0,"label":"Stay"},"i":1,"kind":"dialogue","next":"END","node":"greeting"}]}""",
            canonical,
        )
        assertEquals(SERVER_HASH, sha256Hex(canonical))
    }

    @Test
    fun `replay is deterministic across repeated runs`() {
        val bundle = Fixtures.directoryBundle()
        val a = PlaytestReplay.generate(bundle, 42uL, 512)
        val b = PlaytestReplay.generate(bundle, 42uL, 512)
        assertEquals(a.hash, b.hash)
        assertEquals(a.steps, b.steps)
    }

    @Test
    fun `splitmix64 stream matches independently computed vectors`() {
        val seed0 = SplitMix64(0uL)
        assertEquals(0xe220a8397b1dcdafuL, seed0.nextU64())
        assertEquals(0x6e789e6aa1b965f4uL, seed0.nextU64())
        assertEquals(0x06c45d188009454fuL, seed0.nextU64())
        val seed7 = SplitMix64(7uL)
        assertEquals(0x63cbe1e459320dd7uL, seed7.nextU64())
    }

    @Test
    fun `a different seed yields a different hash even on a one-choice graph`() {
        // The recorded seed is part of the hashed content, so the
        // hash must move even when the walked steps do not.
        val bundle = Fixtures.directoryBundle()
        val seed7 = PlaytestReplay.generate(bundle, 7uL, 32)
        val seed8 = PlaytestReplay.generate(bundle, 8uL, 32)
        assertEquals(seed7.steps, seed8.steps)
        assertNotEquals(seed7.hash, seed8.hash)
    }

    @Test
    fun `a tampered graph yields a different hash`() {
        val tampered = Fixtures.variantEditing("graph.json") { graph ->
            val nodes = graph.arr("nodes").map { raw ->
                val node = raw.asObjOrEmpty()
                if (node.str("id") == "greeting") {
                    val choices = node.arr("choices").map { rawChoice ->
                        val choice = rawChoice.asObjOrEmpty()
                        JsonObj(choice.fields + ("label" to JsonStr("Leave")))
                    }
                    JsonObj(node.fields + ("choices" to JsonArr(choices)))
                } else {
                    node
                }
            }
            JsonObj(graph.fields + ("nodes" to JsonArr(nodes)))
        }
        val result = PlaytestReplay.generate(tampered, 7uL, 32)
        assertNotEquals(SERVER_HASH, result.hash)
    }

    @Test
    fun `the loop guard bounds a cyclic graph`() {
        val cyclic = Fixtures.variant(
            "graph.json",
            """{"nodes":[{"id":"arrival","kind":"scene","next":"arrival"}]}""",
        )
        val result = PlaytestReplay.generate(cyclic, 1uL, 512)
        assertEquals("loop_guard", result.stopReason)
        assertTrue(result.truncated)
    }

    @Test
    fun `a zip bundle replays to the same hash as the directory bundle`() {
        val zipFile = File.createTempFile("evidence-game", ".zip")
        ZipOutputStream(zipFile.outputStream().buffered()).use { zip ->
            for ((name, text) in Fixtures.rawDocuments()) {
                zip.putNextEntry(ZipEntry("evidence-game/$name"))
                zip.write(text.toByteArray(Charsets.UTF_8))
                zip.closeEntry()
            }
        }
        val fromZip = PlaytestReplay.generate(ProjectBundle.load(ZipBundle(zipFile)), 7uL, 32)
        assertEquals(SERVER_HASH, fromZip.hash)
        zipFile.delete()
    }
}
