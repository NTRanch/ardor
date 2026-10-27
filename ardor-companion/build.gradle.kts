// Ardor Stage 2 — Kotlin companion. Toolchain pins follow the
// D-118 pattern as proven on this estate's build host:
//   AGP 8.13.2 · Kotlin Gradle plugin 2.4.21 · Gradle 8.14.4 · JDK 21
// kotlin-sdk 0.16.0 ships Kotlin-2.4 metadata, which is why the
// Kotlin plugin is pinned at 2.4.21 (an older plugin cannot read it).
plugins {
    id("com.android.application") version "8.13.2" apply false
    id("com.android.library") version "8.13.2" apply false
    id("org.jetbrains.kotlin.android") version "2.4.21" apply false
    id("org.jetbrains.kotlin.jvm") version "2.4.21" apply false
}
