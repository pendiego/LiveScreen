package io.github.pendiego.livescreen

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Fill
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONObject

enum class Ink { Accent, Dark, Light, Back }

sealed interface AvatarShape {
    data class Circle(val ink: Ink, val cx: Float, val cy: Float, val r: Float) : AvatarShape
    data class Ellipse(val ink: Ink, val cx: Float, val cy: Float, val rx: Float, val ry: Float) : AvatarShape
    data class Rect(val ink: Ink, val x: Float, val y: Float, val w: Float, val h: Float, val rx: Float) : AvatarShape
    data class Poly(val ink: Ink, val pts: FloatArray) : AvatarShape
    data class Line(val ink: Ink, val w: Float, val x1: Float, val y1: Float, val x2: Float, val y2: Float) : AvatarShape
    data class Pline(val ink: Ink, val w: Float, val pts: FloatArray) : AvatarShape
    data class Ring(val ink: Ink, val w: Float, val cx: Float, val cy: Float, val rx: Float, val ry: Float) : AvatarShape
}

data class AvatarDef(
    val id: Int,
    val name: String,
    val accent: Color,
    val shapes: List<AvatarShape>
)

object AvatarData {
    const val COUNT = 7
    val BACK = Color(0xFF1C2027)
    val DARK = Color(0xFF0B0D10)
    val LIGHT = Color(0xFFF3F5F7)

    fun auto(key: String): Int {
        var h = 0x811C9DC5u
        for (b in key.toByteArray(Charsets.UTF_8)) {
            h = (h xor (b.toUInt() and 0xFFu)) * 0x01000193u
        }
        return ((h % 7u) + 1u).toInt()
    }

    fun resolve(chosen: Int, key: String): Int =
        if (chosen in 1..COUNT) chosen else auto(key)

    private fun parseHexColor(hex: String): Color {
        val clean = hex.removePrefix("#")
        val v = clean.toLong(16)
        return Color(if (clean.length == 6) 0xFF000000L or v else v)
    }

    // fonte única: os desenhos vêm do núcleo (livescreen_core::avatar) pela ponte JNI
    private val cache: Map<Int, AvatarDef> by lazy { parseJson(Native.avatarsJson()) }

    private fun parseJson(raw: String): Map<Int, AvatarDef> = try {
        val root = JSONObject(raw)
        val arr = root.getJSONArray("avatars")
        val map = mutableMapOf<Int, AvatarDef>()
        for (i in 0 until arr.length()) {
            val obj = arr.getJSONObject(i)
            val id = obj.getInt("id")
            val name = obj.getString("name")
            val accent = parseHexColor(obj.getString("accent"))
            val shapesArr = obj.getJSONArray("shapes")
            val shapes = mutableListOf<AvatarShape>()
            for (j in 0 until shapesArr.length()) {
                val s = shapesArr.getJSONArray(j)
                val type = s.getString(0)
                val ink = when (s.getString(1)) {
                    "a" -> Ink.Accent
                    "d" -> Ink.Dark
                    "l" -> Ink.Light
                    "b" -> Ink.Back
                    else -> Ink.Light
                }
                when (type) {
                    "circle" -> shapes.add(
                        AvatarShape.Circle(ink, s.getDouble(2).toFloat(), s.getDouble(3).toFloat(), s.getDouble(4).toFloat())
                    )
                    "ellipse" -> shapes.add(
                        AvatarShape.Ellipse(ink, s.getDouble(2).toFloat(), s.getDouble(3).toFloat(), s.getDouble(4).toFloat(), s.getDouble(5).toFloat())
                    )
                    "rect" -> shapes.add(
                        AvatarShape.Rect(ink, s.getDouble(2).toFloat(), s.getDouble(3).toFloat(), s.getDouble(4).toFloat(), s.getDouble(5).toFloat(), s.getDouble(6).toFloat())
                    )
                    "poly" -> {
                        val pts = FloatArray(s.length() - 2) { idx -> s.getDouble(idx + 2).toFloat() }
                        shapes.add(AvatarShape.Poly(ink, pts))
                    }
                    "line" -> shapes.add(
                        AvatarShape.Line(ink, s.getDouble(2).toFloat(), s.getDouble(3).toFloat(), s.getDouble(4).toFloat(), s.getDouble(5).toFloat(), s.getDouble(6).toFloat())
                    )
                    "pline" -> {
                        val w = s.getDouble(2).toFloat()
                        val pts = FloatArray(s.length() - 3) { idx -> s.getDouble(idx + 3).toFloat() }
                        shapes.add(AvatarShape.Pline(ink, w, pts))
                    }
                    "ring" -> shapes.add(
                        AvatarShape.Ring(ink, s.getDouble(2).toFloat(), s.getDouble(3).toFloat(), s.getDouble(4).toFloat(), s.getDouble(5).toFloat(), s.getDouble(6).toFloat())
                    )
                }
            }
            map[id] = AvatarDef(id, name, accent, shapes)
        }
        map
    } catch (_: Exception) {
        emptyMap()
    }

    fun get(id: Int): AvatarDef? = cache[id]

    fun all(): List<AvatarDef> = (1..COUNT).mapNotNull { cache[it] }
}

@Composable
fun Avatar(
    id: Int,
    size: Dp,
    modifier: Modifier = Modifier
) {
    val def = AvatarData.get(id)
    Canvas(modifier = modifier.size(size)) {
        val scale = this.size.width / 64f
        val backColor = AvatarData.BACK
        drawCircle(color = backColor, radius = this.size.minDimension / 2f, center = center)

        if (def == null) return@Canvas

        val darkColor = AvatarData.DARK
        val lightColor = AvatarData.LIGHT
        val accentColor = def.accent

        fun inkColor(ink: Ink): Color = when (ink) {
            Ink.Accent -> accentColor
            Ink.Dark -> darkColor
            Ink.Light -> lightColor
            Ink.Back -> backColor
        }

        for (shape in def.shapes) {
            when (shape) {
                is AvatarShape.Circle -> {
                    drawCircle(
                        color = inkColor(shape.ink),
                        radius = shape.r * scale,
                        center = Offset(shape.cx * scale, shape.cy * scale)
                    )
                }
                is AvatarShape.Ellipse -> {
                    drawOval(
                        color = inkColor(shape.ink),
                        topLeft = Offset((shape.cx - shape.rx) * scale, (shape.cy - shape.ry) * scale),
                        size = Size(shape.rx * 2f * scale, shape.ry * 2f * scale)
                    )
                }
                is AvatarShape.Rect -> {
                    drawRoundRect(
                        color = inkColor(shape.ink),
                        topLeft = Offset(shape.x * scale, shape.y * scale),
                        size = Size(shape.w * scale, shape.h * scale),
                        cornerRadius = CornerRadius(shape.rx * scale, shape.rx * scale)
                    )
                }
                is AvatarShape.Poly -> {
                    if (shape.pts.size >= 4) {
                        val path = Path().apply {
                            moveTo(shape.pts[0] * scale, shape.pts[1] * scale)
                            for (p in 2 until shape.pts.size step 2) {
                                lineTo(shape.pts[p] * scale, shape.pts[p + 1] * scale)
                            }
                            close()
                        }
                        val c = inkColor(shape.ink)
                        drawPath(path, color = c, style = Fill)
                        drawPath(
                            path,
                            color = c,
                            style = Stroke(width = 1f * scale, cap = StrokeCap.Round, join = StrokeJoin.Round)
                        )
                    }
                }
                is AvatarShape.Line -> {
                    drawLine(
                        color = inkColor(shape.ink),
                        start = Offset(shape.x1 * scale, shape.y1 * scale),
                        end = Offset(shape.x2 * scale, shape.y2 * scale),
                        strokeWidth = shape.w * scale,
                        cap = StrokeCap.Round
                    )
                }
                is AvatarShape.Pline -> {
                    if (shape.pts.size >= 4) {
                        val path = Path().apply {
                            moveTo(shape.pts[0] * scale, shape.pts[1] * scale)
                            for (p in 2 until shape.pts.size step 2) {
                                lineTo(shape.pts[p] * scale, shape.pts[p + 1] * scale)
                            }
                        }
                        drawPath(
                            path,
                            color = inkColor(shape.ink),
                            style = Stroke(
                                width = shape.w * scale,
                                cap = StrokeCap.Round,
                                join = StrokeJoin.Round
                            )
                        )
                    }
                }
                is AvatarShape.Ring -> {
                    drawOval(
                        color = inkColor(shape.ink),
                        topLeft = Offset((shape.cx - shape.rx) * scale, (shape.cy - shape.ry) * scale),
                        size = Size(shape.rx * 2f * scale, shape.ry * 2f * scale),
                        style = Stroke(width = shape.w * scale)
                    )
                }
            }
        }
    }
}

@Composable
fun AvatarPicker(
    selected: Int,
    currentName: String,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Text(
            text = "Avatar",
            style = MaterialTheme.typography.titleSmall,
            color = Sinal.textMuted
        )

        // Grade 4x2: Automático + 7 avatares
        val row1 = listOf(0, 1, 2, 3)
        val row2 = listOf(4, 5, 6, 7)

        for (row in listOf(row1, row2)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                for (id in row) {
                    val isSelected = selected == id
                    val def = if (id > 0) AvatarData.get(id) else null
                    val label = if (id == 0) "Auto" else (def?.name ?: "")
                    val cd = if (id == 0) "Avatar automático" else "Avatar ${def?.name ?: id}"

                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        modifier = Modifier
                            .clip(RoundedCornerShape(10.dp))
                            .clickable { onSelect(id) }
                            .semantics { contentDescription = cd }
                            .padding(horizontal = 4.dp, vertical = 6.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(48.dp)
                                .background(
                                    if (isSelected) Sinal.accent.copy(alpha = 0.12f) else Sinal.surface,
                                    CircleShape
                                )
                                .border(
                                    width = if (isSelected) 2.dp else 1.dp,
                                    color = if (isSelected) Sinal.accent else Sinal.border,
                                    shape = CircleShape
                                )
                                .padding(if (isSelected) 3.dp else 4.dp),
                            contentAlignment = Alignment.Center
                        ) {
                            val displayId = if (id == 0) AvatarData.resolve(0, currentName.trim()) else id
                            Avatar(id = displayId, size = 36.dp)
                        }
                        Spacer(Modifier.height(4.dp))
                        Text(
                            text = label,
                            style = MaterialTheme.typography.bodySmall,
                            fontSize = 12.sp,
                            color = if (isSelected) Sinal.accent else Sinal.textMuted,
                            maxLines = 1
                        )
                    }
                }
            }
        }
    }
}
