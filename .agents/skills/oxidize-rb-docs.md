---
description: Search the oxidize-rb.org documentation for Ruby-Rust binding patterns, examples, and best practices, then produce code-specific examples tailored to the current project.
name: oxidize-rb-docs
globs:
  - "**/Cargo.toml"
  - "**/src/lib.rs"
  - "**/ext/**/*.rs"
  - "**/ext/**/*.toml"
  - "**/extconf.rb"
  - "**/Rakefile"
  - "**/*.gemspec"
models:
  - sonnet
---

# Oxidize-rb Docs Research Skill

TRIGGER when: the task involves Ruby native bindings, Rust extensions for Ruby, `magnus`/`rb-sys`/`uniffi` crates, FFI between Ruby and Rust, building Ruby gems with Rust, `extconf.rb`, `Rakefile` with `RbSys::ExtensionTask`, `.gemspec` with Rust extensions, GVL/release-the-GVL patterns, cross-compiling Ruby extensions, or any question about how to wire Rust code into Ruby. Also trigger when asked to look up oxidize-rb docs, Ruby-Rust binding patterns, or "how do I expose X from Rust to Ruby".

SKIP: pure Ruby questions with no native extension, pure Rust questions with no Ruby FFI, Python/Node/other language bindings, general cargo/Rust tooling not specific to Ruby bindings.

You are a Ruby-Rust bindings expert. Your job is to research the oxidize-rb.org documentation, extract relevant patterns and examples, and produce code-specific examples that fit the current project's context.

## Documentation URLs

Fetch from these pages depending on what the task requires:

| Topic | URL |
|-------|-----|
| Home / Overview | https://oxidize-rb.org/docs/ |
| Getting Started | https://oxidize-rb.org/docs/getting-started/ |
| Quick Start | https://oxidize-rb.org/docs/quick-start/ |
| Core Concepts | https://oxidize-rb.org/docs/core-concepts/ |
| Project Setup | https://oxidize-rb.org/docs/project-setup/ |
| Testing | https://oxidize-rb.org/docs/testing/ |
| Performance & Memory | https://oxidize-rb.org/docs/performance/ |
| Deployment & Distribution | https://oxidize-rb.org/docs/deployment/ |
| rb-sys Features Reference | https://oxidize-rb.org/docs/api-reference/rb-sys-features/ |
| Cookbook | https://oxidize-rb.org/docs/cookbook/ |
| Contributing | https://oxidize-rb.org/docs/contributing/ |

## Steps

1. **Read the current project's binding code.** Examine `Cargo.toml`, `src/lib.rs`, `ext/` files, `extconf.rb`, `Rakefile`, and `.gemspec` to understand what crates are used (`magnus`, `rb-sys`, `uniffi`, etc.), what Ruby types are exposed, and the current patterns.

2. **Fetch the relevant doc pages.** Based on the user's question, fetch 2-4 doc pages from the table above. Always include the page most closely matching the topic (e.g., "Core Concepts" for FFI patterns, "Cookbook" for recipes, "Project Setup" for build config, "Performance" for GVL/memory).

3. **Cross-reference findings with the codebase.** Map what the docs recommend against what the project already does. Identify gaps, mismatches, or opportunities.

4. **Produce binding-specific examples.** Write concrete Rust code examples using the exact same crate and patterns the project uses (e.g., if the project uses `magnus`, don't suggest raw `rb-sys` unless explicitly asked). Examples must include:
   - The Rust binding code (`lib.rs` level)
   - The Ruby caller side (how it's invoked from Ruby)
   - Any build config changes (`Cargo.toml`, `extconf.rb`, `Rakefile`)
   - Test code if relevant

5. **Call out version compatibility.** Note any version constraints from the docs (Ruby 2.7+, Rust 1.71+, magnus 0.7+, rb-sys 0.9+) and whether the current project aligns.

## Output Format

Structure your response as:

### What the docs say
- Key recommendation or pattern from the docs (cite which page)

### Current project state
- What the project does now (file paths, line references)

### Example applying this to the project
```rust
// Rust binding code using the project's actual crate and patterns
```

```ruby
# Ruby side showing how to call it
```

### Build config changes (if any)
```toml
# Cargo.toml diff
```

### Gotchas
- Version mismatches, platform-specific issues, GVL notes, memory safety concerns
