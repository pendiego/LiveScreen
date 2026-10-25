# Graph Report - LiveScreen  (2026-10-24)

## Corpus Check
- 40 files · ~39,057 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 8 file(s) not represented in the graph (top: (none) 4, .ttf 4)

## Summary
- 1120 nodes · 2334 edges · 50 communities (37 shown, 13 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 16 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ac5d1137`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- share_view.rs
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
- playback.rs
- encoder.rs
- SinkHandle
- Screen
- paint_surface
- changelog
- handle_frame
- WgcCapture
- enumerate
- main.rs
- Mmcss
- icon.rs
- livescreen
- Sendable<IDirect3DDevice>
- Sink
- Resources
- LiveScreen
- Dib

## God Nodes (most connected - your core abstractions)
1. `App` - 57 edges
2. `Preset` - 45 edges
3. `SourceId` - 25 edges
4. `Converter` - 24 edges
5. `EncodeParams` - 22 edges
6. `ShareSession` - 22 edges
7. `Encoder` - 22 edges
8. `encode_params()` - 21 edges
9. `EncoderKind` - 20 edges
10. `WatchSession` - 20 edges

## Surprising Connections (you probably didn't know these)
- `handle_viewer()` --calls--> `write_msg()`  [INFERRED]
  src/share.rs → src/protocol.rs
- `run()` --calls--> `write_msg()`  [INFERRED]
  src/watch.rs → src/protocol.rs
- `handle_viewer()` --calls--> `read_msg()`  [INFERRED]
  src/share.rs → src/protocol.rs
- `run()` --calls--> `read_msg()`  [INFERRED]
  src/watch.rs → src/protocol.rs
- `Resources` --references--> `AudioHandle`  [EXTRACTED]
  src/share.rs → src/audio/capture.rs

## Import Cycles
- 1-file cycle: `src/share.rs -> src/share.rs`

## Communities (50 total, 13 thin omitted)

### Community 0 - "share_view.rs"
Cohesion: 0.11
Nodes (13): MAX_ROOM, I_COPY, I_MIC, I_MIC_OFF, I_MONITOR, I_PEOPLE, I_PERSON, I_PLAY (+5 more)

### Community 1 - "decoder.rs"
Cohesion: 0.08
Nodes (33): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, hardware_kind() (+25 more)

### Community 2 - "capture.rs"
Cohesion: 0.08
Nodes (30): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+22 more)

### Community 3 - "Preset"
Cohesion: 0.05
Nodes (63): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), is_reduction(), label(), lighter_is_accepted_equal_and_heavier_are_not() (+55 more)

### Community 4 - "ToastKind"
Cohesion: 0.29
Nodes (5): ToastKind, Info, Success, Warn, Toasts

### Community 5 - "convert.rs"
Cohesion: 0.06
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "App"
Cohesion: 0.06
Nodes (34): App, classify(), own_hwnd(), button(), diagnostics(), FORCED, hint(), icon_button() (+26 more)

### Community 7 - "dxgi.rs"
Cohesion: 0.08
Nodes (9): Composer, create_duplication(), dxgi_draws_cursor(), DxgiCapture, Grab, GrabSink, run(), start() (+1 more)

### Community 9 - "watch.rs"
Cohesion: 0.07
Nodes (17): ChildGuard, e2e_window(), find(), PY, within(), _assert_conn(), CLOSE_FULL_U64, CLOSE_KICKED_U64 (+9 more)

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (16): enumerate_mfts(), friendly_name(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample (+8 more)

### Community 11 - "protocol.rs"
Cohesion: 0.07
Nodes (28): ALPN, check_len(), CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, encode_msg() (+20 more)

### Community 12 - "Shared"
Cohesion: 0.21
Nodes (4): CountGuard, handle_viewer(), sanitize(), Shared

### Community 13 - "update.rs"
Cohesion: 0.09
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "Overrides"
Cohesion: 0.10
Nodes (15): app_dir(), Config, load(), Overrides, overrides(), parse_overrides(), eframe::egui::ThemePreference, file_overrides_win() (+7 more)

### Community 15 - "speedtest.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.10
Nodes (12): app_icon(), DARK, GEIST, GEIST_MD, GEIST_MONO, GEIST_SB, I_SETTINGS, install() (+4 more)

### Community 17 - "watch_view.rs"
Cohesion: 0.11
Nodes (11): I_BACK, I_CHECK, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_INFO, I_MUTE, I_PASTE, I_VOLUME (+3 more)

### Community 19 - "share.rs"
Cohesion: 0.11
Nodes (10): Abr, ABR_FLOOR, HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), MAX_HANDSHAKES, retire() (+2 more)

### Community 20 - "wgc.rs"
Cohesion: 0.13
Nodes (5): border_removable(), PIXEL_FORMAT, property_present(), SESSION_TYPE, start()

### Community 21 - "workflows/release.yml"
Cohesion: 0.12
Nodes (15): jobs, release, name, permissions, contents, tags, runs-on, steps (+7 more)

### Community 23 - "Ui"
Cohesion: 0.20
Nodes (14): badge(), card(), checkbox(), chip(), danger_button(), field(), filled_button(), heading() (+6 more)

### Community 24 - "HWND"
Cohesion: 0.29
Nodes (10): device_name(), fit(), monitor_info(), primary_monitor(), refresh_rate(), source_size(), thumbnail(), thumbnail_monitor() (+2 more)

### Community 25 - "ShareSession"
Cohesion: 0.21
Nodes (3): invite_addr(), Overload, ShareSession

### Community 26 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

### Community 27 - "home.rs"
Cohesion: 0.18
Nodes (8): ended(), home(), tile(), I_ERROR, I_SHARE, I_UPDATE, I_WARN, I_WATCH

### Community 28 - "ci.yml"
Cohesion: 0.17
Nodes (11): runs-on, steps, jobs, build, name, branches, run, uses (+3 more)

### Community 29 - "LiveScreen"
Cohesion: 0.17
Nodes (11): Arquivos e configuração, Como usar, Compilar, Conexões que o app faz, Download e atualização, Licença, LiveScreen, Qualidade, sala e internet (+3 more)

### Community 30 - "playback.rs"
Cohesion: 0.13
Nodes (10): DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, opus_packet_roundtrip(), output_loop(), parse_packet(), PlaybackHandle, PREBUFFER_FLOATS (+2 more)

### Community 31 - "encoder.rs"
Cohesion: 0.08
Nodes (21): configure(), CpuCopy, create_mft(), Encoder, EncoderKind, Hardware, Software, EncoderStats (+13 more)

### Community 32 - "SinkHandle"
Cohesion: 0.08
Nodes (21): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, CountSink, FirstFrameSink (+13 more)

### Community 33 - "Screen"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 35 - "changelog"
Cohesion: 0.25
Nodes (7): labels, title, changelog, categories, exclude, authors, labels

### Community 39 - "enumerate"
Cohesion: 0.24
Nodes (6): enumerate(), exe_name(), inspect_window(), monitor_cb(), monitors(), window_cb()

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 48 - "Sink"
Cohesion: 0.20
Nodes (5): cut_feed(), pack_size(), PendingReconf, Sink, SinkTx

### Community 49 - "Resources"
Cohesion: 0.20
Nodes (4): Reconf, Resources, start_encoder(), EncoderHandle

### Community 51 - "LiveScreen"
Cohesion: 0.17
Nodes (8): Comandos, Estilo, graphify, LiveScreen, Mapa do código, Regras, Release, unpack_size()

## Knowledge Gaps
- **66 isolated node(s):** `livescreen`, `Process`, `System`, `Default`, `Device` (+61 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 417 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **13 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `SinkHandle`, `Screen`, `share_view.rs`, `Preset`, `ToastKind`, `watch.rs`, `update.rs`, `Overrides`, `watch_view.rs`, `ui/mod.rs`, `ShareSession`, `home.rs`, `encoder.rs`?**
  _High betweenness centrality (0.106) - this node is a cross-community bridge._
- **What connects `livescreen`, `Process`, `System` to the rest of the system?**
  _66 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `share_view.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `SourceId` connect `SinkHandle` to `share_view.rs`, `Preset`, `App`, `dxgi.rs`, `sources.rs`, `watch.rs`, `share.rs`, `wgc.rs`, `ui/mod.rs`, `HWND`?**
  _High betweenness centrality (0.085) - this node is a cross-community bridge._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07864488808227466 - nodes in this community are weakly interconnected._
- **Why does `Preset` connect `Preset` to `share_view.rs`, `SinkHandle`, `decoder.rs`, `App`, `watch.rs`, `protocol.rs`, `Shared`, `Overrides`, `Resources`, `share.rs`, `LiveScreen`, `ShareSession`?**
  _High betweenness centrality (0.077) - this node is a cross-community bridge._
- **Should `capture.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0782312925170068 - nodes in this community are weakly interconnected._