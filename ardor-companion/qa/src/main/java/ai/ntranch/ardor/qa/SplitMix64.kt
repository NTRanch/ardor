package ai.ntranch.ardor.qa

/**
 * SplitMix64, a bit-exact port of the stream in the Rust server's
 * model.rs. Playtest generation draws branch choices from this
 * stream, seeded by the caller, so a run is reproducible
 * byte-for-byte across implementations.
 */
class SplitMix64(seed: ULong) {
    private var state: ULong = seed

    fun nextU64(): ULong {
        state += 0x9E3779B97F4A7C15uL
        var z = state
        z = (z xor (z shr 30)) * 0xBF58476D1CE4E5B9uL
        z = (z xor (z shr 27)) * 0x94D049BB133111EBuL
        return z xor (z shr 31)
    }

    /** Draw in `0..n` (n must be positive) — the Rust `below()`. */
    fun below(n: Int): Int {
        require(n > 0) { "below() needs a positive bound" }
        return (nextU64() % n.toULong()).toInt()
    }
}
