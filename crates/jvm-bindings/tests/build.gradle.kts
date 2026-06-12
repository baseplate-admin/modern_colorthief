plugins {
    `java`
    kotlin("jvm") version "2.4.0"
}

group = "modern.colorthief"
version = "0.3.0"

repositories {
    mavenCentral()
}

sourceSets {
    main {
        java.srcDir("src/io")
    }
    test {
        kotlin.srcDir("src/test/kotlin")
    }
}

dependencies {
    implementation(kotlin("stdlib"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.12.2")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
    testImplementation(kotlin("test-junit5"))
}

java {
    toolchain { languageVersion.set(JavaLanguageVersion.of(26)) }
}

kotlin {
    jvmToolchain(26)
}

kotlin.compilerOptions {
    freeCompilerArgs.add("-XXLanguage:+UnnamedLocalVariables")
}

tasks.withType<Test> {
    useJUnitPlatform {
        // Explicitly include test classes to ensure discovery works
        includeEngines("junit-jupiter")
    }

    val nativeLibPath = file("native").absolutePath
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    jvmArgs("-Djava.library.path=$nativeLibPath")
    systemProperty("native.lib.path", nativeLibPath)

    testLogging {
        events("passed", "skipped", "failed")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }

    // Gradle 9.5.1 is strict about test discovery — disable to see actual errors
    failOnNoDiscoveredTests = false
}
