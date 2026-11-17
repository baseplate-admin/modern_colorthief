package io.baseplate_admin.modern_colorthief

import kotlinx.coroutines.suspendCancellableCoroutine
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

/**
 * Kotlin coroutine (suspend) wrappers around the native [Colorthief] API.
 *
 * ```kotlin
 * val palette: Array<ByteArray> = colorthiefGetPalette(pixels, w, h, 5, 1)
 * val color:   ByteArray       = colorthiefGetColor(pixels, w, h, 1)
 * ```
 */
suspend fun colorthiefGetPalette(
    pixels: ByteArray,
    width: Int,
    height: Int,
    colorCount: Int = 5,
    quality: Int = 1,
): Array<ByteArray> = suspendCancellableCoroutine { cont ->
    val future = ColorthiefAsync.getPalette(pixels, width, height, colorCount, quality)
    cont.invokeOnCancellation { future.cancel(true) }
    future.whenComplete { result, ex ->
        if (ex != null) cont.resumeWithException(ex)
        else cont.resume(result)
    }
}

suspend fun colorthiefGetColor(
    pixels: ByteArray,
    width: Int,
    height: Int,
    quality: Int = 1,
): ByteArray = suspendCancellableCoroutine { cont ->
    val future = ColorthiefAsync.getColor(pixels, width, height, quality)
    cont.invokeOnCancellation { future.cancel(true) }
    future.whenComplete { result, ex ->
        if (ex != null) cont.resumeWithException(ex)
        else cont.resume(result)
    }
}
