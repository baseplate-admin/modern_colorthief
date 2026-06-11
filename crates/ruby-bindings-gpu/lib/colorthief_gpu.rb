# frozen_string_literal: true

require "rbconfig"

# Load the native GPU extension.
# When installed as a gem, the compiled library is in lib/modern_colorthief_gpu/.
# When running from source, it's expected in lib/modern_colorthief_gpu/ next to this file.

LIB_DIR = File.expand_path("modern_colorthief_gpu", __dir__)

lib_name = case RbConfig::CONFIG["host_os"]
when /linux/ then "libmodern_colorthief_gpu.so"
when /darwin|mac/ then "libmodern_colorthief_gpu.bundle"
when /windows|mingw/ then "modern_colorthief_gpu.dll"
else "libmodern_colorthief_gpu.so"
end

lib_path = File.join(LIB_DIR, lib_name)

if File.exist?(lib_path)
  require lib_path
else
  # Fallback: try loading from Ruby load path (for gem installs)
  require "modern_colorthief_gpu/#{lib_name}"
end
