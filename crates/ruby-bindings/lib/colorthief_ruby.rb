# frozen_string_literal: true

require "rbconfig"

# Load the native extension.
# When installed as a gem, the compiled library is in lib/modern_colorthief/.
# When running from source, it's expected in lib/modern_colorthief/ next to this file.

LIB_DIR = File.expand_path("modern_colorthief", __dir__)

lib_name = case RbConfig::CONFIG["host_os"]
when /linux/ then "libmodern_colorthief.so"
when /darwin|mac/ then "libmodern_colorthief.bundle"
when /windows|mingw/ then "modern_colorthief.dll"
else "libmodern_colorthief.so"
end

lib_path = File.join(LIB_DIR, lib_name)

if File.exist?(lib_path)
  require lib_path
else
  # Fallback: try loading from Ruby load path (for gem installs)
  require "modern_colorthief/#{lib_name}"
end
