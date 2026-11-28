# frozen_string_literal: true

require 'rspec'
require 'open3'

GPU_AVAILABLE = begin
  require_relative '../lib/colorthief_gpu'

  # GPU extension loaded — now verify Vulkan init actually works by running
  # a probe in a subprocess. This isolates segfaults during Vulkan instance
  # creation (known with SwiftShader/llvmpipe on CI) from the test harness.
  probe_script = File.expand_path('gpu_probe.rb', __dir__)

  _, stderr, status = Open3.capture3(
    RbConfig.ruby, probe_script,
    chdir: __dir__
  )

  if status.success?
    warn 'GPU probe: OK'
    true
  else
    exit_code = status.exitstatus || 'crashed'
    warn "GPU probe failed (exit #{exit_code}): #{stderr.strip}"
    false
  end
rescue LoadError, StandardError => e
  warn "GPU extension not available: #{e.class} - #{e.message}"
  false
end

RSpec.configure do |config|
  unless GPU_AVAILABLE
    config.filter_run_excluding gpu_required: true
    warn 'Skipping GPU tests — extension not loaded'
  end
end
