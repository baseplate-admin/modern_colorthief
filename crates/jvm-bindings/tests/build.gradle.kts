plugins {
    `java`
    `maven-publish`
    signing
    kotlin("jvm") version "2.4.0"
    id("io.github.gradle-nexus.publish-plugin") version "2.0.0"
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
    withSourcesJar()
    withJavadocJar()
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

    filter {
        includeTestsMatching("modern.colorthief.*")
        includeTestsMatching("io.baseplate_admin.modern_colorthief.*")
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
    }
}

// Include native libraries in JAR under META-INF/natives/
tasks.jar {
    from("native") {
        into("META-INF/natives")
        include("*")
    }
}

// Configure publishing artifacts
publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])

            pom {
                name.set("Modern ColorThief JVM")
                description.set("Fast color palette extraction from images via JVM bindings")
                url.set("https://github.com/baseplate-admin/modern_colorthief")

                scm {
                    connection.set("scm:git:git://github.com/baseplate-admin/modern_colorthief.git")
                    developerConnection.set("scm:git:ssh://github.com/baseplate-admin/modern_colorthief.git")
                    url.set("https://github.com/baseplate-admin/modern_colorthief")
                }

                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                }

                developers {
                    developer {
                        id.set("baseplate-admin")
                        name.set("baseplate-admin")
                    }
                }
            }
        }
    }

    if (findProperty("signingInMemoryKey") != null) {
        signing {
            useInMemoryPgpKeys(
                findProperty("signingInMemoryKey") as String,
                findProperty("signingInMemoryPassword") as String
            )
            sign(publishing.publications["maven"])
        }
    }
}

nexusPublishing {
    packageGroup.set("modern")
    repositoryDescription.set("modern_colorthief JVM bindings ${project.version}")
    useStaging.set(!gradle.startParameter.isDryRun)
}
