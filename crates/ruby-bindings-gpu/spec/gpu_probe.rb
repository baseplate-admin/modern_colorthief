#!/usr/bin/env ruby
# frozen_string_literal: true

# Subprocess probe: tests whether the GPU extension can actually do Vulkan init.
# Run this as a separate process so that a segfault during Vulkan init
# (known with SwiftShader/llvmpipe) doesn't crash the parent test harness.
# Exit 0 = GPU works, exit 1 = GPU unavailable or crashed.

require_relative '../lib/colorthief_gpu'

# Create a tiny 2x2 red image (8 bytes = 4 pixels * 4 channels)
pixels = [255, 0, 0, 255] * 4
pixels = pixels.pack('C*')

# This triggers Vulkan init — if it segfaults, the process crashes.
ColorthiefGpu.get_palette(pixels, 2, 2, 5, 1)

puts "GPU probe: OK"
exit(0)
