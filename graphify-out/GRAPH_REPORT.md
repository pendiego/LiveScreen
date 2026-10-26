# Graph Report - LiveScreen-publish  (2026-10-25)

## Corpus Check
- 63 files · ~62,243 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 29 file(s) not represented in the graph (top: .xml 16, (none) 4, .ttf 4)

## Summary
- 1641 nodes · 3299 edges · 85 communities (59 shown, 26 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 70 edges (avg confidence: 0.89)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `27e77291`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- settings.rs
- capture.rs
- share_view.rs
- ToastKind
- convert.rs
- App
- sources.rs
- e2e_window.rs
- mf.rs
- protocol.rs
- Approach
- update.rs
- Ui
- playback.rs
- theme.rs
- watch_view.rs
- gpu.rs
- share.rs
- encoder.rs
- with
- ui/mod.rs
- Theme.kt
- HWND
- ShareSession
- AudioState
- sharing
- steps
- LiveScreen
- Encoder
- HMONITOR
- capture/mod.rs
- test-changelog.sh
- rust/src/lib.rs
- Native
- UpdateInfo
- frame_from_body
- write_msg
- main.rs
- present.rs
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- ID3D11Device
- codec_roundtrip.rs
- Native
- LiveScreen
- Theme.kt
- preset.rs
- Dib
- format_invite
- viewer.rs
- capture/mod.rs
- Preset
- VideoDecoder
- AudioPlayer.kt
- BudgetExceeded
- Codec
- wgc.rs
- HomeScreen
- ViewerModel
- run
- MainActivity.kt
- read_video_frame
- paint_surface
- Tab
- Callback
- HostMsg
- Viewer 0.5 assiste host 0.4 + mensagens de versão claras
- handle_frame
- watch_view.rs
- WgcCapture
- start

## God Nodes (most connected - your core abstractions)
1. `App` - 59 edges
2. `Preset` - 43 edges
3. `Converter` - 33 edges
4. `Codec` - 29 edges
5. `ViewerModel` - 27 edges
6. `EncoderKind` - 26 edges
7. `SourceId` - 25 edges
8. `ShareSession` - 25 edges
9. `EncodeParams` - 23 edges
10. `Encoder` - 23 edges

## Surprising Connections (you probably didn't know these)
- `Critical files & anchors` --references--> `AudioPlayer`  [INFERRED]
  .omp/plans/ANDROID_BATTERY_PLAN.md → android/app/src/main/java/io/github/pendiego/livescreen/AudioPlayer.kt
- `5. Android: `HomeScreen` (`Screens.kt` ≈L68–151)` --references--> `HomeScreen()`  [INFERRED]
  .omp/plans/INVITE_PASTE_UX_PLAN.md → android/app/src/main/java/io/github/pendiego/livescreen/Screens.kt
- `Critical files & anchors` --references--> `HomeScreen()`  [INFERRED]
  .omp/plans/INVITE_PASTE_UX_PLAN.md → android/app/src/main/java/io/github/pendiego/livescreen/Screens.kt
- `2. Decodificador de vídeo em modo assíncrono (`PKG/VideoDecoder.kt`)` --references--> `Status`  [INFERRED]
  .omp/plans/ANDROID_BATTERY_PLAN.md → android/app/src/main/java/io/github/pendiego/livescreen/ViewerModel.kt
- `2. Desktop: tela de entrada (`src/ui/watch_view.rs::input`)` --references--> `input()`  [EXTRACTED]
  .omp/plans/INVITE_PASTE_UX_PLAN.md → src/ui/watch_view.rs

## Import Cycles
- None detected.

## Communities (85 total, 26 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.11
Nodes (20): as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size(), Gpu (+12 more)

### Community 1 - "settings.rs"
Cohesion: 0.30
Nodes (14): button(), diagnostics(), FORCED, hint(), icon_button(), look(), share(), show() (+6 more)

### Community 2 - "capture.rs"
Cohesion: 0.05
Nodes (38): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+30 more)

### Community 3 - "share_view.rs"
Cohesion: 0.12
Nodes (16): SourceId, Monitor, Window, SourceInfo, bar_frame(), begin_start(), MAX_ROOM, Picker (+8 more)

### Community 4 - "ToastKind"
Cohesion: 0.29
Nodes (5): ToastKind, Info, Success, Warn, Toasts

### Community 5 - "convert.rs"
Cohesion: 0.06
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "App"
Cohesion: 0.14
Nodes (3): App, classify(), own_hwnd()

### Community 9 - "e2e_window.rs"
Cohesion: 0.18
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (18): enumerate_mfts(), friendly_name(), has_param_sets(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange (+10 more)

### Community 11 - "protocol.rs"
Cohesion: 0.09
Nodes (13): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, LEGACY_PROTOCOL_VERSION (+5 more)

### Community 12 - "Approach"
Cohesion: 0.14
Nodes (11): AudioPlayer, 0. Medir a linha de base, antes de qualquer edição, 1. JNI `setVisible` (`android/rust/src/lib.rs`, `PKG/Native.kt`), 4. Áudio: pausa ao ficar mudo ou em segundo plano, modo de desempenho padrão (`PKG/AudioPlayer.kt`), 5. Remover o diretório perdido, Approach, Assumptions & contingencies, Context (+3 more)

### Community 13 - "update.rs"
Cohesion: 0.10
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "Ui"
Cohesion: 0.11
Nodes (15): app_dir(), Config, load(), Overrides, overrides(), parse_overrides(), eframe::egui::ThemePreference, file_overrides_win() (+7 more)

### Community 15 - "playback.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.07
Nodes (23): app_icon(), DARK, GEIST, GEIST_MD, GEIST_MONO, GEIST_SB, I_COPY, I_MIC (+15 more)

### Community 17 - "watch_view.rs"
Cohesion: 0.14
Nodes (12): ended(), home(), tile(), I_CHECK, I_ERROR, I_INFO, I_SHARE, I_UPDATE (+4 more)

### Community 19 - "share.rs"
Cohesion: 0.06
Nodes (19): sanitize(), Abr, ABR_FLOOR, CountGuard, cut_feed(), handle_viewer(), HANDSHAKE_TIMEOUT, ids() (+11 more)

### Community 20 - "encoder.rs"
Cohesion: 0.14
Nodes (6): EncoderStats, black_nv12(), candidates(), hardware_kinds(), run_child(), try_encoder()

### Community 21 - "with"
Cohesion: 0.07
Nodes (32): runs-on, steps, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release, name (+24 more)

### Community 23 - "Theme.kt"
Cohesion: 0.20
Nodes (14): badge(), card(), checkbox(), chip(), danger_button(), field(), filled_button(), heading() (+6 more)

### Community 24 - "HWND"
Cohesion: 0.36
Nodes (6): fit(), source_size(), thumbnail(), thumbnail_monitor(), thumbnail_window(), window_rect()

### Community 25 - "ShareSession"
Cohesion: 0.09
Nodes (11): EncodeParams, is_reduction(), invite_addr(), Overload, Reconf, Resources, ShareSession, start_encoder() (+3 more)

### Community 26 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

### Community 27 - "sharing"
Cohesion: 0.20
Nodes (17): viewers_for(), ShareStatus, Upload, Done, Failed, Measuring, Unknown, Choices (+9 more)

### Community 28 - "steps"
Cohesion: 0.10
Nodes (23): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+15 more)

### Community 29 - "LiveScreen"
Cohesion: 0.17
Nodes (11): Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença, LiveScreen (+3 more)

### Community 30 - "Encoder"
Cohesion: 0.12
Nodes (14): EncodedFrame, configure(), CpuCopy, create_mft(), Encoder, Feed, Frame, Idle (+6 more)

### Community 31 - "HMONITOR"
Cohesion: 0.40
Nodes (7): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate()

### Community 32 - "capture/mod.rs"
Cohesion: 0.16
Nodes (4): CountSink, FirstFrameSink, FrameSink, SinkHandle

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.08
Nodes (25): AbortOnDrop, __android_log_write(), audio_loop(), get_session(), init_logging(), INIT_LOGGING, Java_io_github_pendiego_livescreen_Native_decoderFailed(), Java_io_github_pendiego_livescreen_Native_findInvite() (+17 more)

### Community 37 - "UpdateInfo"
Cohesion: 0.17
Nodes (10): GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle, Ready (+2 more)

### Community 38 - "frame_from_body"
Cohesion: 0.29
Nodes (4): frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 39 - "write_msg"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.06
Nodes (9): Mmcss, Display, GpuFrame, Imported, NEXT_SURFACES_ID, Producer, SLOTS, Surfaces (+1 more)

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "ID3D11Device"
Cohesion: 0.27
Nodes (4): Composer, Grab, GrabSink, texture()

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.14
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 50 - "Native"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 51 - "LiveScreen"
Cohesion: 0.25
Nodes (7): Comandos, Estilo, graphify, LiveScreen, Mapa do código, Regras, Release

### Community 52 - "Theme.kt"
Cohesion: 0.17
Nodes (3): colorScheme(), LiveScreenTheme(), StatusColors

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (25): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), lighter_is_accepted_equal_and_heavier_are_not(), LIMIT_FPS, LIMIT_PIXELS (+17 more)

### Community 55 - "format_invite"
Cohesion: 0.19
Nodes (11): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), verify_room_proof() (+3 more)

### Community 56 - "viewer.rs"
Cohesion: 0.06
Nodes (22): ViewerMsg, Hello, RequestKeyframe, SetQuality, SetVisible, BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64 (+14 more)

### Community 57 - "capture/mod.rs"
Cohesion: 0.27
Nodes (9): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+1 more)

### Community 58 - "Preset"
Cohesion: 0.15
Nodes (11): Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60, P2160p60 (+3 more)

### Community 61 - "AudioPlayer.kt"
Cohesion: 0.50
Nodes (3): exe_name(), inspect_window(), window_cb()

### Community 63 - "Codec"
Cohesion: 0.19
Nodes (10): Codec, Av1, H264, Hevc, pick_encoder(), EncoderKind, Hardware, Software (+2 more)

### Community 65 - "HomeScreen"
Cohesion: 0.12
Nodes (22): App(), clipboardText(), HomeScreen(), UpdateBanner(), WatchContent(), WatchScreen(), 1. Ícones vetoriais (Material Symbols Outlined), 2. Tema M3 (`PKG/Theme.kt`, `PKG/MainActivity.kt`, `android/app/build.gradle.kts`) (+14 more)

### Community 66 - "ViewerModel"
Cohesion: 0.29
Nodes (3): Status, ViewerModel, 3. Visibilidade: vídeo e áudio param em segundo plano (`PKG/ViewerModel.kt`)

### Community 67 - "run"
Cohesion: 0.31
Nodes (5): create_duplication(), dxgi_draws_cursor(), DxgiCapture, run(), start()

### Community 68 - "MainActivity.kt"
Cohesion: 0.12
Nodes (7): MainActivity, 1. Core: `find_invite` (base de tudo), 2. Desktop: tela de entrada (`src/ui/watch_view.rs::input`), 3. Desktop: preenchimento ao abrir (`src/ui/home.rs` ≈L156–165), 4. Android: JNI (`android/rust/src/lib.rs`), 5. Android: `HomeScreen` (`Screens.kt` ≈L68–151), Approach

### Community 69 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 72 - "Tab"
Cohesion: 0.29
Nodes (7): Tab, Diagnostics, Look, Share, Updates, Video, TABS

### Community 74 - "HostMsg"
Cohesion: 0.40
Nodes (5): HostMsg, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "Viewer 0.5 assiste host 0.4 + mensagens de versão claras"
Cohesion: 0.20
Nodes (9): 1. `parse_invite` aceita ticket puro (core), 2. Viewer fala protocolo 4 quando o convite não tem chave (core), 3. Host explica a recusa por versão (Windows), Approach, Assumptions & contingencies, Context, Critical files & anchors, Verification (+1 more)

### Community 83 - "watch_view.rs"
Cohesion: 0.07
Nodes (16): I_BACK, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_MUTE, I_PASTE, I_VOLUME, connect(), ControlAction (+8 more)

### Community 85 - "start"
Cohesion: 0.67
Nodes (3): border_removable(), property_present(), start()

## Knowledge Gaps
- **73 isolated node(s):** `livescreen`, `livescreen-android`, `P480p30`, `P720p30`, `P720p60` (+68 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 631 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **26 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `settings.rs`, `share_view.rs`, `ToastKind`, `Tab`, `update.rs`, `Ui`, `watch_view.rs`, `Native`, `watch_view.rs`, `ui/mod.rs`, `format_invite`, `viewer.rs`, `ShareSession`, `sharing`, `Codec`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **What connects `livescreen`, `livescreen-android`, `P480p30` to the rest of the system?**
  _73 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10631229235880399 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `ViewerModel` to `HomeScreen`, `Native`, `MainActivity.kt`, `UpdateInfo`, `Approach`, `ViewerModel`, `VideoDecoder`?**
  _High betweenness centrality (0.076) - this node is a cross-community bridge._
- **Should `capture.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.052313883299798795 - nodes in this community are weakly interconnected._
- **Why does `HomeScreen()` connect `HomeScreen` to `Screens.kt`, `Native`, `MainActivity.kt`, `ViewerModel`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Should `share_view.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12310606060606061 - nodes in this community are weakly interconnected._