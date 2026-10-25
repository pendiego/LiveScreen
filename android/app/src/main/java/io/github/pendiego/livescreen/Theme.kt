package io.github.pendiego.livescreen

// Material 3 do Android: cor dinâmica no Android 12+, azul fixo antes disso. O vídeo força o escuro.

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext

/** Cores de estado que o Material 3 não tem: conexão direta (success) e via servidor ou aviso (warn). */
data class StatusColors(val success: Color, val warn: Color)

private val DarkStatus = StatusColors(success = Color(0xFF6CCB5F), warn = Color(0xFFF2C744))
private val LightStatus = StatusColors(success = Color(0xFF0F7B0F), warn = Color(0xFF9D5D00))
val LocalStatusColors = staticCompositionLocalOf { DarkStatus }

private val FallbackDark = darkColorScheme(
    primary = Color(0xFFA8C7FA), onPrimary = Color(0xFF062E6F),
    primaryContainer = Color(0xFF0842A0), onPrimaryContainer = Color(0xFFD3E3FD),
)
private val FallbackLight = lightColorScheme(
    primary = Color(0xFF0B57D0), onPrimary = Color(0xFFFFFFFF),
    primaryContainer = Color(0xFFD3E3FD), onPrimaryContainer = Color(0xFF041E49),
)

@Composable
private fun colorScheme(dark: Boolean): ColorScheme {
    val context = LocalContext.current
    return when {
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.S ->
            if (dark) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        dark -> FallbackDark
        else -> FallbackLight
    }
}

@Composable
fun LiveScreenTheme(dark: Boolean = isSystemInDarkTheme(), content: @Composable () -> Unit) {
    CompositionLocalProvider(LocalStatusColors provides if (dark) DarkStatus else LightStatus) {
        MaterialTheme(colorScheme = colorScheme(dark), content = content)
    }
}
