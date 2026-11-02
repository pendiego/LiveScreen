package io.github.pendiego.livescreen

// Tema Sinal (escuro único), espelho de src/ui/theme.rs. Fontes copiadas de assets/fonts pelo Gradle (copyFonts).

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

object Sinal {
    val bg = Color(0xFF0B0D10); val surface = Color(0xFF14171C); val raised = Color(0xFF1C2027)
    val border = Color(0xFF262B33); val borderStrong = Color(0xFF39404B)
    val text = Color(0xFFF3F5F7); val textMuted = Color(0xFF9AA3AE); val textFaint = Color(0xFF5F6773)
    val accent = Color(0xFFC8F135); val onAccent = Color(0xFF0B0D10)
    val success = Color(0xFF3DDC97); val warn = Color(0xFFFFB547)
    val danger = Color(0xFFFF5A4E); val onDanger = Color(0xFF1A0705)
}
private val SpaceGrotesk = FontFamily(Font(R.font.spacegrotesk_medium, FontWeight.Medium))
private val Inter = FontFamily(
    Font(R.font.inter_regular, FontWeight.Normal),
    Font(R.font.inter_medium, FontWeight.Medium),
    Font(R.font.inter_semibold, FontWeight.SemiBold),
)
/** Códigos de convite e de amigo (JetBrains Mono 14/22). */
val MonoStyle = TextStyle(fontFamily = FontFamily(Font(R.font.jetbrainsmono_regular)), fontSize = 14.sp, lineHeight = 22.sp)
private fun inter(w: FontWeight, size: Int, lh: Int) = TextStyle(fontFamily = Inter, fontWeight = w, fontSize = size.sp, lineHeight = lh.sp)
private val SinalTypography = Typography(
    titleLarge = TextStyle(fontFamily = SpaceGrotesk, fontWeight = FontWeight.Medium, fontSize = 20.sp, lineHeight = 28.sp), // títulos
    titleMedium = inter(FontWeight.SemiBold, 16, 24), // botões de 48–56 dp
    titleSmall = inter(FontWeight.Medium, 14, 20),    // rótulos, nome do amigo, título do player
    bodyLarge = inter(FontWeight.Normal, 16, 24),     // texto corrido
    bodyMedium = inter(FontWeight.Normal, 14, 20),
    bodySmall = inter(FontWeight.Normal, 12, 16),     // estado, ajuda, estatísticas
    labelLarge = inter(FontWeight.SemiBold, 14, 20),  // botões de 32–40 dp
    labelMedium = inter(FontWeight.Medium, 12, 16),   // badges
)
private val SinalColors = darkColorScheme(
    primary = Sinal.accent, onPrimary = Sinal.onAccent,
    background = Sinal.bg, onBackground = Sinal.text,
    surface = Sinal.bg, onSurface = Sinal.text,
    surfaceVariant = Sinal.raised, onSurfaceVariant = Sinal.textMuted,
    surfaceContainerLow = Sinal.surface, surfaceContainer = Sinal.raised,
    surfaceContainerHigh = Sinal.raised, surfaceContainerHighest = Sinal.raised,
    outline = Sinal.border, outlineVariant = Sinal.border,
    error = Sinal.danger, onError = Sinal.onDanger,
)
@Composable
fun LiveScreenTheme(content: @Composable () -> Unit) =
    MaterialTheme(colorScheme = SinalColors, typography = SinalTypography, content = content)
