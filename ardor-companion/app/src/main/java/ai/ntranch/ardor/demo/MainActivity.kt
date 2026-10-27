package ai.ntranch.ardor.demo

import android.app.Activity
import android.os.Bundle
import android.widget.TextView

/**
 * The demo app's only shipped screen. It deliberately references
 * nothing from the companion QA module: in the store flavor the
 * module is not even a dependency, and this class is the proof
 * surface — the release artifact is scanned for companion
 * symbols and must come back empty.
 */
class MainActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(
            TextView(this).apply {
                text = "Ardor companion demo — store build.\n" +
                    "The on-device QA module ships only in the qa flavor."
                textSize = 16f
                setPadding(48, 48, 48, 48)
            },
        )
    }
}
