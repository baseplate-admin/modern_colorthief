# frozen_string_literal: true

require "rbconfig"

LIB_DIR = File.expand_path("modern_colorthief", __dir__)

lib_name = case RbConfig::CONFIG["host_os"]
when /linux/ then "modern_colorthief.so"
when /darwin|mac/ then "modern_colorthief.bundle"
when /windows|mingw/ then "modern_colorthief.so"
else "modern_colorthief.so"
end

lib_path = File.join(LIB_DIR, lib_name)

if File.exist?(lib_path)
  require lib_path
else
  require "modern_colorthief/#{lib_name}"
end
