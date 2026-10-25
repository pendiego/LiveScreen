# Graph Report - LiveScreen-publish  (2026-10-25)

## Corpus Check
- 59 files · ~52,916 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 16 file(s) not represented in the graph (top: (none) 5, .ttf 4, .xml 2)

## Summary
- 1556 nodes · 3181 edges · 83 communities (58 shown, 25 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 48 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2c19c06f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- encoder.rs
- capture.rs
- sharing
- ToastKind
- convert.rs
- App
- sources.rs
- e2e_window.rs
- mf.rs
- protocol.rs
- Shared
- update.rs
- Ui
- playback.rs
- theme.rs
- watch_view.rs
- gpu.rs
- share.rs
- wgc.rs
- with
- ui/mod.rs
- Theme.kt
- SourceId
- ShareSession
- .start
- encoder.rs
- steps
- LiveScreen
- Screens.kt
- Encoder
- capture/mod.rs
- test-changelog.sh
- rust/src/lib.rs
- Native
- Updater.kt
- handle_frame
- WgcCapture
- HMONITOR
- main.rs
- present.rs
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- start_encoder
- EncodeParams
- Native
- LiveScreen
- Theme.kt
- preset.rs
- Dib
- format_invite
- viewer.rs
- sharing
- write_msg
- ViewerModel
- result
- AudioPlayer.kt
- BudgetExceeded
- run
- capture/mod.rs
- HomeScreen
- Preset
- OutputResult
- MainActivity.kt
- read_video_frame
- frame_from_body
- ViewerSession
- DxgiCapture
- Callback
- HostMsg
- ViewerMsg
- WatchStatus
- start

## God Nodes (most connected - your core abstractions)
1. `App` - 59 edges
2. `Preset` - 43 edges
3. `Converter` - 33 edges
4. `Codec` - 29 edges
5. `ViewerModel` - 26 edges
6. `EncoderKind` - 26 edges
7. `SourceId` - 25 edges
8. `ShareSession` - 25 edges
9. `EncodeParams` - 23 edges
10. `Encoder` - 23 edges

## Surprising Connections (you probably didn't know these)
- `audio_loop()` --calls--> `audio_gap()`  [INFERRED]
  android/rust/src/lib.rs → crates/livescreen-core/src/protocol.rs
- `audio_loop()` --calls--> `parse_audio_datagram()`  [INFERRED]
  android/rust/src/lib.rs → crates/livescreen-core/src/protocol.rs
- `Java_io_github_pendiego_livescreen_Native_isValidTicket()` --calls--> `is_valid_invite()`  [INFERRED]
  android/rust/src/lib.rs → crates/livescreen-core/src/viewer.rs
- `supported_codecs()` --references--> `Codec`  [EXTRACTED]
  src/video/decoder.rs → crates/livescreen-core/src/protocol.rs
- `handle_viewer()` --calls--> `write_msg()`  [INFERRED]
  src/share.rs → crates/livescreen-core/src/protocol.rs

## Import Cycles
- None detected.

## Communities (83 total, 25 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.07
Nodes (39): EncodedFrame, bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H (+31 more)

### Community 1 - "encoder.rs"
Cohesion: 0.18
Nodes (7): configure(), create_mft(), EncoderStats, read_seq_header(), spawn(), stream_ids(), vbv_bytes()

### Community 2 - "capture.rs"
Cohesion: 0.05
Nodes (38): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+30 more)

### Community 3 - "sharing"
Cohesion: 0.13
Nodes (13): SourceId, Monitor, Window, SourceInfo, begin_start(), Picker, poll_starting(), Preview (+5 more)

### Community 4 - "ToastKind"
Cohesion: 0.29
Nodes (5): ToastKind, Info, Success, Warn, Toasts

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "App"
Cohesion: 0.08
Nodes (25): App, classify(), own_hwnd(), button(), diagnostics(), FORCED, hint(), icon_button() (+17 more)

### Community 9 - "e2e_window.rs"
Cohesion: 0.19
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 10 - "mf.rs"
Cohesion: 0.09
Nodes (9): enumerate_mfts(), friendly_name(), sample_survives_need_more_input(), set_codec_value(), startup(), subtype(), variant_bool(), variant_u32() (+1 more)

### Community 11 - "protocol.rs"
Cohesion: 0.10
Nodes (12): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, MAX_FRAME (+4 more)

### Community 12 - "Shared"
Cohesion: 0.17
Nodes (5): sanitize(), CountGuard, handle_viewer(), Peer, Shared

### Community 13 - "update.rs"
Cohesion: 0.11
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "Ui"
Cohesion: 0.10
Nodes (15): app_dir(), Config, load(), Overrides, overrides(), parse_overrides(), eframe::egui::ThemePreference, file_overrides_win() (+7 more)

### Community 15 - "playback.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.07
Nodes (24): app_icon(), DARK, GEIST, GEIST_MD, GEIST_MONO, GEIST_SB, I_COPY, I_MIC (+16 more)

### Community 17 - "watch_view.rs"
Cohesion: 0.08
Nodes (18): is_valid_invite(), I_BACK, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_MUTE, I_PASTE, I_VOLUME, connect() (+10 more)

### Community 19 - "share.rs"
Cohesion: 0.12
Nodes (9): Abr, ABR_FLOOR, HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), MAX_HANDSHAKES, retire() (+1 more)

### Community 21 - "with"
Cohesion: 0.07
Nodes (31): runs-on, steps, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release, name (+23 more)

### Community 23 - "Theme.kt"
Cohesion: 0.18
Nodes (15): badge(), card(), checkbox(), chip(), danger_button(), field(), filled_button(), heading() (+7 more)

### Community 24 - "SourceId"
Cohesion: 0.36
Nodes (6): fit(), source_size(), thumbnail(), thumbnail_monitor(), thumbnail_window(), window_rect()

### Community 25 - "ShareSession"
Cohesion: 0.22
Nodes (3): Regras, ShareSession, unpack_size()

### Community 26 - ".start"
Cohesion: 0.17
Nodes (7): AudioState, Off, On, Unavailable, invite_addr(), Overload, ViewerInfo

### Community 27 - "encoder.rs"
Cohesion: 0.10
Nodes (12): ended(), home(), tile(), I_CHECK, I_ERROR, I_INFO, I_SHARE, I_UPDATE (+4 more)

### Community 28 - "steps"
Cohesion: 0.10
Nodes (23): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+15 more)

### Community 29 - "LiveScreen"
Cohesion: 0.17
Nodes (11): Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença, LiveScreen (+3 more)

### Community 30 - "Screens.kt"
Cohesion: 0.12
Nodes (16): Codec, Av1, H264, Hevc, pick_encoder(), EncoderKind, Hardware, Software (+8 more)

### Community 31 - "Encoder"
Cohesion: 0.19
Nodes (6): CpuCopy, Encoder, Feed, Idle, Stop, Nv12Frame

### Community 32 - "capture/mod.rs"
Cohesion: 0.15
Nodes (4): CountSink, FirstFrameSink, FrameSink, SinkHandle

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.09
Nodes (22): AbortOnDrop, __android_log_write(), audio_loop(), get_session(), init_logging(), INIT_LOGGING, Java_io_github_pendiego_livescreen_Native_decoderFailed(), Java_io_github_pendiego_livescreen_Native_isValidTicket() (+14 more)

### Community 35 - "Native"
Cohesion: 0.08
Nodes (3): AudioPlayer, Native, VideoDecoder

### Community 36 - "Updater.kt"
Cohesion: 0.08
Nodes (10): GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle, Ready (+2 more)

### Community 39 - "HMONITOR"
Cohesion: 0.40
Nodes (7): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate()

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.06
Nodes (9): Mmcss, Display, GpuFrame, Imported, NEXT_SURFACES_ID, Producer, SLOTS, Surfaces (+1 more)

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "start_encoder"
Cohesion: 0.20
Nodes (5): cut_feed(), pack_size(), PendingReconf, Sink, SinkTx

### Community 49 - "EncodeParams"
Cohesion: 0.20
Nodes (4): Reconf, Resources, start_encoder(), EncoderHandle

### Community 50 - "Native"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 51 - "LiveScreen"
Cohesion: 0.29
Nodes (6): Comandos, Estilo, graphify, LiveScreen, Mapa do código, Release

### Community 52 - "Theme.kt"
Cohesion: 0.06
Nodes (4): LiveScreenTheme(), Palette, scheme(), StatusBadge()

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (27): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), is_reduction(), label(), lighter_is_accepted_equal_and_heavier_are_not() (+19 more)

### Community 55 - "format_invite"
Cohesion: 0.25
Nodes (6): format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), verify_room_proof()

### Community 56 - "viewer.rs"
Cohesion: 0.11
Nodes (11): BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64, CLOSE_KICKED_U64, CLOSE_PROTOCOL_U64, close_reason(), CLOSE_STOPPED_U64, DecodeStats (+3 more)

### Community 57 - "sharing"
Cohesion: 0.19
Nodes (19): viewers_for(), ShareStatus, Upload, Done, Failed, Measuring, Unknown, bar_frame() (+11 more)

### Community 58 - "write_msg"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 61 - "AudioPlayer.kt"
Cohesion: 0.50
Nodes (3): exe_name(), inspect_window(), window_cb()

### Community 63 - "run"
Cohesion: 0.26
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 64 - "capture/mod.rs"
Cohesion: 0.28
Nodes (9): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+1 more)

### Community 65 - "HomeScreen"
Cohesion: 0.31
Nodes (9): App(), HomeScreen(), UpdateBanner(), WatchContent(), WatchScreen(), fieldColors(), PrimaryButton(), SecondaryButton() (+1 more)

### Community 66 - "Preset"
Cohesion: 0.17
Nodes (11): Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60, P2160p60 (+3 more)

### Community 67 - "OutputResult"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 69 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 70 - "frame_from_body"
Cohesion: 0.29
Nodes (4): frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 74 - "HostMsg"
Cohesion: 0.40
Nodes (5): HostMsg, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "ViewerMsg"
Cohesion: 0.40
Nodes (5): ViewerMsg, Hello, RequestKeyframe, SetQuality, SetVisible

### Community 77 - "WatchStatus"
Cohesion: 0.50
Nodes (4): Phase, Connecting, Playing, WatchStatus

### Community 78 - "start"
Cohesion: 0.67
Nodes (3): border_removable(), property_present(), start()

## Knowledge Gaps
- **73 isolated node(s):** `livescreen`, `livescreen-android`, `P480p30`, `P720p30`, `P720p60` (+68 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 569 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **25 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `sharing`, `ToastKind`, `update.rs`, `Ui`, `watch_view.rs`, `Native`, `ui/mod.rs`, `ShareSession`, `encoder.rs`, `result`, `Screens.kt`, `sharing`?**
  _High betweenness centrality (0.075) - this node is a cross-community bridge._
- **What connects `livescreen`, `livescreen-android`, `P480p30` to the rest of the system?**
  _73 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07163461538461538 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `ViewerModel` to `HomeScreen`, `Native`, `MainActivity.kt`, `Updater.kt`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._
- **Should `capture.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.051643192488262914 - nodes in this community are weakly interconnected._
- **Why does `SourceId` connect `sharing` to `capture/mod.rs`, `App`, `HMONITOR`, `sources.rs`, `dxgi.rs`, `e2e_window.rs`, `start`, `share.rs`, `wgc.rs`, `ui/mod.rs`, `SourceId`, `sharing`, `run`?**
  _High betweenness centrality (0.043) - this node is a cross-community bridge._
- **Should `sharing` be split into smaller, more focused modules?**
  _Cohesion score 0.12873563218390804 - nodes in this community are weakly interconnected._