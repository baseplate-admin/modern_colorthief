# frozen_string_literal: true

require "rspec"

# Load the native GPU extension with correct cross-platform naming.
GPU_LIB_NAME = case RbConfig::CONFIG["host_os"]
when /linux/ then "modern_colorthief_gpu.so"
when /darwin|mac/ then "modern_colorthief_gpu.bundle"
when /windows|mingw/ then "modern_colorthief_gpu.dll"
else "modern_colorthief_gpu.so"
end

begin
  lib_dir = File.expand_path("../lib", __dir__)
  lib_path = File.join(lib_dir, GPU_LIB_NAME)
  if File.exist?(lib_path)
    require lib_path
  else
    require GPU_LIB_NAME
  end
  $gpu_available = true
rescue Exception
  # Extension not yet compiled or failed to initialize; tests will be skipped.
  $gpu_available = false
end

RSpec.configure do |config|
  config.expect_with :rspec do |expectations|
    expectations.include_chain_clauses_in_custom_matcher_descriptions = true
  end
  config.mock_with :rspec do |mocks|
    mocks.verify_partial_doubles = true
  end
  config.shared_context_metadata_behavior = :apply_to_host_groups

  # Disable rspec-warnings if available
  if defined?(RSpec::Warnings)
    config.warnings = true
  end

  # Run in parallel if the gem is available
  if defined?(RSpec::Core::Parallelization)
    config.parallelize_across 2 workers
  end
end
