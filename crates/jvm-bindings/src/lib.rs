//! JVM (JNI) bindings for `modern_colorthief`.
//!
//! The Java class is `io.baseplate_admin.modern_colorthief.Colorthief` (see
//! `tests/src/io/baseplate_admin/modern_colorthief/Colorthief.java`), so the
//! JNI symbol names below are mangled to match that fully-qualified name.
//!
//! Error handling: core errors carry their message into the Java exception
//! (validation errors become `IllegalArgumentException`, backend/extraction
//! errors become `RuntimeException`), and the method returns `null`.

use jni::objects::{JByteArray, JObject};
use jni::strings::JNIString;
use jni::sys::jint;
use jni::{Env, EnvUnowned};

use modern_colorthief_core::ColorthiefError;

fn is_validation_err(err: &ColorthiefError) -> bool {
    matches!(
        err,
        ColorthiefError::EmptyBuffer
            | ColorthiefError::NoPixels
            | ColorthiefError::InvalidDimensions
    )
}

fn jni_err_from_core(err: &ColorthiefError) -> String {
    format!("modern_colorthief: {err}")
}

fn extract_palette_jni<'a>(
    env: &mut Env<'a>,
    pixels: JByteArray<'a>,
    width: jint,
    height: jint,
    color_count: u8,
    quality: u8,
) -> jni::errors::Result<Vec<(u8, u8, u8)>> {
    let len = pixels.len(env)? as usize;
    let mut pixel_data = vec![0i8; len];
    pixels.get_region(env, 0, &mut pixel_data)?;
    let u8_data: Vec<u8> = pixel_data.iter().copied().map(|b| b as u8).collect();

    match modern_colorthief_core_cpu::extract_palette_from_buffer(
        &u8_data,
        width as u32,
        height as u32,
        color_count,
        quality,
    ) {
        Ok(colors) => Ok(colors),
        Err(core_err) => {
            // Throw with the right class and message, then return an error
            // so the policy knows something went wrong.
            let class = if is_validation_err(&core_err) {
                jni::jni_str!("java/lang/IllegalArgumentException")
            } else {
                jni::jni_str!("java/lang/RuntimeException")
            };
            let _ = env.throw_new(class, JNIString::new(jni_err_from_core(&core_err)));
            Err(jni::errors::Error::JavaException)
        }
    }
}

/// Extract a palette of dominant colors from raw RGBA pixel data.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_baseplate_admin_modern_colorthief_Colorthief_getPalette<'a>(
    mut env: EnvUnowned<'a>,
    _class: JObject<'a>,
    pixels: JByteArray<'a>,
    width: jint,
    height: jint,
    color_count: jint,
    quality: jint,
) -> JObject<'a> {
    env.with_env(|env| -> jni::errors::Result<JObject<'a>> {
        let colors =
            extract_palette_jni(env, pixels, width, height, color_count as u8, quality as u8)?;

        let result_array =
            env.new_object_array(colors.len() as i32, jni::jni_str!("[B"), JObject::null())?;

        for (i, (r, g, b)) in colors.into_iter().enumerate() {
            let color_array = env.byte_array_from_slice(&[r, g, b])?;
            result_array.set_element(env, i, color_array)?;
        }

        Ok(result_array.into())
    })
    .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

/// Extract the dominant color from raw RGBA pixel data.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_baseplate_admin_modern_colorthief_Colorthief_getColor<'a>(
    mut env: EnvUnowned<'a>,
    _class: JObject<'a>,
    pixels: JByteArray<'a>,
    width: jint,
    height: jint,
    quality: jint,
) -> JObject<'a> {
    env.with_env(|env| -> jni::errors::Result<JObject<'a>> {
        let palette = extract_palette_jni(env, pixels, width, height, 5, quality as u8)?;

        let (r, g, b) = match modern_colorthief_core::bindings::dominant_color(&palette) {
            Ok(c) => c,
            Err(e) => {
                let _ = env.throw_new(
                    jni::jni_str!("java/lang/RuntimeException"),
                    JNIString::new(jni_err_from_core(&e)),
                );
                return Err(jni::errors::Error::JavaException);
            }
        };

        let result = env.byte_array_from_slice(&[r, g, b])?;
        Ok(result.into())
    })
    .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}
