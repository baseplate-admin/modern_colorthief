// See: https://oxidize-rb.org/docs/api-reference/test-helpers
fn main() -> Result<(), Box<dyn std::error::Error>> {
    rb_sys_env::activate()?;
    Ok(())
}
