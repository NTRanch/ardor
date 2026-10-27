@file:OptIn(io.modelcontextprotocol.kotlin.sdk.ExperimentalMcpApi::class)

package ai.ntranch.ardor.client

import io.modelcontextprotocol.kotlin.sdk.client.StdioClientTransport
import io.modelcontextprotocol.kotlin.sdk.types.CallToolResult
import io.modelcontextprotocol.kotlin.sdk.types.DiscoverResult
import io.modelcontextprotocol.kotlin.sdk.types.JSONRPCError
import io.modelcontextprotocol.kotlin.sdk.types.JSONRPCMessage
import io.modelcontextprotocol.kotlin.sdk.types.JSONRPCRequest
import io.modelcontextprotocol.kotlin.sdk.types.JSONRPCResponse
import io.modelcontextprotocol.kotlin.sdk.types.ListToolsResult
import io.modelcontextprotocol.kotlin.sdk.types.RequestId
import io.modelcontextprotocol.kotlin.sdk.types.TextContent
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.withTimeout
import kotlinx.io.asSink
import kotlinx.io.asSource
import kotlinx.io.buffered
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicLong

/** A protocol-level MCP failure surfaced to the caller. */
class McpSessionException(val code: Int, message: String) : Exception("MCP error $code: $message")

/**
 * A modern-era (2026-07-28) MCP client session over stdio,
 * built on the official Kotlin MCP SDK's transport and types —
 * the D-118 pattern, applied to the Ardor Rust server.
 *
 * Why this class exists: kotlin-sdk 0.16.0's high-level `Client`
 * still performs the legacy `initialize` handshake (its
 * `LATEST_PROTOCOL_VERSION` is 2025-11-25), which modern-only
 * servers refuse by design. The SDK's core types and its stdio
 * transport are already modern-capable (`DiscoverRequest` /
 * `DiscoverResult`, request-scoped `_meta`), so this session
 * drives that machinery in the modern order:
 *
 * 1. `server/discover` first contact — no `initialize`, ever.
 * 2. Every request carries `_meta` with the protocol version,
 *    client info, and client capabilities — the per-request
 *    stamping the stateless modern era runs on.
 *
 * When the SDK's `Client` gains modern-era support, this class
 * is the single, small replacement point.
 */
class ArdorMcpSession private constructor(
    private val transport: StdioClientTransport,
) {
    private val pending = ConcurrentHashMap<Long, CompletableDeferred<JSONRPCMessage>>()
    private val ids = AtomicLong(1)

    /** Every method this session has sent, in order (test-visible). */
    val sentMethods = CopyOnWriteArrayList<String>()

    companion object {
        const val PROTOCOL_VERSION: String = "2026-07-28"
        const val CLIENT_NAME: String = "ardor-companion"
        const val CLIENT_VERSION: String = "0.1.0"

        /**
         * A session against a spawned server process: the SDK's
         * stdio transport over the process's streams.
         */
        fun overProcess(process: Process): ArdorMcpSession {
            val transport = StdioClientTransport(
                input = process.inputStream.asSource().buffered(),
                output = process.outputStream.asSink().buffered(),
                error = process.errorStream.asSource().buffered(),
            )
            return ArdorMcpSession(transport)
        }
    }

    private fun modernMeta(): JsonObject = buildJsonObject {
        put("io.modelcontextprotocol/protocolVersion", JsonPrimitive(PROTOCOL_VERSION))
        put(
            "io.modelcontextprotocol/clientInfo",
            buildJsonObject {
                put("name", JsonPrimitive(CLIENT_NAME))
                put("version", JsonPrimitive(CLIENT_VERSION))
            },
        )
        put("io.modelcontextprotocol/clientCapabilities", buildJsonObject {})
    }

    private fun idValue(id: RequestId): Long = (id as? RequestId.NumberId)?.value ?: -1L

    /** Install the response correlator and start the transport. */
    suspend fun connect() {
        transport.onMessage { message ->
            when (message) {
                is JSONRPCResponse ->
                    pending.remove(idValue(message.id))?.complete(message)

                is JSONRPCError ->
                    message.id?.let { pending.remove(idValue(it))?.complete(message) }

                else -> Unit
            }
        }
        transport.start()
    }

    suspend fun close() {
        transport.close()
    }

    private suspend fun request(method: String, params: JsonObject): JSONRPCMessage {
        val id = ids.getAndIncrement()
        sentMethods.add(method)
        val deferred = CompletableDeferred<JSONRPCMessage>()
        pending[id] = deferred
        try {
            transport.send(
                JSONRPCRequest(id = RequestId.NumberId(id), method = method, params = params),
            )
            return withTimeout(30_000) { deferred.await() }
        } finally {
            pending.remove(id)
        }
    }

    private fun unwrap(message: JSONRPCMessage): JSONRPCResponse = when (message) {
        is JSONRPCResponse -> message
        is JSONRPCError -> throw McpSessionException(message.error.code, message.error.message)
        else -> throw McpSessionException(-1, "unexpected message type: $message")
    }

    /** Modern first contact: `server/discover`, never `initialize`. */
    suspend fun discover(): DiscoverResult {
        val response = unwrap(
            request("server/discover", buildJsonObject { put("_meta", modernMeta()) }),
        )
        return response.result as? DiscoverResult
            ?: throw McpSessionException(-1, "discover returned ${response.result}")
    }

    /** The server's tool inventory. */
    suspend fun listTools(): ListToolsResult {
        val response = unwrap(
            request("tools/list", buildJsonObject { put("_meta", modernMeta()) }),
        )
        return response.result as? ListToolsResult
            ?: throw McpSessionException(-1, "tools/list returned ${response.result}")
    }

    /** Call one tool with JSON arguments. */
    suspend fun callTool(name: String, arguments: JsonObject): CallToolResult {
        val response = unwrap(
            request(
                "tools/call",
                buildJsonObject {
                    put("_meta", modernMeta())
                    put("name", JsonPrimitive(name))
                    put("arguments", arguments)
                },
            ),
        )
        return response.result as? CallToolResult
            ?: throw McpSessionException(-1, "tools/call returned ${response.result}")
    }
}

/** The text payload of a tool result (the first text block). */
fun CallToolResult.firstText(): String =
    content.filterIsInstance<TextContent>().firstOrNull()?.text ?: ""
