# frozen_string_literal: true

require "mkmf"

# This extconf is used by the gem build process.
# The actual Rust compilation is handled by the Rakefile via `cargo build`.
create_makefile("modern_colorthief")
