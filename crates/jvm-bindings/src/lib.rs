use jni::EnvUnowned;
use jni::errors::{Error, ThrowRuntimeExAndDefault};
use jni::objects::{JByteArray, JObject};
use jni::sys::{jint, jsize};

/// Extract a palette of dominant colors from raw RGBA pixel data.
#[unsafe(no_mangle)]
pub extern "system" fn Java_modern_colorthief_Colorthief_getPalette<'a>(
    mut env: EnvUnowned<'a>,
    _class: JObject<'a>,
    pixels: JByteArray<'a>,
    width: jint,
    height: jint,
    color_count: jint,
    quality: jint,
) -> JObject<'a> {
    env.with_env(|env| -> jni::errors::Result<JObject<'a>> {
        let len = env.get_array_length(&pixels)?;
        let len = len.max(0) as usize;
        let expected = (width as usize).saturating_mul(height as usize).saturating_mul(4);
        if len < expected {
            return Err(Error::JavaException);
        }
        let mut pixel_data = vec![0i8; len];
        env.get_byte_array_region(&pixels, 0, &mut pixel_data)?;

        let u8_data: Vec<u8> = pixel_data.iter().copied().map(|b| b as u8).collect();

        let colors = modern_colorthief_core_cpu::extract_palette_from_buffer(
            &u8_data,
            width as u32,
            height as u32,
            color_count as u8,
            quality as u8,
        )
        .map_err(|_| Error::JavaException)?;

        let result_array = env.new_object_array(
            colors.len() as jsize,
            jni::jni_str!("[B"),
            JObject::null(),
        )?;

        for (i, (r, g, b)) in colors.into_iter().enumerate() {
            let color_array: JObject = env.byte_array_from_slice(&[r, g, b])?.into();
            env.set_object_array_element(&result_array, i as jsize, color_array)?;
        }

        Ok(result_array.into())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

/// Extract the dominant color from raw RGBA pixel data.
#[unsafe(no_mangle)]
pub extern "system" fn Java_modern_colorthief_Colorthief_getColor<'a>(
    mut env: EnvUnowned<'a>,
    _class: JObject<'a>,
    pixels: JByteArray<'a>,
    width: jint,
    height: jint,
    quality: jint,
) -> JObject<'a> {
    env.with_env(|env| -> jni::errors::Result<JObject<'a>> {
        let len = env.get_array_length(&pixels)?;
        let len = len.max(0) as usize;
        let expected = (width as usize).saturating_mul(height as usize).saturating_mul(4);
        if len < expected {
            return Err(Error::JavaException);
        }
        let mut pixel_data = vec![0i8; len];
        env.get_byte_array_region(&pixels, 0, &mut pixel_data)?;

        let u8_data: Vec<u8> = pixel_data.iter().copied().map(|b| b as u8).collect();

        let colors = modern_colorthief_core_cpu::extract_palette_from_buffer(
            &u8_data,
            width as u32,
            height as u32,
            5,
            quality as u8,
        )
        .map_err(|_| Error::JavaException)?;

        let (r, g, b) = colors
            .first()
            .copied()
            .ok_or(Error::JavaException)?;

        let result = env.byte_array_from_slice(&[r, g, b])?;
        Ok(result.into())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}
