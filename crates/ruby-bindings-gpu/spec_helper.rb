# spec_helper.rb - RSpec configuration for modern_colorthief_gpu

require 'rspec'

# Load the native GPU extension.
# The compiled .so/.bundle/.dll lives under lib/ after `rake compile`.
begin
  require_relative 'lib/modern_colorthief_gpu'
rescue LoadError
  # Extension not yet compiled; tests will be skipped at runtime.
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
