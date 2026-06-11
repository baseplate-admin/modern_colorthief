# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name          = "modern_colorthief"
  spec.version       = "0.3.0"
  spec.authors       = ["Modern Colorthief Contributors"]
  spec.summary       = "High-performance color palette extraction from images"
  spec.description   = "Rust-powered color palette extraction using median cut algorithm"
  spec.homepage      = "https://github.com/rustiq/luma"
  spec.license       = "MIT"

  spec.required_ruby_version = ">= 2.7.0"

  spec.files = Dir["lib/**/*", "ext/**/*", "*.gemspec", "Rakefile", "README.md"]
  spec.require_paths = ["lib"]

  spec.extensions = ["ext/extconf.rb"]

  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rake-compiler", "~> 1.2"
  spec.add_development_dependency "rspec", "~> 3.13"
end
