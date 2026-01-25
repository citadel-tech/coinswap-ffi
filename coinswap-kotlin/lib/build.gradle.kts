plugins {
    id("com.android.library")
    kotlin("android")
    id("maven-publish")
    id("signing")
    id("org.jetbrains.dokka") version "1.9.20"
}

group = "org.coinswap"
version = "1.0.0"

android {
    namespace = "org.coinswap"
    compileSdk = 34

    defaultConfig {
        minSdk = 24
        
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("proguard-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(file("proguard-android-optimize.txt"), file("proguard-rules.pro"))
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
        getByName("test") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    testOptions {
        unitTests.isIncludeAndroidResources = true
    }

    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

dependencies {
    implementation(kotlin("stdlib"))
    implementation("net.java.dev.jna:jna:5.13.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
    
    testImplementation(kotlin("test"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.1")
}

tasks.withType<Test>().configureEach {
    useJUnitPlatform()

    val x86_64JniLibsPath = file("src/main/jniLibs/x86_64").absolutePath
    systemProperty("jna.library.path", x86_64JniLibsPath)
    systemProperty("jna.platform.library.path", x86_64JniLibsPath)
    systemProperty("coinswap.resource.dir", x86_64JniLibsPath)

    testLogging {
        events("passed", "skipped", "failed", "standardOut", "standardError")
        showExceptions = true
        showCauses = true
        showStackTraces = true
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }
}

tasks.matching { it.name == "test" }.configureEach {
    dependsOn("testDebugUnitTest")
}

val dokkaJavadocJar by tasks.registering(Jar::class) {
    archiveClassifier.set("javadoc")
    from(tasks.named("dokkaHtml"))
}

afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("maven") {
                groupId = "org.coinswap"
                artifactId = "coinswap-kotlin"
                version = project.version.toString()

                from(components["release"])
                artifact(dokkaJavadocJar)
                pom {
                    name.set("coinswap-kotlin")
                    description.set("Coinswap Kotlin language bindings.")
                    url.set("https://github.com/citadel-tech/coinswap-ffi")
                    licenses {
                        license {
                            name.set("Apache-2.0")
                            url.set("https://www.apache.org/licenses/LICENSE-2.0")
                        }
                    }
                    developers {
                        developer {
                            id.set("citadel-tech")
                            name.set("Citadel-Tech")
                            email.set("dev@citadel-tech.org")
                        }
                    }
                    scm {
                        connection.set("scm:git:github.com/citadel-tech/coinswap-ffi.git")
                        developerConnection.set("scm:git:ssh://github.com/citadel-tech/coinswap-ffi.git")
                        url.set("https://github.com/citadel-tech/coinswap-ffi")
                    }
                }
            }
        }
    }
}

signing {
    if (project.hasProperty("localBuild")) {
        isRequired = false
    }
    sign(publishing.publications)
}