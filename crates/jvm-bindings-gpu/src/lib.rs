use jni::EnvUnowned;
use jni::errors::{Error, ThrowRuntimeExAndDefault};
use jni::objects::{JByteArray, JObject};
use jni::sys::jint;

/// Extract a palette using GPU compute.
#[unsafe(no_mangle)]
pub extern "system" fn Java_modern_colorthief_ColorthiefGpu_getPalette<'a>(
    mut env: EnvUnowned<'a>,
    _class: JObject<'a>,
    pixels: JByteArray<'a>,
    width: jint,
    height: jint,
    color_count: jint,
    quality: jint,
) -> JObject<'a> {
    env.with_env(|env| -> jni::errors::Result<JObject<'a>> {
        let len = pixels.len(&env)?;
        let len = len.max(0) as usize;
        let mut pixel_data = vec![0i8; len];
        pixels.get_region(&env, 0, &mut pixel_data)?;

        let u8_data: Vec<u8> = pixel_data.iter().copied().map(|b| b as u8).collect();

        let colors = modern_colorthief_core_gpu::extract_palette_from_buffer(
            &u8_data,
            width as u32,
            height as u32,
            color_count as u8,
            quality as u8,
        )
        .map_err(|_| Error::JavaException)?;

        let result_array = env.new_object_array(
            colors.len(),
            jni::jni_str!("[B"),
            JObject::null(),
        )?;

        for (i, (r, g, b)) in colors.into_iter().enumerate() {
            let color_array = env.byte_array_from_slice(&[r, g, b])?;
            result_array.set_element(&env, i, color_array.into())?;
        }

        Ok(result_array.into())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

/// Extract the dominant color using GPU compute.
#[unsafe(no_mangle)]
pub extern "system" fn Java_modern_colorthief_ColorthiefGpu_getColor<'a>(
    mut env: EnvUnowned<'a>,
    _class: JObject<'a>,
    pixels: JByteArray<'a>,
    width: jint,
    height: jint,
    quality: jint,
) -> JObject<'a> {
    env.with_env(|env| -> jni::errors::Result<JObject<'a>> {
        let len = pixels.len(&env)?;
        let len = len.max(0) as usize;
        let mut pixel_data = vec![0i8; len];
        pixels.get_region(&env, 0, &mut pixel_data)?;

        let u8_data: Vec<u8> = pixel_data.iter().copied().map(|b| b as u8).collect();

        let colors = modern_colorthief_core_gpu::extract_palette_from_buffer(
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
