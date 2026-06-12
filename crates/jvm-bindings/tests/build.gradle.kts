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

tasks.named<Test>("test") {
    useJUnitPlatform()

    val nativeLibPath = file("native").absolutePath
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    jvmArgs("-Djava.library.path=$nativeLibPath")
    systemProperty("native.lib.path", nativeLibPath)

    // Explicitly scan all test class directories
    filter {
        includeTestsInPackages("modern.colorthief")
        includeTestsInPackages("io.baseplate_admin.modern_colorthief")
    }

    testLogging {
        events("passed", "skipped", "failed")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
        showStandardStreams = true
        showCauses = true
        showStackTraces = true
    }

    doFirst {
        logger.lifecycle("Test classpath: ${classpath.asPath}")
        logger.lifecycle("Scan filters: ${filter.includes}")
    }
}
