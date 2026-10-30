# Graph Report - LiveScreen-publish  (2026-10-30)

## Corpus Check
- 76 files · ~92,702 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 54 file(s) not represented in the graph (top: .xml 24, .woff2 10, .ttf 7)

## Summary
- 2258 nodes · 4681 edges · 112 communities (79 shown, 33 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 129 edges (avg confidence: 0.76)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2bc80cbb`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- Ui
- Hosting.kt
- host/mod.rs
- ToastKind
- convert.rs
- App
- sources.rs
- sharing
- mf.rs
- protocol.rs
- steps
- update.rs
- ViewerModel
- playback.rs
- theme.rs
- watch_view.rs
- gpu.rs
- host/web.rs
- Host
- with
- ui/mod.rs
- ShareSession
- SourceInfo
- ShareSession
- .start
- share.rs
- steps
- LiveScreen
- encoder.rs
- HostService.kt
- wgc.rs
- test-changelog.sh
- rust/src/lib.rs
- Native
- src/friends.rs
- Updater.kt
- WatchSession
- capture/mod.rs
- main.rs
- present.rs
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- capture/mod.rs
- codec_roundtrip.rs
- Native
- SourceId
- Theme.kt
- preset.rs
- Dib
- format_invite
- viewer.rs
- MainActivity.kt
- BudgetExceeded
- Resources
- Callback
- Preset
- run
- capture.rs
- config.rs
- HomeScreen
- LiveScreen
- DxgiCapture
- Converter
- read_video_frame
- ScreenEncoder
- handle_frame
- e2e_window.rs
- Callback
- HostMsg
- src/chat.rs
- home.rs
- share_view.rs
- pages.yml
- watch_view.rs
- EncoderKind
- ToastKind
- SignalOut
- DxgiCapture
- BtnKind
- VideoDecoder
- ViewerMsg
- WebState
- SignalIn
- CallState
- FieldTone
- AudioEncoder
- ViewerSession
- SinalSnack
- frame_from_body
- BudgetExceeded
- write_msg
- handle_frame
- room_proof
- WatchStatus
- WgcCapture
- CallTone
- AudioState
- start
- Target

## God Nodes (most connected - your core abstractions)
1. `App` - 73 edges
2. `Preset` - 53 edges
3. `ViewerModel` - 45 edges
4. `Native` - 42 edges
5. `Converter` - 33 edges
6. `Codec` - 30 edges
7. `Shared` - 28 edges
8. `Host` - 26 edges
9. `EncodeParams` - 26 edges
10. `ShareSession` - 26 edges

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

## Communities (112 total, 33 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.13
Nodes (20): EncodedFrame, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size() (+12 more)

### Community 1 - "Ui"
Cohesion: 0.13
Nodes (26): badge(), button(), button_full(), call_button(), checkbox(), draw_button(), ghost_icon_button(), hangup_button() (+18 more)

### Community 2 - "Hosting.kt"
Cohesion: 0.07
Nodes (7): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull(), ChatItem, parseChatItems()

### Community 3 - "host/mod.rs"
Cohesion: 0.07
Nodes (20): Abr, ABR_FLOOR, admit(), answer_presence(), CountGuard, Feeds, handle_viewer(), HANDSHAKE_TIMEOUT (+12 more)

### Community 4 - "ToastKind"
Cohesion: 0.15
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 6 - "App"
Cohesion: 0.06
Nodes (30): App, classify(), own_hwnd(), button(), diagnostics(), FORCED, hint(), icon_button() (+22 more)

### Community 9 - "sharing"
Cohesion: 0.16
Nodes (20): ShareStatus, viewers_for(), Upload, Done, Failed, Measuring, Unknown, chat_card() (+12 more)

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (17): enumerate_mfts(), friendly_name(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample (+9 more)

### Community 11 - "protocol.rs"
Cohesion: 0.08
Nodes (12): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, LEGACY_PROTOCOL_VERSION (+4 more)

### Community 12 - "steps"
Cohesion: 0.10
Nodes (20): runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy, name, branches (+12 more)

### Community 13 - "update.rs"
Cohesion: 0.09
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "ViewerModel"
Cohesion: 0.13
Nodes (3): Friend, Status, ViewerModel

### Community 15 - "playback.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (31): app_icon(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), card(), chevron_icon(), field() (+23 more)

### Community 17 - "watch_view.rs"
Cohesion: 0.08
Nodes (11): Draft, MAX_LIST, I_ADD_FRIEND, I_CIRCLE_CHECK, I_COPY, I_INFO, I_PEOPLE, I_TRASH (+3 more)

### Community 19 - "host/web.rs"
Cohesion: 0.06
Nodes (14): rtt_congested(), base_url(), CONNECT_TIMEOUT, drain_outputs(), INIT_CRYPTO, init_crypto(), MAX_SDP, MAX_WEB_CHAT (+6 more)

### Community 20 - "Host"
Cohesion: 0.09
Nodes (5): Host, HostConfig, invite_addr(), Overload, ViewerInfo

### Community 21 - "with"
Cohesion: 0.06
Nodes (33): runs-on, steps, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release, name (+25 more)

### Community 23 - "ShareSession"
Cohesion: 0.21
Nodes (3): pick_encoder(), ShareSession, unpack_size()

### Community 24 - "SourceInfo"
Cohesion: 0.28
Nodes (7): enumerate(), exe_name(), inspect_window(), monitors(), SourceInfo, window_cb(), Update

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (15): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+7 more)

### Community 26 - ".start"
Cohesion: 0.22
Nodes (3): Hosting, Callback, Run

### Community 27 - "share.rs"
Cohesion: 0.08
Nodes (12): is_key(), main(), split_aus(), cut_feed(), pack_size(), PendingReconf, Reconf, Resources (+4 more)

### Community 28 - "steps"
Cohesion: 0.09
Nodes (29): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+21 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (assistir e transmitir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "encoder.rs"
Cohesion: 0.10
Nodes (13): Codec, Av1, H264, Hevc, supported_codecs(), configure(), vbv_bytes(), has_param_sets() (+5 more)

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.07
Nodes (58): AbortOnDrop, __android_log_write(), audio_loop(), get_host(), get_session(), HostSession, init_logging(), INIT_LOGGING (+50 more)

### Community 35 - "Native"
Cohesion: 0.09
Nodes (4): Amigos e presença, Regras, Native, Presence

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 37 - "Updater.kt"
Cohesion: 0.08
Nodes (11): Android (`android/`), GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle (+3 more)

### Community 39 - "capture/mod.rs"
Cohesion: 0.16
Nodes (4): CountSink, FirstFrameSink, FrameSink, SinkHandle

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.06
Nodes (9): Mmcss, Display, GpuFrame, Imported, NEXT_SURFACES_ID, Producer, SLOTS, Surfaces (+1 more)

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "capture/mod.rs"
Cohesion: 0.27
Nodes (9): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+1 more)

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.24
Nodes (4): AbortJoinSet<T>, Target, Friend, Invite

### Community 50 - "Native"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 51 - "SourceId"
Cohesion: 0.22
Nodes (14): device_name(), fit(), monitor_cb(), monitor_info(), primary_monitor(), refresh_rate(), source_size(), SourceId (+6 more)

### Community 52 - "Theme.kt"
Cohesion: 0.18
Nodes (3): inter(), LiveScreenTheme(), Sinal

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (28): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), is_reduction(), label(), light_presets_are_reductions_and_fall_back() (+20 more)

### Community 55 - "format_invite"
Cohesion: 0.27
Nodes (8): find_invite(), find_invite_in_pasted_text(), format_invite(), new_room_key(), parse_invite(), is_valid_invite(), input(), normalize_ticket()

### Community 56 - "viewer.rs"
Cohesion: 0.15
Nodes (10): BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64, CLOSE_KICKED_U64, CLOSE_PROTOCOL_U64, close_reason(), CLOSE_STOPPED_U64, DecodeStats (+2 more)

### Community 58 - "BudgetExceeded"
Cohesion: 0.11
Nodes (15): ROOM_KEY_LEN, host_ws_url(), sdp_fingerprint(), stun_binding_request(), STUN_MAGIC_COOKIE, stun_mapped_addr(), test_host_ws_url(), test_stun_binding_request() (+7 more)

### Community 59 - "Resources"
Cohesion: 0.19
Nodes (6): CpuCopy, Encoder, Feed, Idle, Stop, Nv12Frame

### Community 62 - "run"
Cohesion: 0.24
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 63 - "capture.rs"
Cohesion: 0.05
Nodes (40): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+32 more)

### Community 64 - "config.rs"
Cohesion: 0.10
Nodes (17): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+9 more)

### Community 65 - "HomeScreen"
Cohesion: 0.22
Nodes (16): AddFriendDialog(), App(), Badge(), CallButton(), ChatSheet(), clipboardText(), FriendsSection(), HomeScreen() (+8 more)

### Community 66 - "LiveScreen"
Cohesion: 0.20
Nodes (9): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+1 more)

### Community 67 - "DxgiCapture"
Cohesion: 0.20
Nodes (4): bytesToHex(), hexToBytes(), iceServers(), Room

### Community 69 - "read_video_frame"
Cohesion: 0.28
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 71 - "handle_frame"
Cohesion: 0.20
Nodes (10): wrangler, devDependencies, wrangler@^4, name, private, scripts, check, deploy (+2 more)

### Community 72 - "e2e_window.rs"
Cohesion: 0.18
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 73 - "Callback"
Cohesion: 0.15
Nodes (19): addOpusStereo(), appendChat(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect(), hexToBytes() (+11 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "src/chat.rs"
Cohesion: 0.11
Nodes (16): CHAT_HISTORY, ChatLimiter, ChatLine, ChatLog, ChatRole, Host, Viewer, clean_chat() (+8 more)

### Community 77 - "home.rs"
Cohesion: 0.16
Nodes (12): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+4 more)

### Community 78 - "share_view.rs"
Cohesion: 0.08
Nodes (17): AUTO_QUALITY_COOLDOWN, begin_start(), Picker, poll_starting(), Preview, Starting, Thumb, to_image() (+9 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 83 - "watch_view.rs"
Cohesion: 0.08
Nodes (18): I_CHAT, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY (+10 more)

### Community 84 - "EncoderKind"
Cohesion: 0.19
Nodes (9): create_mft(), EncoderKind, Hardware, Software, read_seq_header(), spawn(), stream_ids(), detect() (+1 more)

### Community 85 - "ToastKind"
Cohesion: 0.25
Nodes (6): spawn_share_job(), ToastKind, Info, Success, Warn, Toasts

### Community 87 - "SignalOut"
Cohesion: 0.33
Nodes (6): SignalOut, Answer, Close, Host, Ping, Reject

### Community 89 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 91 - "ViewerMsg"
Cohesion: 0.33
Nodes (6): ViewerMsg, Chat, Hello, RequestKeyframe, SetQuality, SetVisible

### Community 92 - "WebState"
Cohesion: 0.40
Nodes (5): WebState, Connecting, Off, Online, Retrying

### Community 93 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 94 - "CallState"
Cohesion: 0.40
Nodes (5): CallState, Active, Alert, HangUp, Neutral

### Community 95 - "FieldTone"
Cohesion: 0.50
Nodes (4): FieldTone, Error, Normal, Warn

### Community 98 - "SinalSnack"
Cohesion: 0.40
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 100 - "frame_from_body"
Cohesion: 0.29
Nodes (4): frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 102 - "write_msg"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 104 - "room_proof"
Cohesion: 0.40
Nodes (3): invite_roundtrip_and_proof_binding(), room_proof(), verify_room_proof()

### Community 105 - "WatchStatus"
Cohesion: 0.40
Nodes (5): Phase, Connecting, Ended, Playing, WatchStatus

### Community 107 - "CallTone"
Cohesion: 0.40
Nodes (5): CallTone, Active, Alert, Neutral, Plain

### Community 109 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

### Community 110 - "start"
Cohesion: 0.67
Nodes (3): border_removable(), property_present(), start()

### Community 111 - "Target"
Cohesion: 0.67
Nodes (3): Target, Nv12, Rgb

## Knowledge Gaps
- **118 isolated node(s):** `livescreen`, `Primary`, `Secondary`, `Ghost`, `Danger` (+113 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 846 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **33 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `config.rs`, `WatchSession`, `sharing`, `home.rs`, `share_view.rs`, `update.rs`, `watch_view.rs`, `Native`, `SourceId`, `watch_view.rs`, `ToastKind`, `ui/mod.rs`, `ShareSession`, `EncoderKind`, `format_invite`?**
  _High betweenness centrality (0.054) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Primary`, `Secondary` to the rest of the system?**
  _118 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1337126600284495 - nodes in this community are weakly interconnected._
- **Why does `Preset` connect `ShareSession` to `config.rs`, `ViewerSession`, `rust/src/lib.rs`, `host/mod.rs`, `App`, `share.rs`, `sharing`, `share_view.rs`, `Host`, `preset.rs`, `ShareSession`, `viewer.rs`, `ViewerMsg`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._
- **Should `Ui` be split into smaller, more focused modules?**
  _Cohesion score 0.13333333333333333 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `ViewerModel` to `HomeScreen`, `Hosting.kt`, `Native`, `Updater.kt`, `MainActivity.kt`, `VideoDecoder`, `Preset`?**
  _High betweenness centrality (0.042) - this node is a cross-community bridge._