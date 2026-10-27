plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "ai.ntranch.ardor.demo"
    compileSdk = 36

    defaultConfig {
        applicationId = "ai.ntranch.ardor.demo"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    flavorDimensions += "channel"
    productFlavors {
        // The shipped shape: no companion code at all. The QA
        // library is not a dependency of this flavor, so the
        // release artifact provably contains no companion surface.
        create("store") {
            dimension = "channel"
            isDefault = true
        }
        // The QA shape: same app plus the companion module and
        // its on-device battery. Separate applicationId, never
        // submitted anywhere.
        create("qa") {
            dimension = "channel"
            applicationIdSuffix = ".qa"
            versionNameSuffix = "-qa"
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

kotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }
}

dependencies {
    // The companion exists only in the qa flavor.
    "qaImplementation"(project(":qa"))
}
