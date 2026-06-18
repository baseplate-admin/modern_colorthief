//! Fuzz / property-based tests for palette extraction.
//!
//! Generates random images and verifies:
//! 1. CPU and GPU backends produce the same palette for the same input.
//! 2. Our CPU backend produces comparable results to `color_thief` (RazrFalcon reference).

#[cfg(test)]
mod tests;

/// Compare two palettes for approximate equality.
pub fn palettes_approximately_equal(a: &[(u8, u8, u8)], b: &[(u8, u8, u8)], tolerance: u32) -> bool {
    if a.is_empty() && b.is_empty() {
        return true;
    }
    if a.is_empty() || b.is_empty() {
        return false;
    }
    for ca in a {
        let matched = b.iter().any(|cb| {
            let dr = (ca.0 as i32 - cb.0 as i32).unsigned_abs();
            let dg = (ca.1 as i32 - cb.1 as i32).unsigned_abs();
            let db = (ca.2 as i32 - cb.2 as i32).unsigned_abs();
            dr + dg + db <= tolerance
        });
        if !matched {
            return false;
        }
    }
    for cb in b {
        let matched = a.iter().any(|ca| {
            let dr = (ca.0 as i32 - cb.0 as i32).unsigned_abs();
            let dg = (ca.1 as i32 - cb.1 as i32).unsigned_abs();
            let db = (ca.2 as i32 - cb.2 as i32).unsigned_abs();
            dr + dg + db <= tolerance
        });
        if !matched {
            return false;
        }
    }
    true
}
