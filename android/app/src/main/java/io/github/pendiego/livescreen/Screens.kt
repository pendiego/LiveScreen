package io.github.pendiego.livescreen

import android.app.Activity
import android.content.ClipboardManager
import android.content.Context
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import kotlinx.coroutines.delay

@Composable
fun App(vm: ViewerModel) {
    if (vm.status == null) {
        HomeScreen(vm)
    } else {
        WatchScreen(vm)
    }
}

@Composable
fun HomeScreen(vm: ViewerModel) {
    val context = LocalContext.current
    val trimmedTicket = vm.ticket.trim()
    val isTicketValid = remember(trimmedTicket) { Native.isValidTicket(trimmedTicket) }
    val isTicketNotEmpty = trimmedTicket.isNotEmpty()

    val p = LocalPalette.current
    Box(
        modifier = Modifier
            .fillMaxSize()
            .safeDrawingPadding()
            .padding(24.dp),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(8.dp),
            modifier = Modifier.widthIn(max = 560.dp).verticalScroll(rememberScrollState())
        ) {
            Text(
                text = "LiveScreen",
                style = MaterialTheme.typography.headlineLarge
            )
            Text(
                text = "Assista a uma tela compartilhada pelo LiveScreen do Windows.",
                style = MaterialTheme.typography.bodyMedium,
                color = p.weak
            )
            Spacer(Modifier.height(16.dp))
            UpdateBanner(vm)
            SurfaceCard(Modifier.fillMaxWidth()) {
                OutlinedTextField(
                    value = vm.name,
                    onValueChange = { vm.name = it },
                    label = { Text("Seu nome (opcional)") },
                    singleLine = true,
                    colors = fieldColors(),
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = vm.ticket,
                    onValueChange = { vm.ticket = it },
                    label = { Text("Código de convite") },
                    singleLine = true,
                    textStyle = MonoStyle,
                    colors = fieldColors(),
                    modifier = Modifier.fillMaxWidth()
                )
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End)
                ) {
                    SecondaryButton(
                        "Colar",
                        onClick = {
                            val cm = context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
                            val clip = cm?.primaryClip
                            if (clip != null && clip.itemCount > 0) {
                                val text = clip.getItemAt(0)?.coerceToText(context)?.toString() ?: ""
                                vm.ticket = text.trim()
                            }
                        }
                    )
                    PrimaryButton("Assistir", onClick = { vm.connect() }, enabled = isTicketValid)
                }
                if (isTicketNotEmpty && !isTicketValid) {
                    Text(
                        text = "Código de convite inválido",
                        color = p.danger,
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
                if (vm.lastEnd != null) {
                    Text(
                        text = vm.lastEnd!!,
                        color = p.danger,
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
            }
        }
    }
}

@Composable
fun WatchScreen(vm: ViewerModel) {
    // Vídeo sempre sobre preto: o overlay usa a paleta escura mesmo com o sistema claro.
    CompositionLocalProvider(LocalPalette provides DarkPalette) {
        MaterialTheme(
            colorScheme = DarkPalette.scheme(true),
            typography = MaterialTheme.typography,
            shapes = MaterialTheme.shapes
        ) {
            WatchContent(vm)
        }
    }
}

// Mesmo `OVERLAY_TIMEOUT` do desktop (watch_view.rs).
private const val OVERLAY_TIMEOUT_MS = 2500L

@Composable
private fun WatchContent(vm: ViewerModel) {
    val view = LocalView.current
    DisposableEffect(Unit) {
        val window = (view.context as? Activity)?.window
        if (window != null) {
            val controller = WindowCompat.getInsetsController(window, view)
            controller.systemBarsBehavior =
                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            controller.hide(WindowInsetsCompat.Type.systemBars())
        }
        onDispose {
            if (window != null) {
                val controller = WindowCompat.getInsetsController(window, view)
                controller.show(WindowInsetsCompat.Type.systemBars())
            }
        }
    }

    BackHandler {
        vm.leave()
    }

    val status = vm.status
    val videoSize = vm.videoSize
    val (w, h) = when {
        videoSize != null && videoSize.first > 0 && videoSize.second > 0 -> videoSize
        status != null && status.width > 0 && status.height > 0 -> Pair(status.width, status.height)
        else -> Pair(16, 9)
    }
    val aspectRatio = w.toFloat() / h.toFloat()

    var showOverlay by remember { mutableStateOf(true) }
    var qualityExpanded by remember { mutableStateOf(false) }
    var lastTouchTime by remember { mutableLongStateOf(System.currentTimeMillis()) }

    LaunchedEffect(showOverlay, qualityExpanded, lastTouchTime) {
        if (showOverlay && !qualityExpanded) {
            delay(OVERLAY_TIMEOUT_MS)
            showOverlay = false
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .clickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null
            ) {
                showOverlay = !showOverlay
                if (showOverlay) {
                    lastTouchTime = System.currentTimeMillis()
                }
            }
    ) {
        AndroidView(
            factory = { ctx ->
                SurfaceView(ctx).apply {
                    keepScreenOn = true
                    holder.addCallback(object : SurfaceHolder.Callback {
                        override fun surfaceCreated(holder: SurfaceHolder) {
                            vm.attachSurface(holder.surface)
                        }

                        override fun surfaceChanged(
                            holder: SurfaceHolder,
                            format: Int,
                            width: Int,
                            height: Int
                        ) {}

                        override fun surfaceDestroyed(holder: SurfaceHolder) {
                            vm.detachSurface()
                        }
                    })
                }
            },
            modifier = Modifier
                .align(Alignment.Center)
                .aspectRatio(aspectRatio)
        )

        if (status?.connecting == true) {
            Column(
                modifier = Modifier.align(Alignment.Center),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                CircularProgressIndicator(Modifier.size(24.dp), color = Color(0xFFC8C8C8), strokeWidth = 2.dp)
                Text("Conectando…", style = MaterialTheme.typography.bodyMedium, color = Color(0xFFC8C8C8))
            }
        }

        if (showOverlay) {
            val p = LocalPalette.current
            @OptIn(ExperimentalLayoutApi::class)
            FlowRow(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(start = 16.dp, end = 16.dp, bottom = 24.dp)
                    .widthIn(max = 720.dp)
                    .fillMaxWidth()
                    .background(Color.Black.copy(alpha = 190f / 255f), RoundedCornerShape(8.dp))
                    .padding(horizontal = 12.dp, vertical = 8.dp),
                itemVerticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                SecondaryButton("Sair", onClick = { vm.leave() })

                val badgeText = when {
                    status?.connecting == true -> "Conectando…"
                    status?.path == "Relay" -> "Via servidor"
                    else -> "Conexão direta"
                }
                val badgeColor = when {
                    status?.connecting == true -> p.weak
                    status?.path == "Relay" -> p.warn
                    else -> p.success
                }
                StatusBadge(badgeText, badgeColor)

                val rttStr = status?.rttMs?.let { "$it ms" } ?: "- ms"
                val fpsStr = "${status?.fps ?: 0} fps"
                val dimStr = "${w}×${h}"
                val titleStr = status?.title?.takeIf { it.isNotEmpty() }?.let { " · $it" } ?: ""
                val infoText = "$rttStr · $fpsStr · $dimStr$titleStr"

                Text(
                    text = infoText,
                    style = MaterialTheme.typography.bodySmall,
                    color = p.weak,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.align(Alignment.CenterVertically)
                )

                if (status?.audio == true) {
                    SecondaryButton(
                        if (vm.muted) "Ativar som" else "Silenciar",
                        onClick = { vm.muted = !vm.muted }
                    )
                }

                // qualityExpanded lifted to WatchScreen scope
                Box {
                    SecondaryButton("Qualidade", onClick = { qualityExpanded = true })
                    DropdownMenu(
                        expanded = qualityExpanded,
                        onDismissRequest = { qualityExpanded = false },
                        shape = RoundedCornerShape(8.dp),
                        containerColor = p.card,
                        border = BorderStroke(1.dp, p.border),
                        shadowElevation = 4.dp
                    ) {
                        DropdownMenuItem(
                            text = {
                                Text(
                                    text = "Só vale se for mais leve que a atual",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = p.weak
                                )
                            },
                            onClick = {},
                            enabled = false
                        )
                        for ((id, name) in vm.presets) {
                            DropdownMenuItem(
                                text = { Text(name, style = MaterialTheme.typography.bodyMedium, color = p.text) },
                                onClick = {
                                    vm.requestQuality(id)
                                    qualityExpanded = false
                                }
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun UpdateBanner(vm: ViewerModel) {
    val p = LocalPalette.current
    val s = vm.update
    val info = when (s) {
        is UpdateState.Available -> s.info
        is UpdateState.Downloading -> s.info
        is UpdateState.Ready -> s.info
        is UpdateState.Failed -> s.info
        UpdateState.Idle -> return
    }
    val label = when (s) {
        is UpdateState.Downloading -> s.progress?.let { "Baixando… ${(it * 100).toInt()}%" } ?: "Baixando…"
        is UpdateState.Ready -> "Instalar"
        is UpdateState.Failed -> "Tentar de novo"
        else -> "Atualizar"
    }
    SurfaceCard(Modifier.fillMaxWidth()) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Text(
                "Versão ${info.version} disponível",
                style = MaterialTheme.typography.bodyMedium,
                color = p.text,
                modifier = Modifier.weight(1f)
            )
            PrimaryButton(label, onClick = { vm.startUpdate() }, enabled = s !is UpdateState.Downloading)
        }
        if (s is UpdateState.Failed) {
            Text(s.message, style = MaterialTheme.typography.bodyMedium, color = p.danger)
        }
    }
    Spacer(Modifier.height(12.dp))
}
