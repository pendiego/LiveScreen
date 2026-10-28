# Graph Report - LiveScreen  (2026-10-27)

## Corpus Check
- 59 files · ~83,742 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 43 file(s) not represented in the graph (top: .xml 20, .ttf 7, .woff2 6)

## Summary
- 1814 nodes · 3719 edges · 86 communities (61 shown, 25 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 69 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `b364c22c`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- config.rs
- capture.rs
- sharing
- codec_roundtrip.rs
- convert.rs
- App
- sources.rs
- e2e_window.rs
- mf.rs
- protocol.rs
- Resources
- update.rs
- ViewerModel
- OutputResult
- theme.rs
- home.rs
- gpu.rs
- Codec
- with
- ui/mod.rs
- Ui
- HWND
- Preset
- HMONITOR
- share.rs
- with
- LiveScreen
- encoder.rs
- enumerate_mfts
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
- variant_with
- EncodeParams
- Mmcss
- AudioPlayer
- Callback
- room_proof
- AbortJoinSet<T>
- HomeScreen
- ViewerModel.kt
- BudgetExceeded
- MainActivity.kt
- read_video_frame
- VideoDecoder
- Picker
- HostMsg
- ToastKind
- share_view.rs
- watch_view.rs
- CallTone
- inspect_window
- paint_surface
- .start
- DxgiCapture

## God Nodes (most connected - your core abstractions)
1. `App` - 73 edges
2. `Preset` - 49 edges
3. `ViewerModel` - 42 edges
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

## Communities (86 total, 25 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.11
Nodes (19): as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size(), Gpu (+11 more)

### Community 1 - "config.rs"
Cohesion: 0.05
Nodes (38): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+30 more)

### Community 2 - "capture.rs"
Cohesion: 0.05
Nodes (40): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+32 more)

### Community 3 - "sharing"
Cohesion: 0.18
Nodes (18): viewers_for(), ShareStatus, Upload, Done, Failed, Measuring, Unknown, Choices (+10 more)

### Community 4 - "codec_roundtrip.rs"
Cohesion: 0.13
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "App"
Cohesion: 0.05
Nodes (38): ended(), footer(), home(), left_column(), share_tile(), update_prompt(), App, classify() (+30 more)

### Community 9 - "e2e_window.rs"
Cohesion: 0.18
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 11 - "protocol.rs"
Cohesion: 0.09
Nodes (13): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, LEGACY_PROTOCOL_VERSION (+5 more)

### Community 12 - "Resources"
Cohesion: 0.16
Nodes (4): cut_feed(), Resources, Sink, EncoderHandle

### Community 13 - "update.rs"
Cohesion: 0.10
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "ViewerModel"
Cohesion: 0.19
Nodes (3): Friend, Status, ViewerModel

### Community 15 - "OutputResult"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (33): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), card(), field() (+25 more)

### Community 17 - "home.rs"
Cohesion: 0.11
Nodes (10): I_CHEVRON, I_CIRCLE_CHECK, I_ERROR, I_INFO, I_SHARE, I_UPDATE, I_WARN, I_WATCH (+2 more)

### Community 18 - "gpu.rs"
Cohesion: 0.09
Nodes (3): create_device(), Sendable, winrt_device()

### Community 20 - "Codec"
Cohesion: 0.18
Nodes (11): Codec, Av1, H264, Hevc, EncodedFrame, frame_from_body(), parse_video_body(), roundtrip() (+3 more)

### Community 21 - "with"
Cohesion: 0.07
Nodes (32): runs-on, steps, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release, name (+24 more)

### Community 23 - "Ui"
Cohesion: 0.12
Nodes (27): button(), button_full(), call_button(), CallTone, Active, Alert, Neutral, Plain (+19 more)

### Community 24 - "HWND"
Cohesion: 0.36
Nodes (6): fit(), source_size(), thumbnail(), thumbnail_monitor(), thumbnail_window(), window_rect()

### Community 25 - "Preset"
Cohesion: 0.08
Nodes (18): is_reduction(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+10 more)

### Community 26 - "HMONITOR"
Cohesion: 0.40
Nodes (7): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate()

### Community 27 - "share.rs"
Cohesion: 0.07
Nodes (18): sanitize(), Abr, ABR_FLOOR, admit(), answer_presence(), CountGuard, handle_viewer(), HANDSHAKE_TIMEOUT (+10 more)

### Community 28 - "with"
Cohesion: 0.10
Nodes (23): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+15 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (só para assistir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "encoder.rs"
Cohesion: 0.11
Nodes (11): EncoderKind, Hardware, Software, EncoderStats, black_nv12(), candidates(), detect(), hardware_kinds() (+3 more)

### Community 31 - "enumerate_mfts"
Cohesion: 0.22
Nodes (4): enumerate_mfts(), friendly_name(), set_codec_value(), subtype()

### Community 32 - "SinkHandle"
Cohesion: 0.15
Nodes (3): CountSink, FirstFrameSink, FrameSink

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

### Community 38 - "wgc.rs"
Cohesion: 0.11
Nodes (7): border_removable(), handle_frame(), PIXEL_FORMAT, property_present(), SESSION_TYPE, start(), WgcCapture

### Community 39 - "run"
Cohesion: 0.27
Nodes (3): Composer, GrabSink, texture()

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.07
Nodes (8): Display, GpuFrame, Imported, NEXT_SURFACES_ID, Producer, SLOTS, Surfaces, Target

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "write_msg"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 49 - "Target"
Cohesion: 0.40
Nodes (3): Target, Friend, Invite

### Community 50 - "Screen"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 51 - "LiveScreen"
Cohesion: 0.20
Nodes (9): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+1 more)

### Community 52 - "Theme.kt"
Cohesion: 0.17
Nodes (3): colorScheme(), LiveScreenTheme(), StatusColors

### Community 53 - "preset.rs"
Cohesion: 0.11
Nodes (28): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not() (+20 more)

### Community 55 - "parse_invite"
Cohesion: 0.27
Nodes (8): find_invite(), find_invite_in_pasted_text(), format_invite(), new_room_key(), parse_invite(), is_valid_invite(), input(), normalize_ticket()

### Community 56 - "viewer.rs"
Cohesion: 0.05
Nodes (21): ViewerMsg, Hello, RequestKeyframe, SetQuality, SetVisible, BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64 (+13 more)

### Community 57 - "capture/mod.rs"
Cohesion: 0.22
Nodes (12): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+4 more)

### Community 58 - "variant_with"
Cohesion: 0.47
Nodes (3): variant_bool(), variant_u32(), variant_with()

### Community 59 - "EncodeParams"
Cohesion: 0.12
Nodes (13): configure(), CpuCopy, create_mft(), Encoder, Feed, Frame, Idle, Stop (+5 more)

### Community 63 - "room_proof"
Cohesion: 0.50
Nodes (3): invite_roundtrip_and_proof_binding(), room_proof(), verify_room_proof()

### Community 65 - "HomeScreen"
Cohesion: 0.28
Nodes (10): AddFriendDialog(), App(), CallButton(), CallToggle(), clipboardText(), FriendsSection(), HomeScreen(), UpdateBanner() (+2 more)

### Community 69 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 72 - "Picker"
Cohesion: 0.16
Nodes (8): SourceInfo, begin_start(), Picker, poll_starting(), Preview, Starting, to_image(), Update

### Community 74 - "HostMsg"
Cohesion: 0.40
Nodes (5): HostMsg, Rejected, StreamEnded, StreamParams, Welcome

### Community 77 - "ToastKind"
Cohesion: 0.25
Nodes (6): spawn_share_job(), ToastKind, Info, Success, Warn, Toasts

### Community 78 - "share_view.rs"
Cohesion: 0.07
Nodes (18): Draft, MAX_LIST, AUTO_QUALITY_COOLDOWN, MAX_ROOM, I_ADD_FRIEND, I_BACK, I_CHECK, I_COPY (+10 more)

### Community 83 - "watch_view.rs"
Cohesion: 0.11
Nodes (11): I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY, I_SETTINGS (+3 more)

### Community 85 - "CallTone"
Cohesion: 0.50
Nodes (4): CallTone, Active, Alert, Neutral

### Community 86 - "inspect_window"
Cohesion: 0.50
Nodes (3): exe_name(), inspect_window(), window_cb()

### Community 90 - ".start"
Cohesion: 0.12
Nodes (8): AudioState, Off, On, Unavailable, invite_addr(), Overload, pack_size(), ViewerInfo

### Community 91 - "DxgiCapture"
Cohesion: 0.26
Nodes (6): create_duplication(), dxgi_draws_cursor(), DxgiCapture, run(), start(), SinkHandle

## Knowledge Gaps
- **89 isolated node(s):** `livescreen`, `Neutral`, `Active`, `Alert`, `livescreen-android` (+84 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 689 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **25 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `config.rs`, `sharing`, `src/friends.rs`, `Picker`, `ToastKind`, `share_view.rs`, `update.rs`, `home.rs`, `Screen`, `watch_view.rs`, `parse_invite`, `ui/mod.rs`, `encoder.rs`, `viewer.rs`, `capture/mod.rs`, `Preset`?**
  _High betweenness centrality (0.070) - this node is a cross-community bridge._
- **What connects `livescreen`, `Neutral`, `Active` to the rest of the system?**
  _89 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11463414634146342 - nodes in this community are weakly interconnected._
- **Why does `SourceId` connect `capture/mod.rs` to `sharing`, `wgc.rs`, `dxgi.rs`, `sources.rs`, `Picker`, `e2e_window.rs`, `share.rs`, `App`, `share_view.rs`, `ui/mod.rs`, `HWND`, `HMONITOR`, `DxgiCapture`?**
  _High betweenness centrality (0.041) - this node is a cross-community bridge._
- **Should `config.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05048076923076923 - nodes in this community are weakly interconnected._
- **Why does `Converter` connect `convert.rs` to `SinkHandle`, `decoder.rs`, `codec_roundtrip.rs`, `present.rs`, `Resources`, `share.rs`?**
  _High betweenness centrality (0.039) - this node is a cross-community bridge._
- **Should `capture.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.050351721584598295 - nodes in this community are weakly interconnected._