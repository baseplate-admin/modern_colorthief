require 'rbconfig'
require 'pathname'
require 'singleton'

# Resolve the shared library name across platforms:
# Linux: libmodern_colorthief.so
# macOS: libmodern_colorthief.bundle
# Windows: libmodern_colorthief.dll
LIB_NAME = case RbConfig::CONFIG['host_os']
when /linux/ then 'libmodern_colorthief.so'
when /darwin|mac/ then 'libmodern_colorthief.bundle'
when /windows|mingw/ then 'libmodern_colorthief.dll'
else 'libmodern_colorthief.so'
end

require LIB_NAME
