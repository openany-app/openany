import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

// DIE SIGNATUR LIEGT NICHT IM REPO. Gelesen wird eine Properties-Datei
// (storeFile, storePassword, keyAlias, keyPassword): aus `OPENANY_SIGNATUR`,
// sonst aus ~/anyx/geheim/android-signatur.properties (dort git-ignoriert).
//
// Fehlt sie, bricht ein Release-Build AB, statt ein unsigniertes APK zu
// liefern: Das liesse sich nicht installieren, und der Grund stuende nur
// im Dateinamen `-unsigned`.
val signatur = Properties().apply {
    val pfad = System.getenv("OPENANY_SIGNATUR")
        ?: "${System.getProperty("user.home")}/anyx/geheim/android-signatur.properties"
    val datei = file(pfad)
    if (datei.exists()) {
        datei.inputStream().use { load(it) }
    }
}

android {
    compileSdk = 36
    signingConfigs {
        if (signatur.getProperty("storeFile") != null) {
            create("release") {
                storeFile = file(signatur.getProperty("storeFile"))
                storePassword = signatur.getProperty("storePassword")
                keyAlias = signatur.getProperty("keyAlias")
                keyPassword = signatur.getProperty("keyPassword")
            }
        }
    }
    namespace = "de.openany.app"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "de.openany.app"
        minSdk = 24
        targetSdk = 36
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            signingConfig = signingConfigs.findByName("release")
            isMinifyEnabled = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
}

rust {
    rootDirRel = "../../../"
}

// Erst beim Ausfuehren eines Release-Tasks pruefen, nicht beim Einlesen:
// Sonst braeche auch jeder Debug-Build auf einem Rechner ohne Signatur ab.
tasks.matching { it.name.matches(Regex("(assemble|bundle|package)\\w*Release")) }.configureEach {
    doFirst {
        if (android.signingConfigs.findByName("release") == null) {
            throw GradleException(
                "Keine Signatur: OPENANY_SIGNATUR setzen oder " +
                    "~/anyx/geheim/android-signatur.properties anlegen."
            )
        }
    }
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.lifecycle:lifecycle-process:2.10.0")
    // Der Auffrischer: Abgleich im Hintergrund, etwa stuendlich (Auffrischer.kt).
    implementation("androidx.work:work-runtime-ktx:2.10.1")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")