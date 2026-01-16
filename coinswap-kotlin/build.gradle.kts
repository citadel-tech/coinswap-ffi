plugins {
    kotlin("jvm") version "1.9.22"
}

group = "uniffi.coinswap"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
    google()
}

dependencies {
    // Kotlin stdlib
    implementation(kotlin("stdlib"))
    
    // JNA for FFI bindings
    implementation("net.java.dev.jna:jna:5.13.0")
    
    // Coroutines (for async operations)
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
    
    // Testing
    testImplementation(kotlin("test"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.1")
}

kotlin {
    jvmToolchain(21)
}

tasks.test {
    useJUnitPlatform()
    
    // Show test output in console
    testLogging {
        events("passed", "skipped", "failed", "standardOut", "standardError")
        showExceptions = true
        showCauses = true
        showStackTraces = true
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }
}

// Task to copy generated bindings from uniffi/ to src/main/kotlin/
tasks.register<Copy>("syncBindings") {
    description = "Sync generated UniFFI bindings to src/main/kotlin"
    from("uniffi/coinswap")
    into("src/main/kotlin/uniffi/coinswap")
    include("*.kt")
}

// Task to copy native library to resources
tasks.register<Copy>("syncNativeLib") {
    description = "Sync native library to resources"
    from(".")
    into("src/main/resources/linux-x86-64")
    include("libcoinswap_ffi.so")
}

// Make build depend on syncing
tasks.named("compileKotlin") {
    dependsOn("syncBindings", "syncNativeLib")
}

tasks.named("processResources") {
    dependsOn("syncNativeLib")
}
