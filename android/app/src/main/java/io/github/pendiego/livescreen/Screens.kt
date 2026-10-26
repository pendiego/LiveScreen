package io.github.pendiego.livescreen

import android.app.Activity
import android.content.ClipData
import android.widget.Toast
import android.content.ClipboardManager
import android.content.Context
import android.content.pm.ActivityInfo
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import androidx.lifecycle.compose.LifecycleStartEffect

private const val LEGACY_MSG = "Código sem chave de sala: só conecta em quem transmite com o LiveScreen 0.4. Se veio de uma versão mais nova, falta o final do código (a parte depois do ponto)."

@Composable
fun App(vm: ViewerModel) {
    if (vm.status == null) {
        HomeScreen(vm)
    } else {
        WatchScreen(vm)
    }
}

private fun clipboardText(context: Context): String? {
    val cm = context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager ?: return null
    val clip = cm.primaryClip ?: return null
    if (clip.itemCount == 0) return null
    return clip.getItemAt(0)?.coerceToText(context)?.toString()
}

@Composable
fun HomeScreen(vm: ViewerModel) {
    val context = LocalContext.current
    LifecycleStartEffect(Unit) {
        vm.startPresence()
        onStopOrDispose {
            vm.stopPresence()
        }
    }

    var showAddDialog by remember { mutableStateOf(false) }
    var friendToRemove by remember { mutableStateOf<Friend?>(null) }
    val isTicketValid = remember(vm.ticket) { Native.isValidTicket(vm.ticket) }
    val isLegacy = remember(vm.ticket) { Native.isLegacyInvite(vm.ticket) }
    val isTicketNotEmpty = vm.ticket.isNotEmpty()
    val windowFocused = LocalWindowInfo.current.isWindowFocused
    LaunchedEffect(windowFocused) {
        if (windowFocused && !Native.isValidTicket(vm.ticket)) {
            clipboardText(context)?.let(Native::findInvite)?.let { vm.ticket = it }
        }
    }

    val snackbar = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val sc = LocalStatusColors.current
    val colors = MaterialTheme.colorScheme
    val type = MaterialTheme.typography
    val paste: () -> Unit = {
        val inv = clipboardText(context)?.let(Native::findInvite)
        if (inv == null) {
            scope.launch { snackbar.showSnackbar("O texto colado não tem um código de convite") }
        } else {
            vm.ticket = inv
            if (!Native.isLegacyInvite(inv) && !vm.starting) vm.connect()
        }
    }

    Scaffold(
        snackbarHost = { SnackbarHost(snackbar) },
        bottomBar = {
            Button(
                onClick = { vm.connect() },
                enabled = isTicketValid && !vm.starting,
                modifier = Modifier
                    .fillMaxWidth()
                    .windowInsetsPadding(WindowInsets.navigationBars.union(WindowInsets.ime))
                    .padding(16.dp)
                    .height(56.dp)
            ) { Text(if (vm.starting) "Conectando…" else "Assistir pelo convite", style = type.titleMedium) }
        }
    ) { inner ->
        Box(
            Modifier.fillMaxSize().padding(inner).consumeWindowInsets(inner),
            contentAlignment = Alignment.TopCenter
        ) {
            Column(
                Modifier
                    .widthIn(max = 560.dp)
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 24.dp, vertical = 32.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Column(
                    Modifier.fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Surface(shape = CircleShape, color = colors.primaryContainer, modifier = Modifier.size(72.dp)) {
                        Box(contentAlignment = Alignment.Center) {
                            Icon(
                                painterResource(R.drawable.ic_live_tv), null,
                                tint = colors.onPrimaryContainer, modifier = Modifier.size(36.dp)
                            )
                        }
                    }
                    Text("LiveScreen", style = type.headlineMedium)
                    Text(
                        "Assista a uma tela compartilhada pelo LiveScreen do Windows.",
                        style = type.bodyLarge, color = colors.onSurfaceVariant, textAlign = TextAlign.Center
                    )
                }
                Spacer(Modifier.height(8.dp))
                UpdateBanner(vm)
                if (vm.friends.isNotEmpty()) {
                    FriendsSection(
                        vm = vm,
                        onAddClick = { showAddDialog = true },
                        onRemoveFriend = { friendToRemove = it },
                        onCopied = { scope.launch { snackbar.showSnackbar("Código de amigo copiado") } }
                    )
                }
                OutlinedTextField(
                    value = vm.ticket,
                    onValueChange = { vm.ticket = Native.findInvite(it) ?: it.filterNot(Char::isWhitespace) },
                    label = { Text("Código de convite") },
                    placeholder = { Text("Cole o código que você recebeu") },
                    textStyle = LocalTextStyle.current.copy(fontFamily = FontFamily.Monospace),
                    maxLines = 4,
                    isError = isTicketNotEmpty && !isTicketValid,
                    supportingText = when {
                        isTicketNotEmpty && !isTicketValid -> { { Text("Código inválido — confira se copiou o código inteiro.") } }
                        isTicketValid && isLegacy -> { { Text(LEGACY_MSG, color = sc.warn) } }
                        else -> null
                    },
                    trailingIcon = {
                        IconButton(onClick = paste) {
                            Icon(painterResource(R.drawable.ic_content_paste), contentDescription = "Colar")
                        }
                    },
                    keyboardOptions = KeyboardOptions(
                        autoCorrectEnabled = false, keyboardType = KeyboardType.Ascii, imeAction = ImeAction.Go
                    ),
                    keyboardActions = KeyboardActions(onGo = { if (isTicketValid && !vm.starting) vm.connect() }),
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = vm.name,
                    onValueChange = { vm.name = it },
                    label = { Text("Seu nome (opcional)") },
                    leadingIcon = { Icon(painterResource(R.drawable.ic_person), null) },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                    modifier = Modifier.fillMaxWidth()
                )
                vm.lastEnd?.let { end ->
                    Card(
                        colors = CardDefaults.cardColors(
                            containerColor = colors.errorContainer, contentColor = colors.onErrorContainer
                        ),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Row(
                            Modifier.padding(16.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            Icon(painterResource(R.drawable.ic_error), null)
                            Text(end, style = type.bodyMedium)
                        }
                    }
                }
                if (vm.friends.isEmpty()) {
                    FriendsSection(
                        vm = vm,
                        onAddClick = { showAddDialog = true },
                        onRemoveFriend = { friendToRemove = it },
                        onCopied = { scope.launch { snackbar.showSnackbar("Código de amigo copiado") } }
                    )
                }
            }
        }
    }

    if (showAddDialog) {
        AddFriendDialog(
            vm = vm,
            initialName = "",
            initialCode = "",
            onDismiss = { showAddDialog = false },
            onAdded = { addedName ->
                showAddDialog = false
                scope.launch {
                    snackbar.showSnackbar("$addedName adicionado aos amigos. Peça para ele adicionar você também.")
                }
            }
        )
    }

    friendToRemove?.let { friend ->
        AlertDialog(
            onDismissRequest = { friendToRemove = null },
            title = { Text("Remover ${friend.name}?") },
            text = { Text("Você deixará de ver quando esse amigo estiver transmitindo.") },
            confirmButton = {
                Button(onClick = {
                    vm.removeFriend(friend.code)
                    friendToRemove = null
                }) {
                    Text("Remover")
                }
            },
            dismissButton = {
                TextButton(onClick = { friendToRemove = null }) {
                    Text("Cancelar")
                }
            }
        )
    }
}

@Composable
fun WatchScreen(vm: ViewerModel) {
    // Vídeo sempre sobre preto: o overlay usa o esquema escuro mesmo com o sistema claro.
    LiveScreenTheme(dark = true) { WatchContent(vm) }
}

// Mesmo `OVERLAY_TIMEOUT` do desktop (watch_view.rs).
private const val OVERLAY_TIMEOUT_MS = 2500L

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun WatchContent(vm: ViewerModel) {
    val view = LocalView.current
    val context = LocalContext.current
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
    val activity = view.context as? Activity
    // Ao sair, a rotação volta a seguir o sensor.
    DisposableEffect(activity) {
        onDispose { activity?.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED }
    }

    BackHandler {
        vm.leave()
    }

    val status = vm.status
    val hostId = status?.hostId.orEmpty()
    val videoSize = vm.videoSize
    val (w, h) = when {
        videoSize != null && videoSize.first > 0 && videoSize.second > 0 -> videoSize
        status != null && status.width > 0 && status.height > 0 -> Pair(status.width, status.height)
        else -> Pair(16, 9)
    }
    val aspectRatio = w.toFloat() / h.toFloat()
    val type = MaterialTheme.typography
    val colors = MaterialTheme.colorScheme
    val sc = LocalStatusColors.current

    var showOverlay by remember { mutableStateOf(true) }
    var addHost by remember { mutableStateOf(false) }
    var qualitySheet by remember { mutableStateOf(false) }
    var showStats by remember { mutableStateOf(false) }
    var landscape by remember { mutableStateOf(false) }
    var lastTouchTime by remember { mutableLongStateOf(System.currentTimeMillis()) }
    val bump = { lastTouchTime = System.currentTimeMillis() }

    LaunchedEffect(showOverlay, qualitySheet, lastTouchTime) {
        if (showOverlay && !qualitySheet) {
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
                CircularProgressIndicator(Modifier.size(32.dp), color = Color.White, strokeWidth = 3.dp)
                Text("Conectando…", style = type.bodyLarge, color = Color.White.copy(alpha = 0.8f))
            }
        }

        AnimatedVisibility(
            visible = showOverlay,
            enter = fadeIn(),
            exit = fadeOut(),
            modifier = Modifier.fillMaxSize()
        ) {
            Box(Modifier.fillMaxSize()) {
                val badgeText = when {
                    status?.connecting == true -> "Conectando…"
                    status?.path == "Relay" -> "Via servidor"
                    else -> "Conexão direta"
                }
                val badgeColor = when {
                    status?.connecting == true -> Color.White.copy(alpha = 0.7f)
                    status?.path == "Relay" -> sc.warn
                    else -> sc.success
                }
                Row(
                    Modifier
                        .align(Alignment.TopCenter)
                        .fillMaxWidth()
                        .background(Brush.verticalGradient(listOf(Color.Black.copy(alpha = 0.7f), Color.Transparent)))
                        .windowInsetsPadding(WindowInsets.displayCutout)
                        .padding(start = 4.dp, end = 12.dp, top = 8.dp, bottom = 24.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    IconButton(onClick = { vm.leave() }) {
                        Icon(painterResource(R.drawable.ic_arrow_back), "Sair", tint = Color.White)
                    }
                    Text(
                        status?.title?.takeIf { it.isNotEmpty() } ?: "LiveScreen",
                        style = type.titleMedium, color = Color.White,
                        maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f)
                    )
                    Surface(shape = CircleShape, color = badgeColor.copy(alpha = 0.2f)) {
                        Text(
                            badgeText, style = type.labelMedium, color = badgeColor,
                            modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp)
                        )
                    }
                }

                val rttStr = status?.rttMs?.let { "$it ms" } ?: "- ms"
                val fpsStr = "${status?.fps ?: 0} fps"
                val infoText = "$rttStr · $fpsStr · ${w}×${h} · decodificação ${if (status?.decoderHw == true) "por hardware" else "por software"}"
                Column(
                    Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = 0.7f))))
                        .windowInsetsPadding(WindowInsets.displayCutout)
                        .padding(start = 4.dp, end = 4.dp, top = 24.dp, bottom = 8.dp)
                ) {
                    if (showStats) Text(
                        infoText, style = type.bodySmall, color = Color.White.copy(alpha = 0.8f),
                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 4.dp)
                    )
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        if (hostId.isNotEmpty() && status?.connecting == false && vm.friends.none { it.code == hostId }) {
                            IconButton(onClick = { addHost = true; bump() }) {
                                Icon(
                                    painterResource(R.drawable.ic_person_add),
                                    contentDescription = "Adicionar aos amigos",
                                    tint = Color.White
                                )
                            }
                        }
                        if (status?.audio == true) IconButton(onClick = { vm.muted = !vm.muted; bump() }) {
                            Icon(
                                painterResource(if (vm.muted) R.drawable.ic_volume_off else R.drawable.ic_volume_up),
                                if (vm.muted) "Ativar som" else "Silenciar", tint = Color.White
                            )
                        }
                        IconButton(onClick = { qualitySheet = true }) {
                            Icon(painterResource(R.drawable.ic_tune), "Qualidade", tint = Color.White)
                        }
                        Spacer(Modifier.weight(1f))
                        IconButton(onClick = { showStats = !showStats; bump() }) {
                            Icon(painterResource(R.drawable.ic_info), "Estatísticas", tint = Color.White)
                        }
                        IconButton(onClick = {
                            landscape = !landscape
                            activity?.requestedOrientation =
                                if (landscape) ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
                                else ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
                            bump()
                        }) {
                            Icon(painterResource(R.drawable.ic_screen_rotation), "Girar tela", tint = Color.White)
                        }
                    }
                }
            }
        }

        if (qualitySheet) {
            ModalBottomSheet(onDismissRequest = { qualitySheet = false; bump() }) {
                Text("Qualidade", style = type.titleLarge, modifier = Modifier.padding(horizontal = 24.dp))
                Text(
                    "Só vale se for mais leve que a atual", style = type.bodyMedium,
                    color = colors.onSurfaceVariant,
                    modifier = Modifier.padding(start = 24.dp, end = 24.dp, top = 4.dp, bottom = 8.dp)
                )
                for ((id, name) in vm.presets) {
                    ListItem(
                        headlineContent = { Text(name) },
                        colors = ListItemDefaults.colors(containerColor = Color.Transparent),
                        modifier = Modifier.clickable { vm.requestQuality(id); qualitySheet = false; bump() }
                    )
                }
                Spacer(Modifier.height(16.dp))
            }
        }

        if (addHost) {
            AddFriendDialog(
                vm = vm,
                initialName = status?.hostName.orEmpty(),
                initialCode = hostId,
                onDismiss = { addHost = false; bump() },
                onAdded = { addedName ->
                    addHost = false
                    Toast.makeText(
                        context,
                        "$addedName adicionado aos amigos. Peça para ele adicionar você também.",
                        Toast.LENGTH_LONG
                    ).show()
                    bump()
                }
            )
        }
    }
}

@Composable
private fun UpdateBanner(vm: ViewerModel) {
    val colors = MaterialTheme.colorScheme
    val type = MaterialTheme.typography
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
    Card(
        colors = CardDefaults.cardColors(
            containerColor = colors.secondaryContainer, contentColor = colors.onSecondaryContainer
        ),
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            Modifier.padding(start = 16.dp, end = 8.dp, top = 8.dp, bottom = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Icon(painterResource(R.drawable.ic_system_update), null)
            Text("Versão ${info.version} disponível", style = type.bodyMedium, modifier = Modifier.weight(1f))
            TextButton(onClick = { vm.startUpdate() }, enabled = s !is UpdateState.Downloading) { Text(label) }
        }
        if (s is UpdateState.Failed) {
            Text(
                s.message, style = type.bodySmall, color = colors.error,
                modifier = Modifier.padding(start = 16.dp, end = 16.dp, bottom = 12.dp)
            )
        }
    }
}

@Composable
private fun FriendsSection(
    vm: ViewerModel,
    onAddClick: () -> Unit,
    onRemoveFriend: (Friend) -> Unit,
    onCopied: () -> Unit,
) {
    val context = LocalContext.current
    val sc = LocalStatusColors.current
    val colors = MaterialTheme.colorScheme
    val type = MaterialTheme.typography

    Spacer(Modifier.height(8.dp))
    Row(
        Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text("Amigos", style = type.titleMedium)
        OutlinedButton(onClick = onAddClick) {
            Icon(painterResource(R.drawable.ic_person_add), null, modifier = Modifier.size(18.dp))
            Spacer(Modifier.size(8.dp))
            Text("Adicionar amigo")
        }
    }

    Card(
        colors = CardDefaults.cardColors(
            containerColor = colors.surfaceVariant.copy(alpha = 0.5f)
        ),
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(Modifier.weight(1f)) {
                Text("Seu código de amigo", style = type.labelMedium, color = colors.onSurfaceVariant)
                Text(
                    vm.myCode,
                    style = type.bodyMedium.copy(fontFamily = FontFamily.Monospace),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
            IconButton(onClick = {
                val cm = context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
                cm?.setPrimaryClip(ClipData.newPlainText("LiveScreen", vm.myCode))
                onCopied()
            }) {
                Icon(painterResource(R.drawable.ic_content_copy), contentDescription = "Copiar")
            }
        }
    }

    if (vm.friends.isEmpty()) {
        Text(
            "Adicione o código de um amigo que transmite pelo Windows. Ele também precisa adicionar o seu.",
            style = type.bodyMedium,
            color = colors.onSurfaceVariant
        )
    } else {
        Column(
            Modifier.fillMaxWidth(),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            for (friend in vm.friends) {
                val pres = vm.presence[friend.code]
                val state = pres?.state ?: "unknown"
                val isLive = state == "live"
                val subtitle = when (state) {
                    "live" -> if (pres?.title.isNullOrBlank()) "Transmitindo agora" else "Transmitindo: ${pres?.title}"
                    "offline" -> "Offline"
                    "not_friend" -> "Ainda não te adicionou"
                    "removed" -> "Você foi removido desta transmissão"
                    else -> "Verificando…"
                }
                val subtitleColor = when (state) {
                    "live" -> sc.success
                    "not_friend" -> sc.warn
                    "removed" -> colors.error
                    else -> colors.onSurfaceVariant
                }

                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = colors.surfaceVariant.copy(alpha = 0.35f)
                    ),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .padding(start = 16.dp, end = 8.dp, top = 8.dp, bottom = 8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(friend.name, style = type.titleMedium)
                            Text(subtitle, style = type.bodySmall, color = subtitleColor)
                        }
                        FilledTonalButton(
                            onClick = { vm.watchFriend(friend.code) },
                            enabled = isLive && !vm.starting,
                            modifier = Modifier.padding(end = 4.dp)
                        ) {
                            Text("Assistir")
                        }
                        IconButton(onClick = { onRemoveFriend(friend) }) {
                            Icon(
                                painterResource(R.drawable.ic_delete),
                                contentDescription = "Remover ${friend.name}",
                                tint = colors.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun AddFriendDialog(
    vm: ViewerModel,
    initialName: String,
    initialCode: String,
    onDismiss: () -> Unit,
    onAdded: (String) -> Unit,
) {
    val context = LocalContext.current
    val colors = MaterialTheme.colorScheme
    var name by remember { mutableStateOf(initialName) }
    var code by remember { mutableStateOf(initialCode) }
    var nameError by remember { mutableStateOf<String?>(null) }
    var codeError by remember { mutableStateOf<String?>(null) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Adicionar amigo") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = {
                        name = it.take(32)
                        nameError = null
                    },
                    label = { Text("Nome") },
                    singleLine = true,
                    isError = nameError != null,
                    supportingText = nameError?.let { { Text(it, color = colors.error) } },
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = code,
                    onValueChange = {
                        code = it
                        codeError = null
                    },
                    label = { Text("Código de amigo") },
                    placeholder = { Text("Cole o código de 64 caracteres") },
                    textStyle = LocalTextStyle.current.copy(fontFamily = FontFamily.Monospace),
                    maxLines = 3,
                    trailingIcon = {
                        IconButton(onClick = {
                            clipboardText(context)?.let { code = it.filterNot(Char::isWhitespace) }
                            codeError = null
                        }) {
                            Icon(painterResource(R.drawable.ic_content_paste), contentDescription = "Colar")
                        }
                    },
                    isError = codeError != null,
                    supportingText = codeError?.let { { Text(it, color = colors.error) } },
                    modifier = Modifier.fillMaxWidth()
                )
            }
        },
        confirmButton = {
            Button(onClick = {
                if (name.isBlank()) {
                    nameError = "Dê um nome para reconhecer o amigo."
                } else {
                    val err = vm.addFriend(name, code)
                    if (err != null) {
                        codeError = err
                    } else {
                        onAdded(name.trim())
                    }
                }
            }) {
                Text("Adicionar")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancelar")
            }
        }
    )
}
