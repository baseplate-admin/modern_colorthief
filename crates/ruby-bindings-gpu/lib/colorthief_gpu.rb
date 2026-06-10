require 'rbconfig'
require 'pathname'

# Resolve the shared library name across platforms:
# Linux: libmodern_colorthief_gpu.so
# macOS: libmodern_colorthief_gpu.bundle
# Windows: libmodern_colorthief_gpu.dll
LIB_NAME = case RbConfig::CONFIG['host_os']
when /linux/ then 'libmodern_colorthief_gpu.so'
when /darwin|mac/ then 'libmodern_colorthief_gpu.bundle'
when /windows|mingw/ then 'libmodern_colorthief_gpu.dll'
else 'libmodern_colorthief_gpu.so'
end

require LIB_NAME
