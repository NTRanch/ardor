package ai.ntranch.ardor.client

import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Test
import java.io.File
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.TimeUnit

/**
 * Interop gate: this Kotlin client, over the SDK's stdio
 * transport, driving the REAL Ardor Rust server binary —
 * discover-first, modern era only. The session transcript is
 * written to build/interop-transcript.txt.
 *
 * Requires ARDOR_BIN (the server binary). Skipped when unset.
 */
class ArdorServerInteropTest {

    companion object {
        const val SERVER_HASH = "14b271c23aef8468e568ee2785041d015be7a27a753fa9d62f0fa4aa59a088f8"
    }

    @Test
    fun `kotlin client drives the rust server through a full session`() = runBlocking {
        val binary = System.getProperty("ardor.bin") ?: ""
        assumeTrue("ARDOR_BIN not set — interop gate skipped", binary.isNotEmpty() && File(binary).canExecute())
        val fixtureHome = File(System.getProperty("ardor.serverHomeFixture") ?: "")
        assertTrue("server-home fixture missing: $fixtureHome", fixtureHome.isDirectory)

        // A private copy of the fixture home: the server writes
        // (run_playtest records its run), the fixture stays clean.
        val home = Files.createTempDirectory("ardor-interop-home").toFile()
        fixtureHome.walkTopDown().forEach { source ->
            val target = File(home, source.relativeTo(fixtureHome).path)
            if (source.isDirectory) {
                target.mkdirs()
            } else {
                Files.copy(source.toPath(), target.toPath(), StandardCopyOption.REPLACE_EXISTING)
            }
        }

        val transcript = StringBuilder()
        transcript.appendLine("ARDOR STAGE 2 INTEROP — Kotlin client (kotlin-sdk 0.16.0 stdio transport + modern session layer)")
        transcript.appendLine("binary: $binary")
        transcript.appendLine("home: $home (temp copy of fixtures/server-home)")
        fun log(line: String) {
            transcript.appendLine(line)
            println(line)
        }

        val stderrFile = File("build/interop-server-stderr.log")
        stderrFile.parentFile.mkdirs()
        val process = ProcessBuilder(binary, "--home", home.absolutePath)
            .redirectError(stderrFile)
            .start()
        val session = ArdorMcpSession.overProcess(process)
        try {
            session.connect()

            val discover = session.discover()
            log(">>> server/discover")
            log("<<< supportedVersions=${discover.supportedVersions}")
            assertEquals(listOf("2026-07-28"), discover.supportedVersions)

            val tools = session.listTools()
            val names = tools.tools.map { it.name }
            log(">>> tools/list")
            log("<<< ${names.size} tools: ${names.joinToString(", ")}")
            assertTrue("expected the full Stage 1 surface, got ${names.size}", names.size >= 46)
            for (required in listOf("get_project", "run_playtest", "publish_preflight", "list_channels")) {
                assertTrue("missing tool $required", required in names)
            }

            val project = session.callTool(
                "get_project",
                buildJsonObject { put("project", JsonPrimitive("evidence-game")) },
            )
            log(">>> tools/call get_project {project: evidence-game}")
            log("<<< isError=${project.isError}\n${project.firstText()}")
            assertFalse(project.isError == true)
            assertTrue(project.firstText().contains("evidence-game"))

            val playtest = session.callTool(
                "run_playtest",
                buildJsonObject {
                    put("project", JsonPrimitive("evidence-game"))
                    put("seed", JsonPrimitive(7))
                    put("max_steps", JsonPrimitive(32))
                },
            )
            log(">>> tools/call run_playtest {project: evidence-game, seed: 7, max_steps: 32}")
            log("<<< isError=${playtest.isError}\n${playtest.firstText()}")
            assertFalse(playtest.isError == true)
            assertTrue(
                "Kotlin-driven server run must reproduce the D-116 hash",
                playtest.firstText().contains(SERVER_HASH),
            )

            val preflight = session.callTool(
                "publish_preflight",
                buildJsonObject {
                    put("project", JsonPrimitive("evidence-game"))
                    put("channel", JsonPrimitive("itch_io"))
                },
            )
            log(">>> tools/call publish_preflight {project: evidence-game, channel: itch_io}")
            log("<<< isError=${preflight.isError}\n${preflight.firstText()}")
            assertFalse(preflight.isError == true)
            assertTrue(preflight.firstText().contains("itch_io"))

            log("methods sent, in order: ${session.sentMethods}")
            assertEquals("server/discover", session.sentMethods.first())
            assertFalse(
                "the legacy handshake must never be sent",
                session.sentMethods.any { it == "initialize" },
            )
        } finally {
            runCatching { session.close() }
            process.outputStream.close()
            if (!process.waitFor(10, TimeUnit.SECONDS)) {
                process.destroy()
            }
            val out = File("build/interop-transcript.txt")
            out.parentFile.mkdirs()
            out.writeText(transcript.toString())
        }
    }
}
