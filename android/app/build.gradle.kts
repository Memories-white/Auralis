plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "dev.qqlyric.app"
    compileSdk = 37
    compileSdkMinor = 2
    buildFeatures { compose = true; buildConfig = true }
    defaultConfig {
        applicationId = "dev.qqlyric.app"
        minSdk = 26
        targetSdk = 35
        versionCode = 17
        versionName = "0.3.7-preview"
        ndk { abiFilters += "arm64-v8a" }
    }
    compileOptions { sourceCompatibility = JavaVersion.VERSION_17; targetCompatibility = JavaVersion.VERSION_17 }
    kotlin { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }
}
dependencies {
    implementation("top.yukonga.miuix.kmp:miuix-ui-android:0.9.5-beta01")
    implementation("top.yukonga.miuix.kmp:miuix-preference-android:0.9.5-beta01")
    implementation("top.yukonga.miuix.kmp:miuix-icons-android:0.9.5-beta01")
    implementation("androidx.activity:activity-compose:1.12.4")
    implementation("com.google.android.material:material:1.13.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.core:core-ktx:1.18.0")
    implementation("androidx.fragment:fragment:1.5.4")
    implementation("androidx.profileinstaller:profileinstaller:1.3.1")
    implementation("androidx.tracing:tracing:1.2.0")
    implementation("androidx.startup:startup-runtime:1.2.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
    implementation("androidx.lifecycle:lifecycle-runtime:2.6.2")
    implementation("androidx.lifecycle:lifecycle-viewmodel-savedstate:2.6.2")
}
