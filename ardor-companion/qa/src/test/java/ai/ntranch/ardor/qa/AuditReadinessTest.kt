package ai.ntranch.ardor.qa

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Safety audit + channel readiness: validation and adversarial cases. */
class AuditReadinessTest {

    // ---------- safety audit: validation ----------

    @Test
    fun `audit passes on the fixture with the server's exact check shape`() {
        val audit = SafetyAudit.audit(Fixtures.directoryBundle())
        assertTrue(audit.ok)
        assertEquals(
            listOf(
                "safety_surface:accessibility",
                "safety_surface:aftercare",
                "safety_surface:purge",
                "safety_surface:safeword",
                "safety_surface:stop",
                "age_gate_present",
            ),
            audit.checks.map { it.name },
        )
        assertTrue(audit.checks.all { it.ok })
    }

    @Test
    fun `age gate is reported as its own item`() {
        val check = SafetyAudit.ageGateCheck(Fixtures.directoryBundle())
        assertEquals("age_gate_present", check?.name)
        assertEquals(true, check?.ok)
        assertEquals("age-gate pattern recorded", check?.detail)
    }

    // ---------- safety audit: adversarial ----------

    @Test
    fun `audit refuses a progression-gated safety surface`() {
        val gated = Fixtures.variantEditing("safety.json") { safety ->
            val surfaces = safety.arr("surfaces").map { raw ->
                val surface = raw.asObjOrEmpty()
                if (surface.str("id") == "safeword") {
                    JsonObj(surface.fields + ("progression_gated" to JsonBool(true)))
                } else {
                    surface
                }
            }
            JsonObj(safety.fields + ("surfaces" to JsonArr(surfaces)))
        }
        val audit = SafetyAudit.audit(gated)
        assertFalse(audit.ok)
        val failed = audit.checks.single { it.name == "safety_surface:safeword" }
        assertFalse(failed.ok)
        assertEquals("surface progression-gated", failed.detail)
    }

    @Test
    fun `audit refuses a missing safety surface`() {
        val missing = Fixtures.variantEditing("safety.json") { safety ->
            val surfaces = safety.arr("surfaces").filter {
                (it as? JsonObj)?.str("id") != "stop"
            }
            JsonObj(safety.fields + ("surfaces" to JsonArr(surfaces)))
        }
        val audit = SafetyAudit.audit(missing)
        assertFalse(audit.ok)
        val failed = audit.checks.single { it.name == "safety_surface:stop" }
        assertEquals("surface missing", failed.detail)
    }

    @Test
    fun `audit refuses a disabled safety surface`() {
        val disabled = Fixtures.variantEditing("safety.json") { safety ->
            val surfaces = safety.arr("surfaces").map { raw ->
                val surface = raw.asObjOrEmpty()
                if (surface.str("id") == "purge") {
                    JsonObj(surface.fields + ("enabled" to JsonBool(false)))
                } else {
                    surface
                }
            }
            JsonObj(safety.fields + ("surfaces" to JsonArr(surfaces)))
        }
        val audit = SafetyAudit.audit(disabled)
        assertFalse(audit.ok)
        val failed = audit.checks.single { it.name == "safety_surface:purge" }
        assertEquals("surface disabled", failed.detail)
    }

    @Test
    fun `audit refuses an adult project whose age gate was stripped`() {
        val stripped = Fixtures.variantEditing("project.json") { meta ->
            JsonObj(meta.fields + ("age_gate" to JsonStr("")))
        }
        val audit = SafetyAudit.audit(stripped)
        assertFalse(audit.ok)
        val failed = audit.checks.single { it.name == "age_gate_present" }
        assertFalse(failed.ok)
        assertEquals("adult-rated project has no age-gate pattern", failed.detail)
        assertEquals(false, SafetyAudit.ageGateCheck(stripped)?.ok)
    }

    // ---------- channel readiness ----------

    private val apkBytes = "demo apk bytes for readiness tests".toByteArray(Charsets.UTF_8)

    private fun bundleWithHandoff(channel: String, artifactName: String, sha256: String): ProjectBundle =
        Fixtures.variant(
            "releases.json",
            """{"handoffs":[{"seq":1,"id":"handoff-1","channel":"$channel",""" +
                """"channel_name":"$channel","executor":"slipway",""" +
                """"artifacts":[{"name":"$artifactName","sha256":"$sha256","kind":"signed_apk"}],""" +
                """"status":"prepared"}]}""",
        )

    @Test
    fun `readiness passes when the artifact is present and its digest matches`() {
        val bundle = bundleWithHandoff("nutaku", "demo.apk", sha256Hex(apkBytes))
        val result = ChannelReadiness.check(bundle) { name ->
            if (name == "demo.apk") apkBytes else null
        }
        assertTrue(result.ok)
        assertEquals(1, result.handoffs.size)
        assertTrue(result.handoffs[0].hasAndroidPackage)
        assertTrue(result.handoffs[0].artifacts[0].digestMatches)
    }

    @Test
    fun `readiness covers itch and fdroid handoffs too`() {
        for (channel in listOf("itch_io", "fdroid_self_hosted")) {
            val bundle = bundleWithHandoff(channel, "game.apk", sha256Hex(apkBytes))
            val result = ChannelReadiness.check(bundle) { apkBytes }
            assertTrue("channel $channel", result.ok)
            assertEquals(1, result.handoffs.size)
        }
    }

    @Test
    fun `readiness fails when the artifact bytes were tampered with`() {
        val bundle = bundleWithHandoff("nutaku", "demo.apk", sha256Hex(apkBytes))
        val tampered = apkBytes + byteArrayOf(0x21)
        val result = ChannelReadiness.check(bundle) { tampered }
        assertFalse(result.ok)
        val artifact = result.handoffs[0].artifacts[0]
        assertTrue(artifact.present)
        assertFalse(artifact.digestMatches)
    }

    @Test
    fun `readiness fails when the artifact is absent on the device`() {
        val bundle = bundleWithHandoff("nutaku", "demo.apk", sha256Hex(apkBytes))
        val result = ChannelReadiness.check(bundle) { null }
        assertFalse(result.ok)
        assertFalse(result.handoffs[0].artifacts[0].present)
    }

    @Test
    fun `readiness fails when an android handoff carries no apk artifact`() {
        val bundle = bundleWithHandoff("fdroid_self_hosted", "game.zip", sha256Hex(apkBytes))
        val result = ChannelReadiness.check(bundle) { apkBytes }
        assertFalse(result.ok)
        assertFalse(result.handoffs[0].hasAndroidPackage)
    }

    @Test
    fun `readiness ignores handoffs to non-android channels`() {
        val bundle = bundleWithHandoff("patreon", "demo.apk", sha256Hex(apkBytes))
        val result = ChannelReadiness.check(bundle) { null }
        assertTrue(result.handoffs.isEmpty())
        assertTrue(result.ok)
    }

    // ---------- the runner, end to end ----------

    @Test
    fun `runner reproduces the recorded run and passes the fixture audit`() {
        val report = QaRunner.run(Fixtures.directoryBundle()) { null }
        assertTrue(report.safety.ok)
        assertEquals(true, report.ageGate?.ok)
        assertEquals(true, report.replay.matchesRecorded)
        assertEquals(ReplayTest.SERVER_HASH, report.replay.result.hash)
        // The fixture's itch handoff artifact is not on this
        // "device", so readiness — and only readiness — fails.
        assertFalse(report.readiness.ok)
        assertTrue(report.summaryLines().any { it.startsWith("PASS replay_recorded_hash") })
        assertTrue(report.summaryLines().any { it.startsWith("FAIL artifact itch_io/") })
    }

    @Test
    fun `runner is fully green on a bundle whose handoff artifact is present`() {
        val bundle = bundleWithHandoff("nutaku", "demo.apk", sha256Hex(apkBytes))
        val report = QaRunner.run(bundle) { name -> if (name == "demo.apk") apkBytes else null }
        assertTrue(report.ok)
    }
}
