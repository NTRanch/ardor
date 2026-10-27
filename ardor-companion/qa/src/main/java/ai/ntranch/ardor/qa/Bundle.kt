package ai.ntranch.ardor.qa

import java.io.File
import java.util.zip.ZipFile

/**
 * An exported Ardor project bundle: the server's per-project JSON
 * documents (project.json, graph.json, safety.json, ...) in one
 * directory or one .zip — the same files the Rust store reads.
 * Reading mirrors the Rust store (store.rs): project.json must be
 * present; any other missing document reads as an empty object.
 */
interface BundleSource {
    /** Text of one document (e.g. "graph.json"), or null if absent. */
    fun readDocument(name: String): String?

    /** Whether the named document exists at all. */
    fun hasDocument(name: String): Boolean
}

class DirectoryBundle(private val dir: File) : BundleSource {
    override fun readDocument(name: String): String? {
        val file = File(dir, name)
        return if (file.isFile) file.readText(Charsets.UTF_8) else null
    }

    override fun hasDocument(name: String): Boolean = File(dir, name).isFile
}

class ZipBundle(zipFile: File) : BundleSource {
    private val documents: Map<String, String> = load(zipFile)

    private fun load(zipFile: File): Map<String, String> {
        val out = HashMap<String, String>()
        ZipFile(zipFile).use { zip ->
            val entries = zip.entries()
            while (entries.hasMoreElements()) {
                val entry = entries.nextElement()
                if (entry.isDirectory) continue
                // Tolerate one leading folder level inside the zip
                // (e.g. "evidence-game/graph.json").
                val name = entry.name.substringAfterLast('/')
                if (name.endsWith(".json") && !out.containsKey(name)) {
                    out[name] = zip.getInputStream(entry).readBytes().toString(Charsets.UTF_8)
                }
            }
        }
        return out
    }

    override fun readDocument(name: String): String? = documents[name]

    override fun hasDocument(name: String): Boolean = documents.containsKey(name)
}

/** An in-memory bundle — tests and adversarial fixtures. */
class MapBundle(private val documents: Map<String, String>) : BundleSource {
    override fun readDocument(name: String): String? = documents[name]

    override fun hasDocument(name: String): Boolean = documents.containsKey(name)
}

class BundleException(message: String) : Exception(message)

/** A loaded Ardor project: the documents the QA checks consume. */
class ProjectBundle private constructor(
    /** project.json — the project's metadata (rating, age gate, start node). */
    val meta: JsonObj,
    private val documents: Map<String, JsonObj>,
) {
    fun doc(name: String): JsonObj = documents[name] ?: JsonObj(emptyMap())

    companion object {
        /** The server's document set (store.rs DOCS), in order. */
        val DOCUMENT_NAMES: List<String> = listOf(
            "assets", "canon", "cast", "graph", "levels", "project",
            "qa", "releases", "safety", "story", "systems",
        )

        fun load(source: BundleSource): ProjectBundle {
            val metaText = source.readDocument("project.json")
                ?: throw BundleException("bundle has no project.json — not an Ardor project bundle")
            val documents = HashMap<String, JsonObj>()
            for (name in DOCUMENT_NAMES) {
                val text = source.readDocument("$name.json")
                documents[name] = if (text == null) {
                    JsonObj(emptyMap())
                } else {
                    parseJson(text).asObjOrEmpty()
                }
            }
            return ProjectBundle(documents.getValue("project"), documents)
        }
    }
}
