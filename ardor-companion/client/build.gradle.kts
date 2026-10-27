plugins {
    id("org.jetbrains.kotlin.jvm")
}

kotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }
}

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

dependencies {
    // The official Kotlin MCP SDK — client artifact only. Its
    // stdio transport and modern-era core types are what the
    // session layer drives (see ArdorMcpSession).
    implementation("io.modelcontextprotocol:kotlin-sdk-client:0.16.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.11.0")

    testImplementation("junit:junit:4.13.2")
}

tasks.test {
    // The interop test drives the real Rust server binary. Paths
    // arrive via the environment so the same build runs anywhere:
    //   ARDOR_BIN        — path to the ardor server binary
    //   ARDOR_SERVER_HOME_FIXTURE — the fixtures/server-home dir
    systemProperty("ardor.bin", System.getenv("ARDOR_BIN") ?: "")
    systemProperty(
        "ardor.serverHomeFixture",
        System.getenv("ARDOR_SERVER_HOME_FIXTURE")
            ?: rootProject.projectDir.resolve("fixtures/server-home").absolutePath,
    )
    testLogging {
        events("passed", "failed", "skipped")
        showStandardStreams = true
    }
}
