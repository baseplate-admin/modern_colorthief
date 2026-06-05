use jni::objects::{JByteArray, JObject, JString};
use jni::sys::{jbyteArray, jint, jsize};
use jni::JNIEnv;

/// Extract a palette of dominant colors from raw RGBA pixel data.
///
/// @param pixels byte[] - Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width int - Image width in pixels
/// @param height int - Image height in pixels
/// @param colorCount int - Number of colors to extract
/// @param quality int - Sampling quality
/// @return byte[][] - Array of [R, G, B] color arrays
#[no_mangle]
pub extern "system" fn Java_colorthief_Colorthief_getPalette(
    mut env: JNIEnv,
    _class: JObject,
    pixels: JByteArray,
    width: jint,
    height: jint,
    color_count: jint,
    quality: jint,
) -> JObject {
    match extract_palette_jvm(
        &mut env,
        pixels,
        width as u32,
        height as u32,
        color_count as u8,
        quality as u8,
    ) {
        Ok(result) => result,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("{}", e));
            JObject::null()
        }
    }
}

/// Extract the dominant color from raw RGBA pixel data.
///
/// @param pixels byte[] - Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width int - Image width in pixels
/// @param height int - Image height in pixels
/// @param quality int - Sampling quality
/// @return byte[] - [R, G, B] color array
#[no_mangle]
pub extern "system" fn Java_colorthief_Colorthief_getColor(
    mut env: JNIEnv,
    _class: JObject,
    pixels: JByteArray,
    width: jint,
    height: jint,
    quality: jint,
) -> JObject {
    match extract_color_jvm(
        &mut env,
        pixels,
        width as u32,
        height as u32,
        quality as u8,
    ) {
        Ok(result) => result,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("{}", e));
            JObject::null()
        }
    }
}

fn extract_palette_jvm(
    env: &mut JNIEnv,
    pixels: JByteArray,
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<JObject, String> {
    let len = env.get_array_length(&pixels)? as usize;
    let mut pixel_data = vec![0u8; len];
    env.get_byte_array_region(&pixels, 0, &mut pixel_data)?;

    let colors = colorthief_core::extract_palette_from_buffer(
        &pixel_data,
        width,
        height,
        color_count,
        quality,
    )?;

    let byte_array_class = env.find_class("java/lang/Byte")?;
    let result_array = env.new_object_array(
        colors.len() as jsize,
        "[B",
        JObject::null(),
    )?;

    for (i, (r, g, b)) in colors.into_iter().enumerate() {
        let color_array = env.byte_array_from_slice(&[r, g, b])?;
        env.set_object_array_element(&result_array, i as jsize, color_array)?;
    }

    Ok(result_array.into())
}

fn extract_color_jvm(
    env: &mut JNIEnv,
    pixels: JByteArray,
    width: u32,
    height: u32,
    quality: u8,
) -> Result<JObject, String> {
    let len = env.get_array_length(&pixels)? as usize;
    let mut pixel_data = vec![0u8; len];
    env.get_byte_array_region(&pixels, 0, &mut pixel_data)?;

    let colors = colorthief_core::extract_palette_from_buffer(
        &pixel_data,
        width,
        height,
        5,
        quality,
    )?;

    let (r, g, b) = colors
        .first()
        .copied()
        .ok_or("Image contains no colors")?;

    let result = env.byte_array_from_slice(&[r, g, b])?;
    Ok(result.into())
}
