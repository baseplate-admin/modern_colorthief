# frozen_string_literal: true

require "rspec"

$gpu_available = false

begin
  require_relative "../lib/colorthief_gpu"
  $gpu_available = true
rescue LoadError, StandardError => e
  warn "GPU extension not available: #{e.class} - #{e.message}"
end

RSpec.configure do |config|
  unless $gpu_available
    config.filter_run_excluding :gpu_required => true
    warn "Skipping GPU tests — extension not loaded"
  end
end
