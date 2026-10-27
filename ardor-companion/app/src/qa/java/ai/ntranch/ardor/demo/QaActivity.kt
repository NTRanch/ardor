package ai.ntranch.ardor.demo

import ai.ntranch.ardor.qa.DirectoryBundle
import ai.ntranch.ardor.qa.ProjectBundle
import ai.ntranch.ardor.qa.QaRunner
import android.app.Activity
import android.os.Bundle
import android.text.method.ScrollingMovementMethod
import android.widget.TextView
import java.io.File
import kotlin.concurrent.thread

/**
 * The on-device QA battery (qa flavor only): extracts the demo
 * bundle from assets, runs the companion's checks against it —
 * safety audit, age gate, deterministic replay against the
 * server's recorded hash, Android channel readiness — and
 * renders one PASS/FAIL line per check, machine-readable via
 * `uiautomator dump` for the gate runs.
 */
class QaActivity : Activity() {

    private lateinit var output: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        output = TextView(this).apply {
            textSize = 13f
            setPadding(32, 32, 32, 32)
            movementMethod = ScrollingMovementMethod()
            text = "ARDOR-QA running…"
        }
        setContentView(output)
        thread(name = "ardor-qa") {
            val lines = runBattery()
            runOnUiThread { output.text = lines.joinToString("\n") }
        }
    }

    private fun runBattery(): List<String> {
        val lines = ArrayList<String>()
        lines.add("ARDOR-QA start")
        try {
            val bundleDir = extractAssetTree("demo-bundle", File(filesDir, "demo-bundle"))
            val artifactsDir = extractAssetTree("artifacts", File(filesDir, "artifacts"))
            val bundle = ProjectBundle.load(DirectoryBundle(bundleDir))
            val report = QaRunner.run(bundle) { name ->
                File(artifactsDir, name).takeIf { it.isFile }?.readBytes()
            }
            lines.addAll(report.summaryLines().map { "ARDOR-QA $it" })
            val passed = report.summaryLines().count { it.startsWith("PASS") }
            val total = report.summaryLines().size
            lines.add("ARDOR-QA DONE $passed/$total ${if (report.ok) "PASS" else "FAIL"}")
        } catch (t: Throwable) {
            lines.add("ARDOR-QA FAIL battery error: ${t.javaClass.simpleName}: ${t.message}")
        }
        return lines
    }

    private fun extractAssetTree(assetPath: String, targetDir: File): File {
        targetDir.mkdirs()
        val entries = assets.list(assetPath) ?: emptyArray()
        for (entry in entries) {
            val childAsset = "$assetPath/$entry"
            val childTarget = File(targetDir, entry)
            val subEntries = assets.list(childAsset)
            if (subEntries != null && subEntries.isNotEmpty()) {
                extractAssetTree(childAsset, childTarget)
            } else {
                assets.open(childAsset).use { input ->
                    childTarget.outputStream().use { output -> input.copyTo(output) }
                }
            }
        }
        return targetDir
    }
}
