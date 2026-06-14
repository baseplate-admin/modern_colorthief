# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name          = "modern_colorthief"
  spec.version       = "0.3.0"
  spec.authors       = ["Modern Colorthief Contributors"]
  spec.summary       = "High-performance color palette extraction from images"
  spec.description   = "Rust-powered color palette extraction using median cut algorithm"
  spec.homepage      = "https://github.com/baseplate-admin/modern_colorthief"
  spec.license       = "MIT"

  spec.required_ruby_version = ">= 2.7.0"

  spec.files = Dir["lib/**/*", "ext/**/*", "*.gemspec", "Rakefile"]
  spec.require_paths = ["lib"]

  spec.extensions = ["ext/extconf.rb"]

  spec.add_dependency "rb_sys", "~> 0.9"

  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rake-compiler", "~> 1.3"
 end
