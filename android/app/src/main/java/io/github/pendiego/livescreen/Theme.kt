package io.github.pendiego.livescreen

// Tokens espelhados de `src/ui/theme.rs` (desktop); mudou lá, muda aqui.

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Shapes
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextFieldColors
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

data class Palette(
    val bg: Color,
    val extreme: Color,
    val text: Color,
    val weak: Color,
    val hover: Color,
    val pressed: Color,
    val card: Color,
    val border: Color,
    val borderStrong: Color,
    val accent: Color,
    val primary: Color,
    val onPrimary: Color,
    val danger: Color,
    val onDanger: Color,
    val success: Color,
    val warn: Color,
)

val DarkPalette = Palette(
    bg = Color(0xFF0A0A0A),
    extreme = Color(0xFF1A1A1A),
    text = Color(0xFFEDEDED),
    weak = Color(0xFFA1A1A1),
    hover = Color(0xFF1F1F1F),
    pressed = Color(0xFF2E2E2E),
    card = Color(0xFF0A0A0A),
    border = Color(0x25FFFFFF),
    borderStrong = Color(0x3DFFFFFF),
    accent = Color(0xFF0072F5),
    primary = Color(0xFFEDEDED),
    onPrimary = Color(0xFF0A0A0A),
    danger = Color(0xFFFF5B4F),
    onDanger = Color(0xFFFFFFFF),
    success = Color(0xFF6CCB5F),
    warn = Color(0xFFF2C744),
)

val LightPalette = Palette(
    bg = Color(0xFFFFFFFF),
    extreme = Color(0xFFFAFAFA),
    text = Color(0xFF171717),
    weak = Color(0xFF666666),
    hover = Color(0xFFF2F2F2),
    pressed = Color(0xFFEBEBEB),
    card = Color(0xFFFFFFFF),
    border = Color(0x14000000),
    borderStrong = Color(0x24000000),
    accent = Color(0xFF0072F5),
    primary = Color(0xFF171717),
    onPrimary = Color(0xFFFFFFFF),
    danger = Color(0xFFFF5B4F),
    onDanger = Color(0xFFFFFFFF),
    success = Color(0xFF0F7B0F),
    warn = Color(0xFF9D5D00),
)

val LocalPalette = staticCompositionLocalOf { DarkPalette }

val Geist = FontFamily(
    Font(R.font.geist_regular, FontWeight.Normal),
    Font(R.font.geist_medium, FontWeight.Medium),
    Font(R.font.geist_semibold, FontWeight.SemiBold),
)
val GeistMono = FontFamily(Font(R.font.geistmono_regular, FontWeight.Normal))

private val Typography = Typography(
    headlineLarge = TextStyle(fontFamily = Geist, fontSize = 28.sp, fontWeight = FontWeight.SemiBold, letterSpacing = (-1.1).sp),
    headlineSmall = TextStyle(fontFamily = Geist, fontSize = 22.sp, fontWeight = FontWeight.SemiBold, letterSpacing = (-0.5).sp),
    bodyLarge = TextStyle(fontFamily = Geist, fontSize = 14.sp, fontWeight = FontWeight.Normal),
    bodyMedium = TextStyle(fontFamily = Geist, fontSize = 14.sp, fontWeight = FontWeight.Normal),
    bodySmall = TextStyle(fontFamily = Geist, fontSize = 12.sp, fontWeight = FontWeight.Normal),
    labelMedium = TextStyle(fontFamily = Geist, fontSize = 12.sp, fontWeight = FontWeight.Normal),
    labelSmall = TextStyle(fontFamily = Geist, fontSize = 12.sp, fontWeight = FontWeight.Normal),
    labelLarge = TextStyle(fontFamily = Geist, fontSize = 14.sp, fontWeight = FontWeight.Medium),
)

val MonoStyle = TextStyle(fontFamily = GeistMono, fontSize = 13.sp)

private val Shapes = Shapes(
    extraSmall = RoundedCornerShape(6.dp),
    small = RoundedCornerShape(6.dp),
    medium = RoundedCornerShape(8.dp),
    large = RoundedCornerShape(12.dp),
    extraLarge = RoundedCornerShape(12.dp),
)

internal fun Palette.scheme(dark: Boolean): ColorScheme =
    if (dark) {
        darkColorScheme(
            primary = primary, onPrimary = onPrimary,
            background = bg, onBackground = text,
            surface = bg, onSurface = text,
            surfaceVariant = extreme, onSurfaceVariant = weak,
            surfaceContainer = card, surfaceContainerLow = card, surfaceContainerHigh = card,
            outline = borderStrong, outlineVariant = border,
            error = danger, onError = onDanger,
        )
    } else {
        lightColorScheme(
            primary = primary, onPrimary = onPrimary,
            background = bg, onBackground = text,
            surface = bg, onSurface = text,
            surfaceVariant = extreme, onSurfaceVariant = weak,
            surfaceContainer = card, surfaceContainerLow = card, surfaceContainerHigh = card,
            outline = borderStrong, outlineVariant = border,
            error = danger, onError = onDanger,
        )
    }

@Composable
fun LiveScreenTheme(dark: Boolean = isSystemInDarkTheme(), content: @Composable () -> Unit) {
    val p = if (dark) DarkPalette else LightPalette
    CompositionLocalProvider(LocalPalette provides p) {
        MaterialTheme(colorScheme = p.scheme(dark), typography = Typography, shapes = Shapes, content = content)
    }
}

private val ButtonShape = RoundedCornerShape(6.dp)
private val ButtonPadding = PaddingValues(horizontal = 12.dp, vertical = 6.dp)

@Composable
fun PrimaryButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    val p = LocalPalette.current
    Button(
        onClick = onClick,
        modifier = modifier.height(36.dp),
        enabled = enabled,
        shape = ButtonShape,
        contentPadding = ButtonPadding,
        colors = ButtonDefaults.buttonColors(
            containerColor = p.primary,
            contentColor = p.onPrimary,
            disabledContainerColor = p.primary.copy(alpha = 0.4f),
            disabledContentColor = p.onPrimary,
        ),
    ) { Text(text, maxLines = 1) }
}

@Composable
fun SecondaryButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    val p = LocalPalette.current
    OutlinedButton(
        onClick = onClick,
        modifier = modifier.height(36.dp),
        enabled = enabled,
        shape = ButtonShape,
        contentPadding = ButtonPadding,
        border = BorderStroke(1.dp, p.borderStrong),
        colors = ButtonDefaults.outlinedButtonColors(
            containerColor = p.card,
            contentColor = p.text,
            disabledContentColor = p.text.copy(alpha = 0.4f),
        ),
    ) { Text(text, maxLines = 1) }
}

@Composable
fun StatusBadge(text: String, color: Color) {
    Text(
        text,
        style = MaterialTheme.typography.bodySmall,
        color = color,
        modifier = Modifier
            .background(color.copy(alpha = 0.18f), RoundedCornerShape(4.dp))
            .padding(horizontal = 6.dp, vertical = 2.dp),
    )
}

@Composable
fun SurfaceCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    val p = LocalPalette.current
    Surface(
        modifier,
        shape = RoundedCornerShape(12.dp),
        color = p.card,
        border = BorderStroke(1.dp, p.border),
        shadowElevation = 1.dp,
    ) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp), content = content)
    }
}

@Composable
fun fieldColors(): TextFieldColors {
    val p = LocalPalette.current
    return OutlinedTextFieldDefaults.colors(
        focusedBorderColor = p.accent,
        unfocusedBorderColor = p.borderStrong,
        focusedLabelColor = p.accent,
        unfocusedLabelColor = p.weak,
        cursorColor = p.accent,
        focusedContainerColor = p.card,
        unfocusedContainerColor = p.card,
    )
}
