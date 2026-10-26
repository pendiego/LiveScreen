plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

// Versão vem do Cargo.toml da raiz (fonte única, ver AGENTS.md); -PappVersion sobrescreve nos testes.
val cargoVersion = rootProject.projectDir.parentFile.resolve("Cargo.toml").readLines()
    .first { it.startsWith("version = ") }.substringAfter('"').substringBefore('"')
val appVersion = (findProperty("appVersion") as String?) ?: cargoVersion
val (vMaj, vMin, vPatch) = appVersion.split('.').map { it.toInt() }

android {
    namespace = "io.github.pendiego.livescreen"
    compileSdk = 35

    defaultConfig {
        applicationId = "io.github.pendiego.livescreen"
        minSdk = 26
        targetSdk = 35
        versionCode = vMaj * 10000 + vMin * 100 + vPatch
        versionName = appVersion
        buildConfigField("String", "UPDATE_REPO", "\"${findProperty("updateRepo") ?: "pendiego/LiveScreen"}\"")

        ndk {
            abiFilters += listOf("x86_64", "arm64-v8a")
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    val releaseKeystore = System.getenv("LIVESCREEN_KEYSTORE")
    signingConfigs {
        if (releaseKeystore != null) create("release") {
            storeFile = file(releaseKeystore)
            storePassword = System.getenv("LIVESCREEN_KEYSTORE_PASSWORD")
            keyAlias = "livescreen"
            keyPassword = System.getenv("LIVESCREEN_KEYSTORE_PASSWORD")
        }
    }
    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            if (releaseKeystore != null) signingConfig = signingConfigs.getByName("release")
        }
    }

    testOptions { unitTests.isReturnDefaultValues = true } // Log.w vira no-op no JVM de teste
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
    val composeBom = platform("androidx.compose:compose-bom:2025.08.00")
    implementation(composeBom)
    implementation("androidx.compose.material3:material3")
    implementation("androidx.activity:activity-compose:1.10.1")
    testImplementation("junit:junit:4.13.2")
}

val cargoBuild by tasks.registering(Exec::class) {
    workingDir = rootProject.projectDir.parentFile
    val sdkDir = System.getenv("ANDROID_HOME")?.let(::File)
        ?: File(System.getProperty("user.home"), "AppData/Local/Android/Sdk")
    environment("ANDROID_HOME", sdkDir.absolutePath)
    val ndkDir = File(sdkDir, "ndk/27.3.13750724")
    if (System.getenv("ANDROID_NDK_HOME") == null && ndkDir.exists()) {
        environment("ANDROID_NDK_HOME", ndkDir.absolutePath)
    }
    commandLine(
        "cargo", "ndk",
        "-t", "x86_64",
        "-t", "arm64-v8a",
        "-P", "26",
        "-o", "android/app/src/main/jniLibs",
        "build", "--release",
        "-p", "livescreen-android"
    )
}

tasks.named("preBuild") {
    dependsOn(cargoBuild)
}
