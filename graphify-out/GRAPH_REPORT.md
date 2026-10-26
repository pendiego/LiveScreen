# Graph Report - LiveScreen-publish  (2026-10-26)

## Corpus Check
- 64 files · ~67,688 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 32 file(s) not represented in the graph (top: .xml 19, (none) 4, .ttf 4)

## Summary
- 1789 nodes · 3641 edges · 86 communities (61 shown, 25 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 107 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ed7c1629`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- config.rs
- capture.rs
- share_view.rs
- ToastKind
- convert.rs
- App
- sources.rs
- e2e_window.rs
- mf.rs
- protocol.rs
- update.rs
- Approach
- playback.rs
- theme.rs
- watch_view.rs
- gpu.rs
- run
- probe.rs
- with
- ui/mod.rs
- Theme.kt
- SourceInfo
- ShareSession
- Shared
- share.rs
- steps
- LiveScreen
- Encoder
- HMONITOR
- SinkHandle
- test-changelog.sh
- rust/src/lib.rs
- Native
- src/friends.rs
- UpdateInfo
- frame_from_body
- write_msg
- main.rs
- present.rs
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- wgc.rs
- codec_roundtrip.rs
- Native
- EncodeParams
- Theme.kt
- preset.rs
- Dib
- format_invite
- viewer.rs
- capture/mod.rs
- BudgetExceeded
- Resources
- encoder.rs
- Preset
- Sink
- omp config sync via private GitHub repo
- LiveScreen
- HomeScreen
- ViewerModel
- DxgiCapture
- MainActivity.kt
- read_video_frame
- VideoDecoder.kt
- handle_frame
- AudioPlayer
- Callback
- HostMsg
- WgcCapture
- start
- ui/friends.rs
- watch_view.rs
- .start
- Target

## God Nodes (most connected - your core abstractions)
1. `App` - 68 edges
2. `ViewerModel` - 42 edges
3. `Preset` - 42 edges
4. `Converter` - 33 edges
5. `Codec` - 29 edges
6. `ShareSession` - 26 edges
7. `EncoderKind` - 26 edges
8. `SourceId` - 25 edges
9. `Native` - 23 edges
10. `EncodeParams` - 23 edges

## Surprising Connections (you probably didn't know these)
- `Android (`android/`)` --references--> `AudioPlayer`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/AudioPlayer.kt
- `Android (`android/`)` --references--> `MainActivity`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/MainActivity.kt
- `Android (`android/`)` --references--> `Native`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/Native.kt
- `2. Desktop: uma identidade por janela` --references--> `App()`  [INFERRED]
  AMIGOS_CORREÇÕES_PLAN.md → android/app/src/main/java/io/github/pendiego/livescreen/Screens.kt
- `3. Desktop: ordem de fechamento e abertura dos endpoints` --references--> `App()`  [INFERRED]
  AMIGOS_CORREÇÕES_PLAN.md → android/app/src/main/java/io/github/pendiego/livescreen/Screens.kt

## Import Cycles
- None detected.

## Communities (86 total, 25 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.13
Nodes (20): EncodedFrame, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size() (+12 more)

### Community 1 - "config.rs"
Cohesion: 0.09
Nodes (22): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+14 more)

### Community 2 - "capture.rs"
Cohesion: 0.05
Nodes (38): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+30 more)

### Community 3 - "share_view.rs"
Cohesion: 0.09
Nodes (20): bar_frame(), begin_start(), Picker, picker(), poll_starting(), Preview, source_card(), Starting (+12 more)

### Community 4 - "ToastKind"
Cohesion: 0.29
Nodes (5): ToastKind, Info, Success, Warn, Toasts

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "App"
Cohesion: 0.06
Nodes (33): App, classify(), own_hwnd(), button(), diagnostics(), FORCED, hint(), icon_button() (+25 more)

### Community 9 - "e2e_window.rs"
Cohesion: 0.21
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (17): enumerate_mfts(), friendly_name(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample (+9 more)

### Community 11 - "protocol.rs"
Cohesion: 0.09
Nodes (13): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, LEGACY_PROTOCOL_VERSION (+5 more)

### Community 13 - "update.rs"
Cohesion: 0.10
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "Approach"
Cohesion: 0.14
Nodes (13): 1. Ícone novo e helper de normalização, 2. Tile primário de compartilhar (substitui `tile()` em `home.rs`), 3. Bloco "Assistir" com convite inline (coluna esquerda, logo abaixo do tile), 4. Layout de duas colunas em `home()` (CentralPanel), 5. Painel Amigos (`friends.rs::card`), 6. Chip do encoder no rodapé (`home()`, `Panel::bottom("home_footer")`), 7. Imports e limpeza, Approach (+5 more)

### Community 15 - "playback.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.09
Nodes (14): app_icon(), DARK, field(), GEIST, GEIST_MD, GEIST_MONO, GEIST_SB, I_WINDOW (+6 more)

### Community 17 - "watch_view.rs"
Cohesion: 0.12
Nodes (14): ended(), home(), left_column(), share_tile(), I_CHECK, I_CHEVRON, I_ERROR, I_INFO (+6 more)

### Community 19 - "run"
Cohesion: 0.26
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 20 - "probe.rs"
Cohesion: 0.12
Nodes (16): Codec, Av1, H264, Hevc, supported_codecs(), EncoderKind, Hardware, Software (+8 more)

### Community 21 - "with"
Cohesion: 0.07
Nodes (32): runs-on, steps, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release, name (+24 more)

### Community 23 - "Theme.kt"
Cohesion: 0.20
Nodes (15): badge(), card(), checkbox(), chip(), danger_button(), filled_button(), ghost_icon_button(), heading() (+7 more)

### Community 24 - "SourceInfo"
Cohesion: 0.40
Nodes (4): exe_name(), inspect_window(), SourceInfo, window_cb()

### Community 25 - "ShareSession"
Cohesion: 0.20
Nodes (4): is_reduction(), pick_encoder(), ShareSession, unpack_size()

### Community 26 - "Shared"
Cohesion: 0.15
Nodes (7): sanitize(), admit(), answer_presence(), CountGuard, handle_viewer(), Peer, Shared

### Community 27 - "share.rs"
Cohesion: 0.11
Nodes (9): Abr, ABR_FLOOR, HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), MAX_HANDSHAKES, retire() (+1 more)

### Community 28 - "steps"
Cohesion: 0.10
Nodes (23): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+15 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (só para assistir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "Encoder"
Cohesion: 0.18
Nodes (7): CpuCopy, Encoder, Feed, Frame, Idle, Stop, Nv12Frame

### Community 31 - "HMONITOR"
Cohesion: 0.19
Nodes (16): device_name(), enumerate(), fit(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate() (+8 more)

### Community 32 - "SinkHandle"
Cohesion: 0.15
Nodes (4): CountSink, FirstFrameSink, FrameSink, SinkHandle

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.08
Nodes (33): AbortOnDrop, __android_log_write(), audio_loop(), get_session(), init_logging(), INIT_LOGGING, Java_io_github_pendiego_livescreen_Native_decoderFailed(), Java_io_github_pendiego_livescreen_Native_findInvite() (+25 more)

### Community 35 - "Native"
Cohesion: 0.13
Nodes (3): Amigos e presença, Regras, Native

### Community 36 - "src/friends.rs"
Cohesion: 0.05
Nodes (36): Amigos: correções de bugs e UI/UX (desktop + Android), Assumptions & contingencies, Context, Critical files & anchors, Verification, AbortJoinSet, denied_reason(), dial_endpoint() (+28 more)

### Community 37 - "UpdateInfo"
Cohesion: 0.17
Nodes (11): Android (`android/`), GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle (+3 more)

### Community 38 - "frame_from_body"
Cohesion: 0.33
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

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.12
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 50 - "Native"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 51 - "EncodeParams"
Cohesion: 0.24
Nodes (14): ShareStatus, Upload, Done, Failed, Measuring, Unknown, confirm_modal(), invite_card() (+6 more)

### Community 52 - "Theme.kt"
Cohesion: 0.17
Nodes (3): colorScheme(), LiveScreenTheme(), StatusColors

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (25): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), lighter_is_accepted_equal_and_heavier_are_not(), LIMIT_FPS, LIMIT_PIXELS (+17 more)

### Community 55 - "format_invite"
Cohesion: 0.23
Nodes (10): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), is_valid_invite() (+2 more)

### Community 56 - "viewer.rs"
Cohesion: 0.05
Nodes (22): ViewerMsg, Hello, RequestKeyframe, SetQuality, SetVisible, BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64 (+14 more)

### Community 57 - "capture/mod.rs"
Cohesion: 0.28
Nodes (9): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+1 more)

### Community 59 - "Resources"
Cohesion: 0.18
Nodes (4): Reconf, Resources, start_encoder(), EncoderHandle

### Community 60 - "encoder.rs"
Cohesion: 0.14
Nodes (7): configure(), create_mft(), EncoderStats, read_seq_header(), spawn(), stream_ids(), vbv_bytes()

### Community 61 - "Preset"
Cohesion: 0.15
Nodes (15): EncodeParams, Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+7 more)

### Community 62 - "Sink"
Cohesion: 0.20
Nodes (5): cut_feed(), pack_size(), PendingReconf, Sink, SinkTx

### Community 63 - "omp config sync via private GitHub repo"
Cohesion: 0.17
Nodes (11): 1. Criar o script `~\.omp\sync\omp-sync.ts`, 2. Wrapper, README e caminhos portáveis, 3. Subcomando `setup` (idempotente; cada etapa pula se já estiver feita), 4. Criar o repo e fazer o primeiro push (uma vez, nesta máquina), Approach, Assumptions & contingencies, Context, Critical files & anchors (+3 more)

### Community 64 - "LiveScreen"
Cohesion: 0.20
Nodes (9): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+1 more)

### Community 65 - "HomeScreen"
Cohesion: 0.18
Nodes (14): 1. Core: estado "removido" na presença e snapshot do prober, 2. Desktop: uma identidade por janela, 3. Desktop: ordem de fechamento e abertura dos endpoints, 4. Desktop: UI/UX, 6. Android: UI/UX, Approach, AddFriendDialog(), App() (+6 more)

### Community 66 - "ViewerModel"
Cohesion: 0.08
Nodes (5): 5. Android: uma única dona do handle de presença e da identidade, Friend, Presence, Status, ViewerModel

### Community 69 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 74 - "HostMsg"
Cohesion: 0.40
Nodes (5): HostMsg, Rejected, StreamEnded, StreamParams, Welcome

### Community 77 - "start"
Cohesion: 0.67
Nodes (3): border_removable(), property_present(), start()

### Community 78 - "ui/friends.rs"
Cohesion: 0.13
Nodes (7): Draft, MAX_LIST, I_ADD_FRIEND, I_COPY, I_PEOPLE, I_PERSON, I_REMOVE

### Community 83 - "watch_view.rs"
Cohesion: 0.13
Nodes (6): I_EXIT_FULLSCREEN, I_FULLSCREEN, I_MUTE, I_PASTE, OVERLAY_TIMEOUT, WatchView

### Community 90 - ".start"
Cohesion: 0.14
Nodes (7): AudioState, Off, On, Unavailable, invite_addr(), Overload, ViewerInfo

### Community 92 - "Target"
Cohesion: 0.20
Nodes (5): AbortJoinSet<T>, Target, Friend, Invite, verify_room_proof()

## Knowledge Gaps
- **79 isolated node(s):** `livescreen`, `livescreen-android`, `Unknown`, `Live`, `Offline` (+74 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 682 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **25 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `config.rs`, `share_view.rs`, `ToastKind`, `update.rs`, `ui/friends.rs`, `watch_view.rs`, `Native`, `EncodeParams`, `watch_view.rs`, `probe.rs`, `ui/mod.rs`, `format_invite`, `viewer.rs`, `ShareSession`, `HMONITOR`?**
  _High betweenness centrality (0.072) - this node is a cross-community bridge._
- **Are the 10 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 10 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `livescreen-android`, `Unknown` to the rest of the system?**
  _79 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1337126600284495 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `ViewerModel` to `HomeScreen`, `Native`, `MainActivity.kt`, `UpdateInfo`, `VideoDecoder.kt`, `AudioPlayer`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Codec` (e.g. with `run()` and `.start()`) actually correct?**
  _`Codec` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Should `config.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08536585365853659 - nodes in this community are weakly interconnected._