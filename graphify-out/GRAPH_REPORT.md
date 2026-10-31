# Graph Report - LiveScreen-publish  (2026-10-31)

## Corpus Check
- 75 files · ~97,461 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 54 file(s) not represented in the graph (top: .xml 24, .woff2 10, .ttf 7)

## Summary
- 2298 nodes · 4767 edges · 116 communities (85 shown, 31 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 144 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `cd6479ad`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- watch_view.rs
- Hosting.kt
- Shared
- ToastKind
- convert.rs
- settings.rs
- sources.rs
- Ui
- mf.rs
- protocol.rs
- steps
- update.rs
- ViewerModel
- playback.rs
- theme.rs
- share_view.rs
- gpu.rs
- host/web.rs
- Host
- with
- ui/mod.rs
- capture.rs
- Picker
- ShareSession
- .start
- share.rs
- steps
- LiveScreen
- share.rs
- HostService.kt
- wgc.rs
- test-changelog.sh
- rust/src/lib.rs
- Native
- src/friends.rs
- App
- viewer.rs
- main.rs
- present.rs
- Screens.kt
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- capture/mod.rs
- codec_roundtrip.rs
- host/mod.rs
- SourceId
- Theme.kt
- preset.rs
- Dib
- format_invite
- WatchSession
- VideoDecoder
- BudgetExceeded
- Resources
- Callback
- Codec
- run
- playback.rs
- config.rs
- HomeScreen
- LiveScreen
- DxgiCapture
- capture/mod.rs
- read_video_frame
- ScreenEncoder
- handle_frame
- e2e_window.rs
- Callback
- HostMsg
- src/chat.rs
- home.rs
- sharing
- pages.yml
- Android (`android/`)
- AudioEncoder
- ui/friends.rs
- SignalOut
- DxgiCapture
- BtnKind
- UpdateInfo
- super
- WebState
- AudioPlayer
- CallState
- FieldTone
- ViewerMsg
- home.rs
- SocketAddr
- MainActivity.kt
- run
- BudgetExceeded
- inspect_window
- peer
- Tab
- controls
- EndpointId
- handle_frame
- CallTone
- chevron_icon
- WebPeer
- SignalIn
- WgcCapture
- ChatLimiter
- AudioState

## God Nodes (most connected - your core abstractions)
1. `App` - 74 edges
2. `Preset` - 53 edges
3. `ViewerModel` - 45 edges
4. `Native` - 42 edges
5. `Converter` - 33 edges
6. `Codec` - 30 edges
7. `Shared` - 28 edges
8. `Host` - 27 edges
9. `EncodeParams` - 26 edges
10. `ShareSession` - 26 edges

## Surprising Connections (you probably didn't know these)
- `Web (`web/`)` --references--> `Room`  [INFERRED]
  AGENTS.md → web/src/worker.js
- `Android (`android/`)` --references--> `AudioEncoder`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/AudioEncoder.kt
- `Android (`android/`)` --references--> `AudioPlayer`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/AudioPlayer.kt
- `Android (`android/`)` --references--> `HostService`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/HostService.kt
- `Regras` --references--> `HostService`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/HostService.kt

## Import Cycles
- None detected.

## Communities (116 total, 31 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.14
Nodes (19): as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size(), Gpu (+11 more)

### Community 1 - "watch_view.rs"
Cohesion: 0.07
Nodes (19): Screen, Ended, Home, SharePicker, Sharing, Watching, WatchInput, I_CHAT (+11 more)

### Community 2 - "Hosting.kt"
Cohesion: 0.07
Nodes (5): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull()

### Community 3 - "Shared"
Cohesion: 0.14
Nodes (9): admit(), answer_presence(), CountGuard, handle_viewer(), Peer, PeerConn, Iroh, Web (+1 more)

### Community 4 - "ToastKind"
Cohesion: 0.12
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "settings.rs"
Cohesion: 0.25
Nodes (17): button(), diagnostics(), experimental(), FORCED, hint(), icon_button(), open_help(), open_logs() (+9 more)

### Community 9 - "Ui"
Cohesion: 0.16
Nodes (22): button(), button_full(), call_button(), checkbox(), draw_button(), ghost_icon_button(), hangup_button(), heading() (+14 more)

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (18): enumerate_mfts(), friendly_name(), has_param_sets(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange (+10 more)

### Community 11 - "protocol.rs"
Cohesion: 0.07
Nodes (19): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, encode_msg() (+11 more)

### Community 12 - "steps"
Cohesion: 0.10
Nodes (20): runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy, name, branches (+12 more)

### Community 13 - "update.rs"
Cohesion: 0.10
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "ViewerModel"
Cohesion: 0.13
Nodes (5): ChatItem, Friend, parseChatItems(), Status, ViewerModel

### Community 15 - "playback.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (33): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), card(), field() (+25 more)

### Community 17 - "share_view.rs"
Cohesion: 0.09
Nodes (15): AUTO_QUALITY_COOLDOWN, MAX_ROOM, Starting, Thumb, Update, I_BACK, I_CHECK, I_MIC (+7 more)

### Community 19 - "host/web.rs"
Cohesion: 0.06
Nodes (9): BROWSER_PREDICT, CONNECT_TIMEOUT, HOST_PREDICT, INIT_CRYPTO, MAX_SDP, MAX_WEB_CHAT, STUN_SERVERS, STUN_TIMEOUT (+1 more)

### Community 21 - "with"
Cohesion: 0.06
Nodes (34): runs-on, steps, GIT_CLIFF_TAG_PATTERN, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release (+26 more)

### Community 22 - "ui/mod.rs"
Cohesion: 0.09
Nodes (8): classify(), EndKind, Error, Info, Warn, on_screen(), own_hwnd(), run()

### Community 23 - "capture.rs"
Cohesion: 0.08
Nodes (30): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+22 more)

### Community 24 - "Picker"
Cohesion: 0.20
Nodes (8): begin_start(), Picker, picker(), poll_starting(), Preview, source_card(), spawn_share_job(), to_image()

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (15): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+7 more)

### Community 26 - ".start"
Cohesion: 0.20
Nodes (3): Hosting, Callback, Run

### Community 27 - "share.rs"
Cohesion: 0.09
Nodes (14): EncodeParams, is_reduction(), SourceInfo, cut_feed(), pack_size(), PendingReconf, Reconf, Resources (+6 more)

### Community 28 - "steps"
Cohesion: 0.09
Nodes (29): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+21 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (assistir e transmitir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "share.rs"
Cohesion: 0.12
Nodes (8): Overrides, black_nv12(), candidates(), detect(), hardware_kinds(), probe_child(), run_child(), try_encoder()

### Community 32 - "wgc.rs"
Cohesion: 0.13
Nodes (5): border_removable(), PIXEL_FORMAT, property_present(), SESSION_TYPE, start()

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.06
Nodes (58): AbortOnDrop, __android_log_write(), audio_loop(), get_host(), get_session(), HostSession, init_logging(), INIT_LOGGING (+50 more)

### Community 35 - "Native"
Cohesion: 0.09
Nodes (6): Amigos e presença, Chat, Regras, Native, HostScreen(), Presence

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 38 - "App"
Cohesion: 0.10
Nodes (4): App, connect(), sync_visible(), Frame

### Community 39 - "viewer.rs"
Cohesion: 0.09
Nodes (16): BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64, CLOSE_KICKED_U64, CLOSE_PROTOCOL_U64, close_reason(), CLOSE_STOPPED_U64, DecodeStats (+8 more)

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.06
Nodes (9): Mmcss, Display, GpuFrame, Imported, NEXT_SURFACES_ID, Producer, SLOTS, Surfaces (+1 more)

### Community 42 - "Screens.kt"
Cohesion: 0.02
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "capture/mod.rs"
Cohesion: 0.15
Nodes (4): CountSink, FirstFrameSink, FrameSink, SinkHandle

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.20
Nodes (5): AbortJoinSet<T>, Target, Friend, Invite, verify_room_proof()

### Community 50 - "host/mod.rs"
Cohesion: 0.09
Nodes (11): Abr, ABR_FLOOR, HANDSHAKE_TIMEOUT, ids(), invite_addr(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), MAX_HANDSHAKES (+3 more)

### Community 51 - "SourceId"
Cohesion: 0.31
Nodes (6): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor()

### Community 52 - "Theme.kt"
Cohesion: 0.18
Nodes (3): inter(), LiveScreenTheme(), Sinal

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (26): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not(), LIMIT_FPS (+18 more)

### Community 54 - "Dib"
Cohesion: 0.20
Nodes (11): Dib, fit(), refresh_rate(), source_size(), SourceId, Monitor, Window, thumbnail() (+3 more)

### Community 55 - "format_invite"
Cohesion: 0.23
Nodes (10): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), is_valid_invite() (+2 more)

### Community 58 - "BudgetExceeded"
Cohesion: 0.12
Nodes (15): ROOM_KEY_LEN, host_ws_url(), sdp_fingerprint(), stun_binding_request(), STUN_MAGIC_COOKIE, stun_mapped_addr(), test_host_ws_url(), test_stun_binding_request() (+7 more)

### Community 59 - "Resources"
Cohesion: 0.12
Nodes (14): HostStats, EncodedFrame, configure(), CpuCopy, create_mft(), Encoder, Feed, Idle (+6 more)

### Community 61 - "Codec"
Cohesion: 0.19
Nodes (9): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+1 more)

### Community 62 - "run"
Cohesion: 0.26
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 63 - "playback.rs"
Cohesion: 0.14
Nodes (10): conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, output_loop(), PlaybackHandle, PREBUFFER_FLOATS (+2 more)

### Community 64 - "config.rs"
Cohesion: 0.22
Nodes (12): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), overrides() (+4 more)

### Community 65 - "HomeScreen"
Cohesion: 0.21
Nodes (15): AddFriendDialog(), App(), Badge(), CallButton(), ChatSheet(), clipboardText(), FriendsSection(), HomeScreen() (+7 more)

### Community 66 - "LiveScreen"
Cohesion: 0.18
Nodes (10): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+2 more)

### Community 67 - "DxgiCapture"
Cohesion: 0.18
Nodes (5): Navegador (web), bytesToHex(), hexToBytes(), ICE_SERVERS, Room

### Community 68 - "capture/mod.rs"
Cohesion: 0.28
Nodes (9): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+1 more)

### Community 69 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 71 - "handle_frame"
Cohesion: 0.20
Nodes (10): wrangler, devDependencies, wrangler@^4, name, private, scripts, check, deploy (+2 more)

### Community 72 - "e2e_window.rs"
Cohesion: 0.18
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 73 - "Callback"
Cohesion: 0.11
Nodes (23): addOpusStereo(), appendChat(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect(), connectTips() (+15 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "src/chat.rs"
Cohesion: 0.11
Nodes (17): CHAT_HISTORY, ChatLine, ChatLog, ChatRole, Host, Viewer, clean_chat(), DONATE_AMOUNTS (+9 more)

### Community 77 - "home.rs"
Cohesion: 0.13
Nodes (10): I_CIRCLE_CHECK, I_INFO, I_WARN, LIFETIME, MAX_VISIBLE, ToastKind, Info, Success (+2 more)

### Community 78 - "sharing"
Cohesion: 0.18
Nodes (18): ShareStatus, viewers_for(), Upload, Done, Failed, Measuring, Unknown, chat_card() (+10 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 84 - "Android (`android/`)"
Cohesion: 0.27
Nodes (4): Android (`android/`), GhAsset, Updater, UpdaterTest

### Community 86 - "ui/friends.rs"
Cohesion: 0.12
Nodes (6): Draft, MAX_LIST, I_ADD_FRIEND, I_COPY, I_PEOPLE, I_TRASH

### Community 87 - "SignalOut"
Cohesion: 0.33
Nodes (6): SignalOut, Answer, Close, Host, Ping, Reject

### Community 89 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 90 - "UpdateInfo"
Cohesion: 0.36
Nodes (7): UpdateInfo, Available, Downloading, Failed, Idle, Ready, UpdateState

### Community 91 - "super"
Cohesion: 0.17
Nodes (5): Experimental, file_overrides_win(), old_ui_json_keeps_new_defaults(), partial_json_keeps_defaults(), Prefs

### Community 92 - "WebState"
Cohesion: 0.40
Nodes (5): WebState, Connecting, Off, Online, Retrying

### Community 94 - "CallState"
Cohesion: 0.40
Nodes (5): CallState, Active, Alert, HangUp, Neutral

### Community 95 - "FieldTone"
Cohesion: 0.50
Nodes (4): FieldTone, Error, Normal, Warn

### Community 96 - "ViewerMsg"
Cohesion: 0.33
Nodes (6): ViewerMsg, Chat, Hello, RequestKeyframe, SetQuality, SetVisible

### Community 97 - "home.rs"
Cohesion: 0.16
Nodes (12): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+4 more)

### Community 98 - "SocketAddr"
Cohesion: 0.20
Nodes (8): drain_outputs(), host_predictions(), host_predictions_radmin_rule(), maps(), ports(), sa(), Socks, stun_mappings()

### Community 100 - "run"
Cohesion: 0.20
Nodes (9): base_url(), default_v4(), init_crypto(), is_physical(), looks_virtual(), route_info(), RouteInfo, run() (+1 more)

### Community 103 - "inspect_window"
Cohesion: 0.50
Nodes (3): exe_name(), inspect_window(), window_cb()

### Community 104 - "peer"
Cohesion: 0.25
Nodes (5): rtt_congested(), browser_predictions(), browser_predictions_follow_first_v4_srflx(), local_v6(), peer()

### Community 105 - "Tab"
Cohesion: 0.29
Nodes (7): Tab, Diagnostics, Experimental, Share, Updates, Video, TABS

### Community 106 - "controls"
Cohesion: 0.33
Nodes (6): ControlAction, Leave, None, ToggleFullscreen, controls(), watching()

### Community 109 - "CallTone"
Cohesion: 0.40
Nodes (5): CallTone, Active, Alert, Neutral, Plain

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 115 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

## Knowledge Gaps
- **119 isolated node(s):** `livescreen`, `Primary`, `Secondary`, `Ghost`, `Danger` (+114 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 864 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **31 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Preset` connect `ShareSession` to `ViewerMsg`, `rust/src/lib.rs`, `Shared`, `settings.rs`, `viewer.rs`, `super`, `sharing`, `share_view.rs`, `host/mod.rs`, `Host`, `preset.rs`, `Picker`, `share.rs`, `share.rs`?**
  _High betweenness centrality (0.073) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Primary`, `Secondary` to the rest of the system?**
  _119 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13513513513513514 - nodes in this community are weakly interconnected._
- **Why does `Regras` connect `Native` to `LiveScreen`, `DxgiCapture`, `ViewerModel`, `.start`, `share.rs`, `HostService.kt`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Should `watch_view.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07407407407407407 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `ViewerModel` to `HomeScreen`, `Hosting.kt`, `MainActivity.kt`, `Native`, `Android (`android/`)`, `VideoDecoder`, `UpdateInfo`, `AudioPlayer`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._