# frozen_string_literal: true

require 'rspec'
require 'open3'

$gpu_available = false

begin
  require_relative '../lib/colorthief_gpu'

  # GPU extension loaded — now verify Vulkan init actually works by running
  # a probe in a subprocess. This isolates segfaults during Vulkan instance
  # creation (known with SwiftShader/llvmpipe on CI) from the test harness.
  ruby_exe = RbConfig::CONFIG['ruby']
  probe_script = File.expand_path('gpu_probe.rb', __dir__)

  stdout, stderr, status = Open3.capture3(
    ruby_exe, probe_script,
    chdir: __dir__
  )

  if status.success?
    $gpu_available = true
    warn "GPU probe: OK"
  else
    exit_code = status.exitstatus rescue 'crashed'
    warn "GPU probe failed (exit #{exit_code}): #{stderr.strip}"
  end
rescue LoadError, StandardError => e
  warn "GPU extension not available: #{e.class} - #{e.message}"
end

RSpec.configure do |config|
  unless $gpu_available
    config.filter_run_excluding :gpu_required => true
    warn "Skipping GPU tests — extension not loaded"
  end
end
