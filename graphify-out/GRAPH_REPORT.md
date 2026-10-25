# Graph Report - LiveScreen  (2026-10-25)

## Corpus Check
- 40 files · ~40,553 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 9 file(s) not represented in the graph (top: (none) 4, .ttf 4, .toml 1)

## Summary
- 1156 nodes · 2404 edges · 64 communities (47 shown, 17 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 25 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `74b3f968`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- codec_roundtrip.rs
- decoder.rs
- capture.rs
- Preset
- ToastKind
- convert.rs
- App
- dxgi.rs
- sources.rs
- watch.rs
- mf.rs
- protocol.rs
- Shared
- update.rs
- Overrides
- speedtest.rs
- theme.rs
- watch_view.rs
- gpu.rs
- share.rs
- wgc.rs
- workflows/release.yml
- ui/mod.rs
- Ui
- HWND
- ShareSession
- AudioState
- home.rs
- ci.yml
- LiveScreen
- encoder.rs
- encoder.rs
- capture/mod.rs
- test-changelog.sh
- paint_surface
- playback.rs
- field
- handle_frame
- WgcCapture
- HMONITOR
- main.rs
- Mmcss
- settings.rs
- icon.rs
- livescreen
- Sendable<IDirect3DDevice>
- Sink
- Resources
- Screen
- Preset
- Codec
- read_video_frame
- Dib
- format_invite
- controls
- Tab
- write_msg
- HostMsg
- ViewerMsg
- inspect_window
- BudgetExceeded
- WatchView

## God Nodes (most connected - your core abstractions)
1. `App` - 57 edges
2. `Preset` - 45 edges
3. `SourceId` - 25 edges
4. `Converter` - 24 edges
5. `ShareSession` - 23 edges
6. `EncodeParams` - 22 edges
7. `Encoder` - 22 edges
8. `encode_params()` - 21 edges
9. `Shared` - 20 edges
10. `EncoderKind` - 20 edges

## Surprising Connections (you probably didn't know these)
- `handle_viewer()` --calls--> `write_msg()`  [INFERRED]
  src/share.rs → src/protocol.rs
- `run()` --calls--> `write_msg()`  [INFERRED]
  src/watch.rs → src/protocol.rs
- `handle_viewer()` --calls--> `read_msg()`  [INFERRED]
  src/share.rs → src/protocol.rs
- `run()` --calls--> `read_msg()`  [INFERRED]
  src/watch.rs → src/protocol.rs
- `run()` --calls--> `room_proof()`  [INFERRED]
  src/watch.rs → src/protocol.rs

## Import Cycles
- 1-file cycle: `src/share.rs -> src/share.rs`

## Communities (64 total, 17 thin omitted)

### Community 0 - "codec_roundtrip.rs"
Cohesion: 0.12
Nodes (19): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, hardware_kind() (+11 more)

### Community 1 - "decoder.rs"
Cohesion: 0.16
Nodes (15): EncodedFrame, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), DecodeStats, drain() (+7 more)

### Community 2 - "capture.rs"
Cohesion: 0.08
Nodes (30): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+22 more)

### Community 3 - "Preset"
Cohesion: 0.10
Nodes (32): SourceId, Monitor, Window, SourceInfo, viewers_for(), ShareStatus, Upload, Done (+24 more)

### Community 4 - "ToastKind"
Cohesion: 0.29
Nodes (5): ToastKind, Info, Success, Warn, Toasts

### Community 5 - "convert.rs"
Cohesion: 0.06
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 7 - "dxgi.rs"
Cohesion: 0.08
Nodes (9): Composer, create_duplication(), dxgi_draws_cursor(), DxgiCapture, Grab, GrabSink, run(), start() (+1 more)

### Community 9 - "watch.rs"
Cohesion: 0.07
Nodes (18): ChildGuard, e2e_window(), find(), PY, within(), _assert_conn(), CLOSE_DENIED_U64, CLOSE_FULL_U64 (+10 more)

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (16): enumerate_mfts(), friendly_name(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample (+8 more)

### Community 11 - "protocol.rs"
Cohesion: 0.11
Nodes (12): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, MAX_FRAME (+4 more)

### Community 12 - "Shared"
Cohesion: 0.19
Nodes (4): CountGuard, handle_viewer(), sanitize(), Shared

### Community 13 - "update.rs"
Cohesion: 0.10
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "Overrides"
Cohesion: 0.11
Nodes (15): app_dir(), Config, load(), Overrides, overrides(), parse_overrides(), eframe::egui::ThemePreference, file_overrides_win() (+7 more)

### Community 15 - "speedtest.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.06
Nodes (24): app_icon(), DARK, GEIST, GEIST_MD, GEIST_MONO, GEIST_SB, I_COPY, I_MIC (+16 more)

### Community 17 - "watch_view.rs"
Cohesion: 0.18
Nodes (7): I_BACK, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_MUTE, I_PASTE, I_VOLUME, OVERLAY_TIMEOUT

### Community 19 - "share.rs"
Cohesion: 0.11
Nodes (10): Abr, ABR_FLOOR, HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), MAX_HANDSHAKES, retire() (+2 more)

### Community 20 - "wgc.rs"
Cohesion: 0.13
Nodes (5): border_removable(), PIXEL_FORMAT, property_present(), SESSION_TYPE, start()

### Community 21 - "workflows/release.yml"
Cohesion: 0.11
Nodes (17): jobs, release, name, permissions, contents, tags, runs-on, steps (+9 more)

### Community 23 - "Ui"
Cohesion: 0.24
Nodes (13): badge(), card(), checkbox(), chip(), danger_button(), filled_button(), heading(), icon_button() (+5 more)

### Community 24 - "HWND"
Cohesion: 0.43
Nodes (6): fit(), source_size(), thumbnail(), thumbnail_monitor(), thumbnail_window(), window_rect()

### Community 25 - "ShareSession"
Cohesion: 0.17
Nodes (4): invite_addr(), Overload, ShareSession, unpack_size()

### Community 26 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

### Community 27 - "home.rs"
Cohesion: 0.11
Nodes (12): ended(), home(), tile(), I_CHECK, I_ERROR, I_INFO, I_SHARE, I_UPDATE (+4 more)

### Community 28 - "ci.yml"
Cohesion: 0.13
Nodes (16): runs-on, steps, runs-on, steps, jobs, build, changelog, name (+8 more)

### Community 29 - "LiveScreen"
Cohesion: 0.17
Nodes (11): Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença, LiveScreen (+3 more)

### Community 30 - "encoder.rs"
Cohesion: 0.12
Nodes (9): EncoderKind, Hardware, Software, EncoderStats, black_nv12(), detect(), probe_child(), run_child() (+1 more)

### Community 31 - "encoder.rs"
Cohesion: 0.12
Nodes (13): configure(), CpuCopy, create_mft(), Encoder, Feed, Frame, Idle, Stop (+5 more)

### Community 32 - "capture/mod.rs"
Cohesion: 0.11
Nodes (13): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, CountSink, FirstFrameSink (+5 more)

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 35 - "playback.rs"
Cohesion: 0.14
Nodes (10): DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, opus_packet_roundtrip(), output_loop(), parse_packet(), PlaybackHandle, PREBUFFER_FLOATS (+2 more)

### Community 39 - "HMONITOR"
Cohesion: 0.35
Nodes (7): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate()

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 42 - "settings.rs"
Cohesion: 0.28
Nodes (15): button(), diagnostics(), FORCED, hint(), icon_button(), look(), preset_name(), share() (+7 more)

### Community 48 - "Sink"
Cohesion: 0.22
Nodes (5): cut_feed(), pack_size(), PendingReconf, Sink, SinkTx

### Community 49 - "Resources"
Cohesion: 0.18
Nodes (4): Reconf, Resources, start_encoder(), EncoderHandle

### Community 50 - "Screen"
Cohesion: 0.15
Nodes (12): classify(), EndKind, Error, Info, Warn, Screen, Ended, Home (+4 more)

### Community 51 - "Preset"
Cohesion: 0.07
Nodes (45): Comandos, Estilo, graphify, LiveScreen, Mapa do código, Regras, Release, bitrate() (+37 more)

### Community 52 - "Codec"
Cohesion: 0.18
Nodes (8): Codec, Av1, H264, Hevc, frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 53 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 55 - "format_invite"
Cohesion: 0.22
Nodes (7): format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), verify_room_proof(), is_valid_invite()

### Community 56 - "controls"
Cohesion: 0.22
Nodes (8): connect(), ControlAction, Leave, None, ToggleFullscreen, controls(), input(), watching()

### Community 57 - "Tab"
Cohesion: 0.29
Nodes (7): Tab, Diagnostics, Look, Share, Updates, Video, TABS

### Community 58 - "write_msg"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 59 - "HostMsg"
Cohesion: 0.40
Nodes (5): HostMsg, Rejected, StreamEnded, StreamParams, Welcome

### Community 60 - "ViewerMsg"
Cohesion: 0.40
Nodes (5): ViewerMsg, Hello, RequestKeyframe, SetQuality, SetVisible

### Community 61 - "inspect_window"
Cohesion: 0.50
Nodes (3): exe_name(), inspect_window(), window_cb()

## Knowledge Gaps
- **70 isolated node(s):** `livescreen`, `Process`, `System`, `Default`, `Device` (+65 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 423 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **17 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `Preset`, `ToastKind`, `watch.rs`, `settings.rs`, `update.rs`, `Overrides`, `watch_view.rs`, `Screen`, `ui/mod.rs`, `controls`, `ShareSession`, `home.rs`, `WatchView`, `encoder.rs`, `Tab`?**
  _High betweenness centrality (0.085) - this node is a cross-community bridge._
- **What connects `livescreen`, `Process`, `System` to the rest of the system?**
  _70 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `codec_roundtrip.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12315270935960591 - nodes in this community are weakly interconnected._
- **Why does `Preset` connect `Preset` to `codec_roundtrip.rs`, `Preset`, `watch.rs`, `settings.rs`, `Shared`, `Overrides`, `Resources`, `share.rs`, `ShareSession`, `ViewerMsg`?**
  _High betweenness centrality (0.082) - this node is a cross-community bridge._
- **Should `capture.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0782312925170068 - nodes in this community are weakly interconnected._
- **Why does `SourceId` connect `Preset` to `capture/mod.rs`, `App`, `HMONITOR`, `sources.rs`, `dxgi.rs`, `watch.rs`, `share.rs`, `wgc.rs`, `ui/mod.rs`, `HWND`?**
  _High betweenness centrality (0.082) - this node is a cross-community bridge._
- **Should `Preset` be split into smaller, more focused modules?**
  _Cohesion score 0.0962962962962963 - nodes in this community are weakly interconnected._