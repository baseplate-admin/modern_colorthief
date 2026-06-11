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
    testImplementation("org.junit.jupiter:junit-jupiter:5.12.2")
    testImplementation(kotlin("test-junit5"))
}

java {
    toolchain { languageVersion.set(JavaLanguageVersion.of(26)) }
}

kotlin {
    jvmToolchain(26)
}

testing {
    suites {
        named<JvmTestSuite>("test") {
            useJUnitJupiter("5.12.2")
        }
    }
}

tasks.named<Test>("test") {
    val nativeLibPath = file("native").absolutePath
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    jvmArgs("-Djava.library.path=$nativeLibPath")
    environment("LD_LIBRARY_PATH", nativeLibPath)
    environment("JAVA_LIBRARY_PATH", nativeLibPath)
    testLogging {
        events("passed", "skipped", "failed")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }
}
