package ai.ntranch.ardor.qa

import java.security.MessageDigest

/**
 * A minimal JSON model + parser + canonical writer.
 *
 * The canonical writer is the piece the replay-hash contract
 * depends on: it reproduces, byte for byte, the Rust server's
 * `canonical()` in ardor/src/model.rs — object keys sorted
 * lexicographically, no insignificant whitespace, and string
 * escaping exactly as serde_json's `to_string` does it (only the
 * quote, the backslash, and control characters are escaped;
 * non-ASCII passes through as UTF-8).
 *
 * Model types are deliberately small: hashed content in Ardor is
 * strings and integers only (model.rs says so), so numbers are
 * held as [Long] when integral and [Double] otherwise.
 */
sealed interface JsonValue

data object JsonNull : JsonValue

data class JsonBool(val value: Boolean) : JsonValue

data class JsonLong(val value: Long) : JsonValue

/**
 * An unsigned 64-bit integer, rendered without a sign. The JSON
 * parser never produces this; it exists so the playtest hash
 * input can carry the server's full u64 seed range exactly.
 */
data class JsonULong(val value: ULong) : JsonValue

data class JsonDouble(val value: Double) : JsonValue

data class JsonStr(val value: String) : JsonValue

data class JsonArr(val items: List<JsonValue>) : JsonValue

data class JsonObj(val fields: Map<String, JsonValue>) : JsonValue {
    operator fun get(key: String): JsonValue? = fields[key]

    fun str(key: String): String? = (fields[key] as? JsonStr)?.value

    fun arr(key: String): List<JsonValue> = (fields[key] as? JsonArr)?.items ?: emptyList()

    fun obj(key: String): JsonObj? = fields[key] as? JsonObj
}

fun JsonValue?.asObjOrEmpty(): JsonObj = this as? JsonObj ?: JsonObj(emptyMap())

class JsonParseException(message: String) : Exception(message)

/** Parse JSON text into the model. Throws [JsonParseException]. */
fun parseJson(text: String): JsonValue {
    val parser = JsonParser(text)
    val value = parser.parseValue()
    parser.skipWhitespace()
    if (!parser.atEnd()) {
        throw JsonParseException("trailing content at offset ${parser.offset}")
    }
    return value
}

private class JsonParser(private val text: String) {
    var offset: Int = 0
        private set

    fun atEnd(): Boolean = offset >= text.length

    fun skipWhitespace() {
        while (offset < text.length && text[offset] in " \t\n\r") offset++
    }

    fun parseValue(): JsonValue {
        skipWhitespace()
        if (atEnd()) throw JsonParseException("unexpected end of input")
        return when (val c = text[offset]) {
            '{' -> parseObject()
            '[' -> parseArray()
            '"' -> JsonStr(parseString())
            't' -> parseLiteral("true", JsonBool(true))
            'f' -> parseLiteral("false", JsonBool(false))
            'n' -> parseLiteral("null", JsonNull)
            '-', in '0'..'9' -> parseNumber()
            else -> throw JsonParseException("unexpected character '$c' at offset $offset")
        }
    }

    private fun parseLiteral(word: String, value: JsonValue): JsonValue {
        if (!text.startsWith(word, offset)) {
            throw JsonParseException("invalid literal at offset $offset")
        }
        offset += word.length
        return value
    }

    private fun parseObject(): JsonObj {
        offset++ // '{'
        val fields = LinkedHashMap<String, JsonValue>()
        skipWhitespace()
        if (!atEnd() && text[offset] == '}') {
            offset++
            return JsonObj(fields)
        }
        while (true) {
            skipWhitespace()
            if (atEnd() || text[offset] != '"') {
                throw JsonParseException("expected object key at offset $offset")
            }
            val key = parseString()
            skipWhitespace()
            if (atEnd() || text[offset] != ':') {
                throw JsonParseException("expected ':' at offset $offset")
            }
            offset++
            fields[key] = parseValue()
            skipWhitespace()
            if (atEnd()) throw JsonParseException("unterminated object")
            when (text[offset]) {
                ',' -> offset++
                '}' -> {
                    offset++
                    return JsonObj(fields)
                }
                else -> throw JsonParseException("expected ',' or '}' at offset $offset")
            }
        }
    }

    private fun parseArray(): JsonArr {
        offset++ // '['
        val items = ArrayList<JsonValue>()
        skipWhitespace()
        if (!atEnd() && text[offset] == ']') {
            offset++
            return JsonArr(items)
        }
        while (true) {
            items.add(parseValue())
            skipWhitespace()
            if (atEnd()) throw JsonParseException("unterminated array")
            when (text[offset]) {
                ',' -> offset++
                ']' -> {
                    offset++
                    return JsonArr(items)
                }
                else -> throw JsonParseException("expected ',' or ']' at offset $offset")
            }
        }
    }

    private fun parseString(): String {
        offset++ // opening quote
        val out = StringBuilder()
        while (true) {
            if (atEnd()) throw JsonParseException("unterminated string")
            val c = text[offset++]
            when (c) {
                '"' -> return out.toString()
                '\\' -> {
                    if (atEnd()) throw JsonParseException("unterminated escape")
                    when (val e = text[offset++]) {
                        '"' -> out.append('"')
                        '\\' -> out.append('\\')
                        '/' -> out.append('/')
                        'b' -> out.append('\b')
                        'f' -> out.append('\u000C')
                        'n' -> out.append('\n')
                        'r' -> out.append('\r')
                        't' -> out.append('\t')
                        'u' -> {
                            if (offset + 4 > text.length) {
                                throw JsonParseException("truncated \\u escape")
                            }
                            val first = text.substring(offset, offset + 4).toIntOrNull(16)
                                ?: throw JsonParseException("bad \\u escape at offset $offset")
                            offset += 4
                            if (first in 0xD800..0xDBFF &&
                                offset + 6 <= text.length &&
                                text[offset] == '\\' && text[offset + 1] == 'u'
                            ) {
                                val second = text.substring(offset + 2, offset + 6).toIntOrNull(16)
                                if (second != null && second in 0xDC00..0xDFFF) {
                                    offset += 6
                                    out.appendCodePoint(
                                        0x10000 + ((first - 0xD800) shl 10) + (second - 0xDC00),
                                    )
                                } else {
                                    out.append(first.toChar())
                                }
                            } else {
                                out.append(first.toChar())
                            }
                        }
                        else -> throw JsonParseException("bad escape '\\$e' at offset ${offset - 1}")
                    }
                }
                else -> out.append(c)
            }
        }
    }

    private fun parseNumber(): JsonValue {
        val start = offset
        if (text[offset] == '-') offset++
        while (!atEnd() && text[offset].isDigit()) offset++
        var floating = false
        if (!atEnd() && text[offset] == '.') {
            floating = true
            offset++
            while (!atEnd() && text[offset].isDigit()) offset++
        }
        if (!atEnd() && (text[offset] == 'e' || text[offset] == 'E')) {
            floating = true
            offset++
            if (!atEnd() && (text[offset] == '+' || text[offset] == '-')) offset++
            while (!atEnd() && text[offset].isDigit()) offset++
        }
        val raw = text.substring(start, offset)
        if (raw.isEmpty() || raw == "-") {
            throw JsonParseException("invalid number at offset $start")
        }
        return if (floating) {
            JsonDouble(raw.toDoubleOrNull() ?: throw JsonParseException("invalid number '$raw'"))
        } else {
            JsonLong(raw.toLongOrNull() ?: throw JsonParseException("invalid integer '$raw'"))
        }
    }
}

/**
 * Canonical JSON, byte-identical to the Rust server's
 * `canonical()` (keys sorted, no whitespace, serde_json string
 * escaping). This is what gets hashed — never hash anything else.
 */
fun canonicalJson(value: JsonValue): String {
    val out = StringBuilder()
    writeCanonical(value, out)
    return out.toString()
}

private fun writeCanonical(value: JsonValue, out: StringBuilder) {
    when (value) {
        JsonNull -> out.append("null")
        is JsonBool -> out.append(if (value.value) "true" else "false")
        is JsonLong -> out.append(value.value.toString())
        is JsonULong -> out.append(value.value.toString())
        is JsonDouble -> out.append(formatDouble(value.value))
        is JsonStr -> writeJsonString(value.value, out)
        is JsonArr -> {
            out.append('[')
            value.items.forEachIndexed { index, item ->
                if (index > 0) out.append(',')
                writeCanonical(item, out)
            }
            out.append(']')
        }
        is JsonObj -> {
            out.append('{')
            val keys = value.fields.keys.sorted()
            keys.forEachIndexed { index, key ->
                if (index > 0) out.append(',')
                writeJsonString(key, out)
                out.append(':')
                writeCanonical(value.fields.getValue(key), out)
            }
            out.append('}')
        }
    }
}

private fun formatDouble(value: Double): String {
    // Hashed Ardor content is integers-only; this path exists so
    // non-hashed reporting never silently mangles a double.
    return if (value.isFinite() && value == Math.floor(value) && Math.abs(value) < 1e15) {
        value.toLong().toString()
    } else {
        value.toString()
    }
}

/** String escaping exactly as serde_json produces it. */
private fun writeJsonString(value: String, out: StringBuilder) {
    out.append('"')
    for (c in value) {
        when (c) {
            '"' -> out.append("\\\"")
            '\\' -> out.append("\\\\")
            '\n' -> out.append("\\n")
            '\r' -> out.append("\\r")
            '\t' -> out.append("\\t")
            '\b' -> out.append("\\b")
            '\u000C' -> out.append("\\f")
            else -> {
                if (c.code < 0x20) {
                    out.append("\\u")
                    out.append(c.code.toString(16).padStart(4, '0'))
                } else {
                    out.append(c)
                }
            }
        }
    }
    out.append('"')
}

/** SHA-256 of the UTF-8 bytes, lowercase hex — the Rust `sha256_hex`. */
fun sha256Hex(text: String): String {
    val digest = MessageDigest.getInstance("SHA-256").digest(text.toByteArray(Charsets.UTF_8))
    return digest.joinToString("") { "%02x".format(it) }
}

/** SHA-256 of raw bytes, lowercase hex. */
fun sha256Hex(bytes: ByteArray): String {
    val digest = MessageDigest.getInstance("SHA-256").digest(bytes)
    return digest.joinToString("") { "%02x".format(it) }
}
