plugins {
    `java`
    kotlin("jvm") version "2.1.0"
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
    sourceCompatibility = JavaVersion.VERSION_26
    targetCompatibility = JavaVersion.VERSION_26
}

kotlin {
    jvmToolchain(26)
}

kotlin.compilerOptions {
    jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_23)
}

tasks.withType<JavaCompile>().configureEach {
    if (name.contains("Test")) {
        options.release.set(23)
    }
}

testing {
    suites {
        named<JvmTestSuite>("test") {
            useJUnitJupiter("5.12.2")
        }
    }
}

tasks.named<Test>("test") {
    testLogging {
        events("passed", "skipped", "failed")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }
}
