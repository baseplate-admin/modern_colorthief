use jni::EnvUnowned;
use jni::errors::ThrowRuntimeExAndDefault;
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
    env.with_env(|env| -> jni::errors::Result<_> {
        let result = extract_palette_jvm(
            env,
            &pixels,
            width as u32,
            height as u32,
            color_count as u8,
            quality as u8,
        )?;
        Ok(result)
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
    env.with_env(|env| -> jni::errors::Result<_> {
        let result = extract_color_jvm(
            env,
            &pixels,
            width as u32,
            height as u32,
            quality as u8,
        )?;
        Ok(result)
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

fn jni_err<T>(result: jni::errors::Result<T>) -> Result<T, String> {
    result.map_err(|e| format!("{}", e))
}

fn extract_palette_jvm<'a>(
    env: &mut jni::Env<'a>,
    pixels: &JByteArray<'a>,
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<JObject<'a>, String> {
    let len = jni_err(env.get_array_length(pixels))?.max(0) as usize;
    let expected = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    if len < expected {
        return Err(format!(
            "Pixel buffer too small: expected {} bytes, got {}",
            expected, len
        ));
    }
    let mut pixel_data = vec![0i8; len];
    jni_err(env.get_byte_array_region(pixels, 0, &mut pixel_data))?;

    let u8_data: Vec<u8> = pixel_data.iter().copied().map(|b| b as u8).collect();

    let colors = modern_colorthief_core_cpu::extract_palette_from_buffer(
        &u8_data,
        width,
        height,
        color_count,
        quality,
    )?;

    let result_array =
        jni_err(env.new_object_array(colors.len() as jsize, jni::jni_str!("[B"), JObject::null()))?;

    for (i, (r, g, b)) in colors.into_iter().enumerate() {
        let color_array = jni_err(env.byte_array_from_slice(&[r, g, b]))?;
        jni_err(env.set_object_array_element(&result_array, i as jsize, color_array))?;
    }

    Ok(result_array.into())
}

fn extract_color_jvm<'a>(
    env: &mut jni::Env<'a>,
    pixels: &JByteArray<'a>,
    width: u32,
    height: u32,
    quality: u8,
) -> Result<JObject<'a>, String> {
    let len = jni_err(env.get_array_length(pixels))?.max(0) as usize;
    let expected = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    if len < expected {
        return Err(format!(
            "Pixel buffer too small: expected {} bytes, got {}",
            expected, len
        ));
    }
    let mut pixel_data = vec![0i8; len];
    jni_err(env.get_byte_array_region(pixels, 0, &mut pixel_data))?;

    let u8_data: Vec<u8> = pixel_data.iter().copied().map(|b| b as u8).collect();

    let colors =
        modern_colorthief_core_cpu::extract_palette_from_buffer(&u8_data, width, height, 5, quality)?;

    let (r, g, b) = colors
        .first()
        .copied()
        .ok_or("Image contains no colors".to_string())?;

    let result = jni_err(env.byte_array_from_slice(&[r, g, b]))?;
    Ok(result.into())
}
