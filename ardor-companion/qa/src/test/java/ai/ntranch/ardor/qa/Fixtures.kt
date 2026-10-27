package ai.ntranch.ardor.qa

import java.io.File

/** Access to the canonical evidence-game fixture bundle. */
object Fixtures {
    val dir: File
        get() = File(
            System.getProperty("ardor.fixtures.dir")
                ?: error("ardor.fixtures.dir system property not set"),
            "evidence-game",
        )

    fun directoryBundle(): ProjectBundle = ProjectBundle.load(DirectoryBundle(dir))

    /** The fixture's documents as raw text, keyed by file name. */
    fun rawDocuments(): LinkedHashMap<String, String> {
        val out = LinkedHashMap<String, String>()
        for (name in ProjectBundle.DOCUMENT_NAMES) {
            val file = File(dir, "$name.json")
            if (file.isFile) out["$name.json"] = file.readText(Charsets.UTF_8)
        }
        return out
    }

    /** A fixture variant with one document replaced — the tamper tool. */
    fun variant(document: String, replacement: String): ProjectBundle {
        val docs = rawDocuments()
        docs[document] = replacement
        return ProjectBundle.load(MapBundle(docs))
    }

    /** A fixture variant with one document edited as parsed JSON. */
    fun variantEditing(document: String, edit: (JsonObj) -> JsonObj): ProjectBundle {
        val docs = rawDocuments()
        val parsed = parseJson(docs.getValue(document)).asObjOrEmpty()
        docs[document] = canonicalJson(edit(parsed))
        return ProjectBundle.load(MapBundle(docs))
    }
}
