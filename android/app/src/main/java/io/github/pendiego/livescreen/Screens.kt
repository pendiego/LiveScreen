package io.github.pendiego.livescreen

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.pm.ActivityInfo
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarVisuals
import androidx.compose.material3.Text
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
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.foundation.Image
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.window.Dialog
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.compose.LifecycleStartEffect
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

private const val LEGACY_MSG = "Código sem chave de sala: só conecta em quem transmite com o LiveScreen 0.4. Se veio de uma versão mais nova, falta o final do código (a parte depois do ponto)."

// Mesmo `OVERLAY_TIMEOUT` do desktop (watch_view.rs).
private const val OVERLAY_TIMEOUT_MS = 2500L

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

// region Componentes Sinal

private enum class BtnKind { Primary, Secondary, Ghost, Danger }

@Composable
private fun SinalButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    kind: BtnKind = BtnKind.Primary,
    height: Dp = 40.dp,
    hPad: Dp = 20.dp,
    icon: Int? = null,
    enabled: Boolean = true
) {
    val (fill, contentColor) = when (kind) {
        BtnKind.Primary -> Sinal.accent to Sinal.onAccent
        BtnKind.Secondary -> Sinal.raised to Sinal.text
        BtnKind.Ghost -> Color.Transparent to Sinal.text
        BtnKind.Danger -> Sinal.danger to Sinal.onDanger
    }
    Row(
        modifier
            .alpha(if (enabled) 1f else 0.4f)
            .height(height)
            .clip(CircleShape)
            .background(fill)
            .then(if (kind == BtnKind.Secondary) Modifier.border(1.dp, Sinal.border, CircleShape) else Modifier)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(horizontal = hPad),
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (icon != null) {
            Icon(
                painterResource(icon),
                null,
                tint = contentColor,
                modifier = Modifier.size(if (height >= 48.dp) 20.dp else 16.dp)
            )
        }
        Text(
            text,
            style = if (height >= 48.dp) MaterialTheme.typography.titleMedium else MaterialTheme.typography.labelLarge,
            color = contentColor,
            maxLines = 1
        )
    }
}

private enum class FieldTone { Normal, Error, Warn }

@Composable
private fun SinalField(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    modifier: Modifier = Modifier,
    label: String? = null,
    mono: Boolean = false,
    leadingIcon: Int? = null,
    trailing: (@Composable () -> Unit)? = null,
    tone: FieldTone = FieldTone.Normal,
    help: String? = null,
    singleLine: Boolean = true,
    maxLines: Int = 1,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    keyboardActions: KeyboardActions = KeyboardActions.Default
) {
    val src = remember { MutableInteractionSource() }
    val focused by src.collectIsFocusedAsState()
    val (borderWidth, borderColor) = when {
        tone == FieldTone.Error -> 2.dp to Sinal.danger
        tone == FieldTone.Warn -> 1.dp to Sinal.warn
        focused -> 2.dp to Sinal.accent
        else -> 1.dp to Sinal.border
    }
    val textStyle = (if (mono) MonoStyle else MaterialTheme.typography.bodyLarge).copy(color = Sinal.text)

    Column(modifier = modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (label != null) {
            Text(label, style = MaterialTheme.typography.titleSmall, color = Sinal.textMuted)
        }
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
            singleLine = singleLine,
            maxLines = maxLines,
            textStyle = textStyle,
            cursorBrush = SolidColor(Sinal.accent),
            interactionSource = src,
            keyboardOptions = keyboardOptions,
            keyboardActions = keyboardActions,
            decorationBox = { innerTextField ->
                Row(
                    Modifier
                        .fillMaxWidth()
                        .heightIn(min = 48.dp)
                        .background(Sinal.bg, RoundedCornerShape(12.dp))
                        .border(borderWidth, borderColor, RoundedCornerShape(12.dp))
                        .padding(start = 12.dp, end = if (trailing != null) 4.dp else 12.dp, top = 8.dp, bottom = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    if (leadingIcon != null) {
                        Icon(painterResource(leadingIcon), null, tint = Sinal.textMuted, modifier = Modifier.size(18.dp))
                    }
                    Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                        if (value.isEmpty()) {
                            Text(placeholder, style = textStyle.copy(color = Sinal.textFaint))
                        }
                        innerTextField()
                    }
                    trailing?.invoke()
                }
            }
        )
        if (help != null) {
            Text(
                help,
                style = MaterialTheme.typography.bodySmall,
                color = when (tone) {
                    FieldTone.Error -> Sinal.danger
                    FieldTone.Warn -> Sinal.warn
                    FieldTone.Normal -> Sinal.textMuted
                }
            )
        }
    }
}

@Composable
private fun SinalDialog(onDismiss: () -> Unit, content: @Composable ColumnScope.() -> Unit) {
    Dialog(onDismissRequest = onDismiss) {
        Column(
            Modifier
                .widthIn(max = 340.dp)
                .fillMaxWidth()
                .shadow(16.dp, RoundedCornerShape(20.dp))
                .background(Sinal.raised, RoundedCornerShape(20.dp))
                .border(1.dp, Sinal.border, RoundedCornerShape(20.dp))
                .padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
            content = content
        )
    }
}

private enum class CallState { Neutral, Active, Alert, HangUp }

@Composable
private fun CallButton(
    icon: Int,
    description: String,
    onClick: () -> Unit,
    state: CallState = CallState.Neutral
) {
    val (fill, ic) = when (state) {
        CallState.Neutral -> Sinal.raised to Sinal.text
        CallState.Active -> Sinal.accent to Sinal.onAccent
        CallState.Alert -> Sinal.danger.copy(alpha = 0.16f) to Sinal.danger
        CallState.HangUp -> Sinal.danger to Sinal.onDanger
    }
    Box(
        Modifier
            .size(width = if (state == CallState.HangUp) 64.dp else 48.dp, height = 48.dp)
            .clip(CircleShape)
            .background(fill)
            .then(if (state == CallState.Neutral) Modifier.border(1.dp, Sinal.border, CircleShape) else Modifier)
            .clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(painterResource(icon), description, tint = ic, modifier = Modifier.size(22.dp))
    }
}

@Composable
private fun Badge(text: String, color: Color, container: Color) {
    Text(
        text,
        style = MaterialTheme.typography.labelMedium,
        color = color,
        modifier = Modifier
            .background(container, CircleShape)
            .padding(horizontal = 8.dp, vertical = 2.dp)
    )
}

private enum class SnackTone { Success, Error }

private class SinalSnack(override val message: String, val tone: SnackTone) : SnackbarVisuals {
    override val actionLabel: String? = null
    override val withDismissAction = false
    override val duration = SnackbarDuration.Short
}

// endregion

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
    val paste: () -> Unit = {
        val inv = clipboardText(context)?.let(Native::findInvite)
        if (inv == null) {
            scope.launch { snackbar.showSnackbar(SinalSnack("O texto colado não tem um código de convite", SnackTone.Error)) }
        } else {
            vm.ticket = inv
            if (!Native.isLegacyInvite(inv) && !vm.starting) vm.connect()
        }
    }

    Scaffold(
        containerColor = Sinal.bg,
        snackbarHost = {
            SnackbarHost(snackbar) { data ->
                val tone = (data.visuals as? SinalSnack)?.tone ?: SnackTone.Success
                Row(
                    Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 24.dp)
                        .shadow(8.dp, RoundedCornerShape(12.dp))
                        .background(Sinal.raised, RoundedCornerShape(12.dp))
                        .border(1.dp, Sinal.border, RoundedCornerShape(12.dp))
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        painterResource(if (tone == SnackTone.Success) R.drawable.ic_circle_check else R.drawable.ic_circle_x),
                        null,
                        tint = if (tone == SnackTone.Success) Sinal.success else Sinal.danger,
                        modifier = Modifier.size(18.dp)
                    )
                    Text(data.visuals.message, style = MaterialTheme.typography.bodyLarge, color = Sinal.text)
                }
            }
        },
        bottomBar = {
            Box(
                Modifier
                    .fillMaxWidth()
                    .windowInsetsPadding(WindowInsets.navigationBars.union(WindowInsets.ime))
                    .padding(horizontal = 24.dp, vertical = 12.dp)
            ) {
                SinalButton(
                    if (vm.starting) "Conectando…" else "Assistir pelo convite",
                    onClick = { vm.connect() },
                    height = 56.dp,
                    hPad = 24.dp,
                    enabled = isTicketValid && !vm.starting,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        }
    ) { inner ->
        Box(
            Modifier
                .fillMaxSize()
                .padding(inner)
                .consumeWindowInsets(inner),
            contentAlignment = Alignment.TopCenter
        ) {
            Column(
                Modifier
                    .widthIn(max = 560.dp)
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 24.dp, vertical = 24.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Column(
                    Modifier.fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Image(painterResource(R.drawable.ic_logo), null, Modifier.size(40.dp))
                        Spacer(Modifier.width(12.dp))
                        Text("LiveScreen", style = MaterialTheme.typography.titleLarge, color = Sinal.text)
                    }
                    Text(
                        "Assista a uma tela compartilhada pelo LiveScreen do Windows.",
                        style = MaterialTheme.typography.bodyLarge,
                        color = Sinal.textMuted,
                        textAlign = TextAlign.Center
                    )
                }

                UpdateBanner(vm)

                vm.lastEnd?.let { end ->
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .background(Sinal.danger.copy(alpha = 0.16f), RoundedCornerShape(20.dp))
                            .border(1.dp, Sinal.danger, RoundedCornerShape(20.dp))
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            painterResource(R.drawable.ic_circle_x),
                            null,
                            tint = Sinal.danger,
                            modifier = Modifier.size(20.dp)
                        )
                        Text(end, style = MaterialTheme.typography.bodyLarge, color = Sinal.danger)
                    }
                }

                if (vm.friends.isNotEmpty()) {
                    FriendsSection(
                        vm = vm,
                        onAddClick = { showAddDialog = true },
                        onRemoveFriend = { friendToRemove = it },
                        onCopied = {
                            scope.launch {
                                snackbar.showSnackbar(SinalSnack("Código de amigo copiado", SnackTone.Success))
                            }
                        }
                    )
                }

                SinalField(
                    value = vm.ticket,
                    onValueChange = { vm.ticket = Native.findInvite(it) ?: it.filterNot(Char::isWhitespace) },
                    placeholder = "Cole o código que você recebeu",
                    label = "Código de convite",
                    mono = true,
                    singleLine = false,
                    maxLines = 4,
                    trailing = {
                        IconButton(onClick = paste, modifier = Modifier.size(40.dp)) {
                            Icon(
                                painterResource(R.drawable.ic_clipboard_paste),
                                "Colar",
                                tint = Sinal.textMuted,
                                modifier = Modifier.size(18.dp)
                            )
                        }
                    },
                    tone = when {
                        isTicketNotEmpty && !isTicketValid -> FieldTone.Error
                        isTicketValid && isLegacy -> FieldTone.Warn
                        else -> FieldTone.Normal
                    },
                    help = when {
                        isTicketNotEmpty && !isTicketValid -> "Código inválido — confira se copiou o código inteiro."
                        isTicketValid && isLegacy -> LEGACY_MSG
                        else -> null
                    },
                    keyboardOptions = KeyboardOptions(
                        autoCorrectEnabled = false,
                        keyboardType = KeyboardType.Ascii,
                        imeAction = ImeAction.Go
                    ),
                    keyboardActions = KeyboardActions(onGo = { if (isTicketValid && !vm.starting) vm.connect() })
                )

                SinalField(
                    value = vm.name,
                    onValueChange = { vm.name = it },
                    placeholder = "Seu nome (opcional)",
                    leadingIcon = R.drawable.ic_user,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done)
                )

                if (vm.friends.isEmpty()) {
                    FriendsSection(
                        vm = vm,
                        onAddClick = { showAddDialog = true },
                        onRemoveFriend = { friendToRemove = it },
                        onCopied = {
                            scope.launch {
                                snackbar.showSnackbar(SinalSnack("Código de amigo copiado", SnackTone.Success))
                            }
                        }
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
                    snackbar.showSnackbar(
                        SinalSnack("$addedName adicionado aos amigos. Peça para ele adicionar você também.", SnackTone.Success)
                    )
                }
            }
        )
    }

    friendToRemove?.let { friend ->
        SinalDialog({ friendToRemove = null }) {
            Column(
                Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Box(
                    Modifier
                        .size(56.dp)
                        .background(Sinal.danger.copy(alpha = 0.16f), CircleShape),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(painterResource(R.drawable.ic_trash), null, tint = Sinal.danger, modifier = Modifier.size(24.dp))
                }
                Text(
                    "Remover ${friend.name}?",
                    style = MaterialTheme.typography.titleLarge,
                    color = Sinal.text,
                    textAlign = TextAlign.Center
                )
                Text(
                    "Você deixará de ver quando esse amigo estiver transmitindo.",
                    style = MaterialTheme.typography.bodyLarge,
                    color = Sinal.textMuted,
                    textAlign = TextAlign.Center
                )
                Row(
                    Modifier.fillMaxWidth().padding(top = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally)
                ) {
                    SinalButton("Cancelar", { friendToRemove = null }, kind = BtnKind.Ghost, height = 48.dp, hPad = 16.dp)
                    SinalButton(
                        "Remover",
                        {
                            vm.removeFriend(friend.code)
                            friendToRemove = null
                        },
                        kind = BtnKind.Danger,
                        height = 48.dp,
                        hPad = 24.dp
                    )
                }
            }
        }
    }

    (vm.update as? UpdateState.Available)?.takeIf { !vm.updatePromptDismissed && !showAddDialog && friendToRemove == null }?.let { s ->
        UpdatePrompt(
            s.info,
            onLater = { vm.updatePromptDismissed = true },
            onUpdate = {
                vm.updatePromptDismissed = true
                vm.startUpdate()
            }
        )
    }
}

@Composable
private fun UpdatePrompt(info: UpdateInfo, onLater: () -> Unit, onUpdate: () -> Unit) = SinalDialog(onLater) {
    Column(
        Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Box(
            Modifier
                .size(56.dp)
                .background(Sinal.accent.copy(alpha = 0.16f), CircleShape),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                painterResource(R.drawable.ic_download),
                null,
                tint = Sinal.accent,
                modifier = Modifier.size(24.dp)
            )
        }
        Text(
            "Atualização disponível",
            style = MaterialTheme.typography.titleLarge,
            color = Sinal.text,
            textAlign = TextAlign.Center
        )
        Text(
            "A versão ${info.version} está disponível. Você está na ${BuildConfig.VERSION_NAME}.",
            style = MaterialTheme.typography.bodyLarge,
            color = Sinal.textMuted,
            textAlign = TextAlign.Center
        )
        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 4.dp)
        ) {
            SinalButton("Agora não", onLater, kind = BtnKind.Ghost, height = 48.dp, hPad = 16.dp)
            SinalButton("Atualizar", onUpdate, height = 48.dp, hPad = 24.dp, icon = R.drawable.ic_download)
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun WatchScreen(vm: ViewerModel) {
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
                CircularProgressIndicator(
                    modifier = Modifier.size(36.dp),
                    color = Sinal.accent,
                    trackColor = Color.Transparent,
                    strokeWidth = 3.dp
                )
                Text("Conectando…", style = MaterialTheme.typography.titleLarge, color = Sinal.text)
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
                val (badgeColor, badgeContainer) = when {
                    status?.connecting == true -> Sinal.textMuted to Sinal.raised
                    status?.path == "Relay" -> Sinal.warn to Sinal.warn.copy(alpha = 0.16f)
                    else -> Sinal.success to Sinal.success.copy(alpha = 0.16f)
                }
                Row(
                    Modifier
                        .align(Alignment.TopCenter)
                        .fillMaxWidth()
                        .background(Brush.verticalGradient(listOf(Color.Black.copy(alpha = 0.7f), Color.Transparent)))
                        .windowInsetsPadding(WindowInsets.displayCutout)
                        .padding(start = 24.dp, end = 24.dp, top = 12.dp, bottom = 24.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        status?.title?.takeIf { it.isNotEmpty() } ?: "LiveScreen",
                        style = MaterialTheme.typography.titleSmall,
                        color = if (status?.connecting == true) Sinal.textMuted else Sinal.text,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f)
                    )
                    Badge(badgeText, badgeColor, badgeContainer)
                }

                val rttStr = status?.rttMs?.let { "$it ms" } ?: "- ms"
                val fpsStr = "${status?.fps ?: 0} fps"
                val infoText = "$rttStr · $fpsStr · ${w}×${h} · decodificação ${if (status?.decoderHw == true) "por hardware" else "por software"}"
                Column(
                    Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .windowInsetsPadding(WindowInsets.displayCutout)
                        .padding(bottom = 16.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    if (showStats) {
                        Text(
                            infoText,
                            style = MaterialTheme.typography.bodySmall,
                            color = Sinal.textMuted,
                            modifier = Modifier
                                .background(Sinal.surface, CircleShape)
                                .border(1.dp, Sinal.border, CircleShape)
                                .padding(horizontal = 12.dp, vertical = 4.dp)
                        )
                    }
                    Row(
                        Modifier
                            .background(Sinal.raised, CircleShape)
                            .border(1.dp, Sinal.border, CircleShape)
                            .padding(8.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        if (hostId.isNotEmpty() && status?.connecting == false && vm.friends.none { it.code == hostId }) {
                            CallButton(
                                icon = R.drawable.ic_user_plus,
                                description = "Adicionar aos amigos",
                                onClick = { addHost = true; bump() }
                            )
                        }
                        if (status?.audio == true) {
                            CallButton(
                                icon = if (vm.muted) R.drawable.ic_volume_x else R.drawable.ic_volume_2,
                                description = if (vm.muted) "Ativar som" else "Silenciar",
                                onClick = { vm.muted = !vm.muted; bump() },
                                state = if (vm.muted) CallState.Alert else CallState.Neutral
                            )
                        }
                        CallButton(
                            icon = R.drawable.ic_sliders,
                            description = "Qualidade",
                            onClick = { qualitySheet = true; bump() }
                        )
                        CallButton(
                            icon = R.drawable.ic_chart,
                            description = "Estatísticas",
                            onClick = { showStats = !showStats; bump() },
                            state = if (showStats) CallState.Active else CallState.Neutral
                        )
                        CallButton(
                            icon = R.drawable.ic_rotate_cw,
                            description = "Girar tela",
                            onClick = {
                                landscape = !landscape
                                activity?.requestedOrientation =
                                    if (landscape) ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
                                    else ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
                                bump()
                            },
                            state = if (landscape) CallState.Active else CallState.Neutral
                        )
                        CallButton(
                            icon = R.drawable.ic_phone_off,
                            description = "Sair",
                            onClick = { vm.leave() },
                            state = CallState.HangUp
                        )
                    }
                }
            }
        }

        if (qualitySheet) {
            ModalBottomSheet(
                onDismissRequest = { qualitySheet = false; bump() },
                sheetMaxWidth = 560.dp,
                containerColor = Sinal.raised,
                shape = RoundedCornerShape(topStart = 20.dp, topEnd = 20.dp),
                scrimColor = Color.Black.copy(alpha = 0.6f),
                dragHandle = {
                    Box(
                        Modifier
                            .padding(top = 12.dp, bottom = 8.dp)
                            .size(width = 36.dp, height = 4.dp)
                            .background(Sinal.textFaint, RoundedCornerShape(2.dp))
                    )
                }
            ) {
                Column(
                    Modifier.padding(start = 24.dp, end = 24.dp, bottom = 16.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    Text("Qualidade", style = MaterialTheme.typography.titleLarge, color = Sinal.text)
                    Text(
                        "Só vale se for mais leve que a atual",
                        style = MaterialTheme.typography.bodyLarge,
                        color = Sinal.textMuted
                    )
                    for ((id, name) in vm.presets) {
                        Row(
                            Modifier
                                .fillMaxWidth()
                                .height(50.dp)
                                .clip(RoundedCornerShape(12.dp))
                                .background(Sinal.surface)
                                .border(1.dp, Sinal.border, RoundedCornerShape(12.dp))
                                .clickable {
                                    vm.requestQuality(id)
                                    qualitySheet = false
                                    bump()
                                }
                                .padding(horizontal = 16.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                name,
                                style = MaterialTheme.typography.titleSmall,
                                color = Sinal.text,
                                modifier = Modifier.weight(1f)
                            )
                            Icon(
                                painterResource(R.drawable.ic_chevron_right),
                                null,
                                tint = Sinal.textMuted,
                                modifier = Modifier.size(18.dp)
                            )
                        }
                    }
                }
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
    Column(
        Modifier
            .fillMaxWidth()
            .background(Sinal.raised, RoundedCornerShape(20.dp))
            .border(1.dp, Sinal.border, RoundedCornerShape(20.dp))
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                painterResource(R.drawable.ic_download),
                null,
                tint = Sinal.accent,
                modifier = Modifier.size(20.dp)
            )
            Text(
                "Versão ${info.version} disponível",
                style = MaterialTheme.typography.bodyLarge,
                color = Sinal.text,
                modifier = Modifier.weight(1f)
            )
            SinalButton(label, { vm.startUpdate() }, height = 32.dp, hPad = 14.dp, enabled = s !is UpdateState.Downloading)
        }
        if (s is UpdateState.Failed) {
            Text(
                s.message,
                style = MaterialTheme.typography.bodySmall,
                color = Sinal.danger
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

    Row(
        Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text("Amigos", style = MaterialTheme.typography.titleLarge, color = Sinal.text, modifier = Modifier.weight(1f))
        SinalButton(
            "Adicionar amigo",
            onAddClick,
            kind = BtnKind.Secondary,
            height = 32.dp,
            hPad = 12.dp,
            icon = R.drawable.ic_user_plus
        )
    }

    Row(
        Modifier
            .fillMaxWidth()
            .background(Sinal.raised, RoundedCornerShape(20.dp))
            .border(1.dp, Sinal.border, RoundedCornerShape(20.dp))
            .padding(start = 16.dp, end = 4.dp, top = 12.dp, bottom = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text("Seu código de amigo", style = MaterialTheme.typography.bodySmall, color = Sinal.textMuted)
            Text(
                vm.myCode,
                style = MonoStyle,
                color = Sinal.text,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
        IconButton(onClick = {
            val cm = context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
            cm?.setPrimaryClip(ClipData.newPlainText("LiveScreen", vm.myCode))
            onCopied()
        }) {
            Icon(painterResource(R.drawable.ic_copy), "Copiar", tint = Sinal.textMuted, modifier = Modifier.size(20.dp))
        }
    }

    if (vm.friends.isEmpty()) {
        Text(
            "Adicione o código de um amigo que transmite pelo Windows. Ele também precisa adicionar o seu.",
            style = MaterialTheme.typography.bodyLarge,
            color = Sinal.textMuted
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
                    "live" -> Sinal.success
                    "offline" -> Sinal.textFaint
                    "not_friend" -> Sinal.warn
                    "removed" -> Sinal.danger
                    else -> Sinal.textFaint
                }

                Row(
                    Modifier
                        .fillMaxWidth()
                        .heightIn(min = 48.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Text(friend.name, style = MaterialTheme.typography.titleSmall, color = Sinal.text)
                        Text(subtitle, style = MaterialTheme.typography.bodySmall, color = subtitleColor)
                    }
                    if (state != "not_friend" && state != "removed") {
                        SinalButton(
                            "Assistir",
                            { vm.watchFriend(friend.code) },
                            height = 32.dp,
                            hPad = 14.dp,
                            enabled = isLive && !vm.starting
                        )
                    }
                    IconButton(onClick = { onRemoveFriend(friend) }) {
                        Icon(
                            painterResource(R.drawable.ic_trash),
                            contentDescription = "Remover ${friend.name}",
                            tint = Sinal.textMuted,
                            modifier = Modifier.size(20.dp)
                        )
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
    var name by remember { mutableStateOf(initialName) }
    var code by remember { mutableStateOf(initialCode) }
    var nameError by remember { mutableStateOf<String?>(null) }
    var codeError by remember { mutableStateOf<String?>(null) }

    SinalDialog(onDismiss) {
        Text("Adicionar amigo", style = MaterialTheme.typography.titleLarge, color = Sinal.text)
        SinalField(
            value = name,
            onValueChange = {
                name = it.take(32)
                nameError = null
            },
            placeholder = "Como você quer reconhecer o amigo",
            label = "Nome",
            tone = if (nameError != null) FieldTone.Error else FieldTone.Normal,
            help = nameError
        )
        SinalField(
            value = code,
            onValueChange = {
                code = it
                codeError = null
            },
            placeholder = "Cole o código de 64 caracteres",
            label = "Código de amigo",
            mono = true,
            singleLine = false,
            maxLines = 3,
            trailing = {
                IconButton(onClick = {
                    clipboardText(context)?.let { code = it.filterNot(Char::isWhitespace) }
                    codeError = null
                }, modifier = Modifier.size(40.dp)) {
                    Icon(
                        painterResource(R.drawable.ic_clipboard_paste),
                        "Colar",
                        tint = Sinal.textMuted,
                        modifier = Modifier.size(18.dp)
                    )
                }
            },
            tone = if (codeError != null) FieldTone.Error else FieldTone.Normal,
            help = codeError
        )
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End)
        ) {
            SinalButton("Cancelar", onDismiss, kind = BtnKind.Ghost, hPad = 16.dp)
            SinalButton("Adicionar", {
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
            }, hPad = 20.dp)
        }
    }
}
