# Graph Report - LiveScreen  (2026-10-27)

## Corpus Check
- 59 files · ~84,631 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 44 file(s) not represented in the graph (top: .xml 21, .ttf 7, .woff2 6)

## Summary
- 1825 nodes · 3752 edges · 94 communities (65 shown, 29 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 68 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c5672128`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- config.rs
- playback.rs
- sharing
- codec_roundtrip.rs
- convert.rs
- App
- sources.rs
- speedtest.rs
- mf.rs
- protocol.rs
- Resources
- update.rs
- ViewerModel
- encoder.rs
- theme.rs
- home.rs
- gpu.rs
- frame_from_body
- with
- ui/mod.rs
- Ui
- HWND
- Preset
- HMONITOR
- share.rs
- with
- LiveScreen
- Codec
- capture.rs
- SinkHandle
- test-changelog.sh
- rust/src/lib.rs
- Native
- src/friends.rs
- UpdateInfo
- wgc.rs
- run
- main.rs
- present.rs
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- write_msg
- Target
- Screen
- LiveScreen
- Theme.kt
- preset.rs
- Dib
- parse_invite
- viewer.rs
- capture/mod.rs
- Mic
- EncodeParams
- capture_loop
- AudioPlayer
- Callback
- SinalField
- AbortJoinSet<T>
- HomeScreen
- ViewerModel.kt
- BudgetExceeded
- MainActivity.kt
- read_video_frame
- VideoDecoder
- handle_frame
- Picker
- AudioHandle
- HostMsg
- BtnKind
- ToastKind
- share_view.rs
- watch_view.rs
- CallState
- SinalSnack
- inspect_window
- WgcCapture
- paint_surface
- CallTone
- .start
- DxgiCapture
- AudioTarget
- start

## God Nodes (most connected - your core abstractions)
1. `App` - 73 edges
2. `Preset` - 49 edges
3. `ViewerModel` - 41 edges
4. `Converter` - 33 edges
5. `ShareSession` - 32 edges
6. `Codec` - 29 edges
7. `EncoderKind` - 26 edges
8. `SourceId` - 25 edges
9. `EncodeParams` - 24 edges
10. `Native` - 23 edges

## Surprising Connections (you probably didn't know these)
- `Android (`android/`)` --references--> `AudioPlayer`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/AudioPlayer.kt
- `Android (`android/`)` --references--> `MainActivity`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/MainActivity.kt
- `Android (`android/`)` --references--> `Native`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/Native.kt
- `Android (`android/`)` --references--> `VideoDecoder`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/VideoDecoder.kt
- `Android (`android/`)` --references--> `ViewerModel`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/ViewerModel.kt

## Import Cycles
- None detected.

## Communities (94 total, 29 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.14
Nodes (19): as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size(), Gpu (+11 more)

### Community 1 - "config.rs"
Cohesion: 0.10
Nodes (17): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+9 more)

### Community 2 - "playback.rs"
Cohesion: 0.12
Nodes (10): conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, output_loop(), PlaybackHandle, PREBUFFER_FLOATS (+2 more)

### Community 3 - "sharing"
Cohesion: 0.17
Nodes (20): EncodeParams, viewers_for(), ShareStatus, Upload, Done, Failed, Measuring, Unknown (+12 more)

### Community 4 - "codec_roundtrip.rs"
Cohesion: 0.14
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "App"
Cohesion: 0.06
Nodes (32): App, classify(), own_hwnd(), button(), diagnostics(), FORCED, hint(), icon_button() (+24 more)

### Community 9 - "speedtest.rs"
Cohesion: 0.06
Nodes (30): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+22 more)

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (18): enumerate_mfts(), friendly_name(), has_param_sets(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange (+10 more)

### Community 11 - "protocol.rs"
Cohesion: 0.09
Nodes (13): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, LEGACY_PROTOCOL_VERSION (+5 more)

### Community 12 - "Resources"
Cohesion: 0.13
Nodes (6): cut_feed(), Reconf, Resources, Sink, start_encoder(), EncoderHandle

### Community 13 - "update.rs"
Cohesion: 0.09
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "ViewerModel"
Cohesion: 0.19
Nodes (3): Friend, Status, ViewerModel

### Community 15 - "encoder.rs"
Cohesion: 0.13
Nodes (8): EncodedFrame, configure(), create_mft(), EncoderStats, read_seq_header(), spawn(), stream_ids(), vbv_bytes()

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (31): app_icon(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), card(), field(), I_CHEVRON_DOWN (+23 more)

### Community 17 - "home.rs"
Cohesion: 0.10
Nodes (16): ended(), footer(), home(), left_column(), share_tile(), update_prompt(), I_CHEVRON, I_CIRCLE_CHECK (+8 more)

### Community 20 - "frame_from_body"
Cohesion: 0.29
Nodes (4): frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 21 - "with"
Cohesion: 0.07
Nodes (32): runs-on, steps, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release, name (+24 more)

### Community 23 - "Ui"
Cohesion: 0.15
Nodes (24): badge(), button(), button_full(), call_button(), checkbox(), draw_button(), ghost_icon_button(), hangup_button() (+16 more)

### Community 24 - "HWND"
Cohesion: 0.36
Nodes (6): fit(), source_size(), thumbnail(), thumbnail_monitor(), thumbnail_window(), window_rect()

### Community 25 - "Preset"
Cohesion: 0.09
Nodes (17): is_reduction(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+9 more)

### Community 26 - "HMONITOR"
Cohesion: 0.40
Nodes (7): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate()

### Community 27 - "share.rs"
Cohesion: 0.08
Nodes (16): sanitize(), Abr, ABR_FLOOR, admit(), answer_presence(), CountGuard, handle_viewer(), HANDSHAKE_TIMEOUT (+8 more)

### Community 28 - "with"
Cohesion: 0.10
Nodes (23): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+15 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (só para assistir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "Codec"
Cohesion: 0.12
Nodes (16): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+8 more)

### Community 31 - "capture.rs"
Cohesion: 0.12
Nodes (11): FRAME_BYTES, FRAME_FLOATS, FRAME_SAMPLES, MAX_LAG_BYTES, mix(), mix_clamps(), OPUS_MAX_PACKET, take_frame() (+3 more)

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
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 37 - "UpdateInfo"
Cohesion: 0.17
Nodes (11): Android (`android/`), GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle (+3 more)

### Community 39 - "run"
Cohesion: 0.27
Nodes (3): Composer, GrabSink, texture()

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.06
Nodes (9): Mmcss, Display, GpuFrame, Imported, NEXT_SURFACES_ID, Producer, SLOTS, Surfaces (+1 more)

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "write_msg"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 49 - "Target"
Cohesion: 0.29
Nodes (4): Target, Friend, Invite, verify_room_proof()

### Community 50 - "Screen"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 51 - "LiveScreen"
Cohesion: 0.20
Nodes (9): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+1 more)

### Community 52 - "Theme.kt"
Cohesion: 0.18
Nodes (3): inter(), LiveScreenTheme(), Sinal

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (26): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not(), LIMIT_FPS (+18 more)

### Community 55 - "parse_invite"
Cohesion: 0.23
Nodes (10): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), is_valid_invite() (+2 more)

### Community 56 - "viewer.rs"
Cohesion: 0.05
Nodes (21): ViewerMsg, Hello, RequestKeyframe, SetQuality, SetVisible, BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64 (+13 more)

### Community 57 - "capture/mod.rs"
Cohesion: 0.17
Nodes (15): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, CountSink, run_backend() (+7 more)

### Community 58 - "Mic"
Cohesion: 0.22
Nodes (7): Mic, Default, Device, open(), open_mic(), pump(), Source

### Community 59 - "EncodeParams"
Cohesion: 0.18
Nodes (7): CpuCopy, Encoder, Feed, Frame, Idle, Stop, Nv12Frame

### Community 60 - "capture_loop"
Cohesion: 0.50
Nodes (6): AudioRequest, capture_loop(), mic_error_text(), Opened, run(), start()

### Community 63 - "SinalField"
Cohesion: 0.25
Nodes (5): FieldTone, Error, Normal, Warn, SinalField()

### Community 65 - "HomeScreen"
Cohesion: 0.30
Nodes (12): AddFriendDialog(), App(), Badge(), CallButton(), clipboardText(), FriendsSection(), HomeScreen(), SinalButton() (+4 more)

### Community 69 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 72 - "Picker"
Cohesion: 0.17
Nodes (6): begin_start(), Picker, poll_starting(), Preview, Starting, to_image()

### Community 74 - "HostMsg"
Cohesion: 0.40
Nodes (5): HostMsg, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 77 - "ToastKind"
Cohesion: 0.25
Nodes (6): spawn_share_job(), ToastKind, Info, Success, Warn, Toasts

### Community 78 - "share_view.rs"
Cohesion: 0.07
Nodes (18): Draft, MAX_LIST, AUTO_QUALITY_COOLDOWN, MAX_ROOM, I_ADD_FRIEND, I_BACK, I_CHECK, I_COPY (+10 more)

### Community 83 - "watch_view.rs"
Cohesion: 0.11
Nodes (11): I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY, I_SETTINGS (+3 more)

### Community 84 - "CallState"
Cohesion: 0.40
Nodes (5): CallState, Active, Alert, HangUp, Neutral

### Community 85 - "SinalSnack"
Cohesion: 0.40
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 86 - "inspect_window"
Cohesion: 0.50
Nodes (3): exe_name(), inspect_window(), window_cb()

### Community 89 - "CallTone"
Cohesion: 0.40
Nodes (5): CallTone, Active, Alert, Neutral, Plain

### Community 90 - ".start"
Cohesion: 0.12
Nodes (8): AudioState, Off, On, Unavailable, invite_addr(), Overload, pack_size(), ViewerInfo

### Community 91 - "DxgiCapture"
Cohesion: 0.29
Nodes (6): create_duplication(), dxgi_draws_cursor(), DxgiCapture, run(), start(), SinkHandle

### Community 92 - "AudioTarget"
Cohesion: 0.50
Nodes (4): AudioTarget, Process, System, open_loopback()

### Community 93 - "start"
Cohesion: 0.67
Nodes (3): border_removable(), property_present(), start()

## Knowledge Gaps
- **99 isolated node(s):** `livescreen`, `Primary`, `Secondary`, `Ghost`, `Danger` (+94 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 690 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **29 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `config.rs`, `sharing`, `src/friends.rs`, `Picker`, `ToastKind`, `share_view.rs`, `update.rs`, `home.rs`, `Screen`, `watch_view.rs`, `parse_invite`, `ui/mod.rs`, `Codec`, `viewer.rs`, `capture/mod.rs`, `Preset`?**
  _High betweenness centrality (0.073) - this node is a cross-community bridge._
- **What connects `livescreen`, `Primary`, `Secondary` to the rest of the system?**
  _99 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13513513513513514 - nodes in this community are weakly interconnected._
- **Why does `SourceId` connect `capture/mod.rs` to `sharing`, `wgc.rs`, `dxgi.rs`, `sources.rs`, `speedtest.rs`, `share.rs`, `App`, `Picker`, `share_view.rs`, `ui/mod.rs`, `HWND`, `HMONITOR`, `DxgiCapture`, `start`?**
  _High betweenness centrality (0.036) - this node is a cross-community bridge._
- **Should `config.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0960960960960961 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `ViewerModel` to `HomeScreen`, `ViewerModel.kt`, `Native`, `MainActivity.kt`, `UpdateInfo`, `VideoDecoder`, `AudioPlayer`?**
  _High betweenness centrality (0.036) - this node is a cross-community bridge._
- **Should `playback.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1225071225071225 - nodes in this community are weakly interconnected._