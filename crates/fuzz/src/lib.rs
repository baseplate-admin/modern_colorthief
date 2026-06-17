//! Fuzz / property-based tests for palette extraction.
//!
//! Generates random images and verifies:
//! 1. CPU and GPU backends produce the same palette for the same input.
//! 2. Our CPU backend produces comparable results to `color_thief` (RazrFalcon reference).

#[cfg(test)]
mod tests;
