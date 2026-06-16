# frozen_string_literal: true

require "rspec"

# Load the native CPU extension (handles cross-platform naming internally)
begin
  require_relative "lib/colorthief_ruby"
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
end
