# frozen_string_literal: true

require "rbconfig"

# Determine expected library base name
lib_base = "modern_colorthief"

# Search for the library file in lib/, handling lib-prefixed and non-prefixed names
lib_dir = __dir__
lib_ext = RbConfig::CONFIG["DLEXT"]
lib_candidates = [
  "#{lib_base}.#{lib_ext}",       # modern_colorthief.so, .bundle, .dylib
  "lib#{lib_base}.#{lib_ext}",    # libmodern_colorthief.so, .dylib
  "#{lib_base}.so",               # Linux
  "lib#{lib_base}.so",
  "#{lib_base}.dylib",            # macOS
  "lib#{lib_base}.dylib",
  "#{lib_base}.bundle",           # Ruby ext
  "lib#{lib_base}.bundle",
  "#{lib_base}.dll",              # Windows
  "lib#{lib_base}.dll",
  "#{lib_base}.so.dll",           # Windows rb-sys
  "lib#{lib_base}.so.dll"
]

lib_file = lib_candidates.find { |name| File.exist?(File.join(lib_dir, name)) }

if lib_file
  require File.join(lib_dir, lib_file)
else
  require lib_base
end
