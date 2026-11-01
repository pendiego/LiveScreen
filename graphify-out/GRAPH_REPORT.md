# Graph Report - LiveScreen-publish  (2026-11-01)

## Corpus Check
- 77 files · ~100,759 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 55 file(s) not represented in the graph (top: .xml 25, .woff2 10, .ttf 7)

## Summary
- 2342 nodes · 4850 edges · 107 communities (79 shown, 28 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 146 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `213e9b92`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- watch_view.rs
- Hosting.kt
- host/mod.rs
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
- speedtest.rs
- theme.rs
- Screen
- gpu.rs
- host/web.rs
- Host
- with
- ui/mod.rs
- capture.rs
- share_view.rs
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
- Converter
- codec_roundtrip.rs
- e2e_window.rs
- SourceId
- Theme.kt
- preset.rs
- OutputResult
- parse_invite
- WatchSession
- VideoDecoder
- BudgetExceeded
- Resources
- Callback
- Codec
- run
- ViewerSession
- super
- HomeScreen
- LiveScreen
- DxgiCapture
- enumerate_mfts
- variant_with
- ScreenEncoder
- handle_frame
- WatchStatus
- Callback
- HostMsg
- src/chat.rs
- toast.rs
- sharing
- pages.yml
- Android (`android/`)
- AudioEncoder
- ui/friends.rs
- SignalOut
- DxgiCapture
- UpdateInfo
- Target
- WebState
- AudioPlayer
- CallState
- FieldTone
- ViewerMsg
- home.rs
- SocketAddr
- run
- inspect_window
- Dib
- EndpointId
- peer
- SignalIn

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

## Communities (107 total, 28 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.14
Nodes (20): EncodedFrame, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size() (+12 more)

### Community 1 - "watch_view.rs"
Cohesion: 0.08
Nodes (18): I_CHAT, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY (+10 more)

### Community 2 - "Hosting.kt"
Cohesion: 0.07
Nodes (5): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull()

### Community 3 - "host/mod.rs"
Cohesion: 0.06
Nodes (22): Abr, ABR_FLOOR, admit(), answer_presence(), AudioState, Off, On, Unavailable (+14 more)

### Community 4 - "ToastKind"
Cohesion: 0.14
Nodes (11): bgra(), collect(), create_bgra(), fingerprint(), forward(), luma(), match_index(), psnr() (+3 more)

### Community 6 - "settings.rs"
Cohesion: 0.14
Nodes (26): button(), diagnostics(), experimental(), FORCED, hint(), icon_button(), open_help(), open_logs() (+18 more)

### Community 9 - "Ui"
Cohesion: 0.12
Nodes (27): button(), button_full(), call_button(), CallTone, Active, Alert, Neutral, Plain (+19 more)

### Community 10 - "mf.rs"
Cohesion: 0.12
Nodes (3): has_param_sets(), sample_survives_need_more_input(), startup()

### Community 11 - "protocol.rs"
Cohesion: 0.05
Nodes (26): ALPN, budget_released_on_drop(), Budgeted, BudgetExceeded, check_len(), CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED (+18 more)

### Community 12 - "steps"
Cohesion: 0.09
Nodes (22): runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy, name, permissions (+14 more)

### Community 13 - "update.rs"
Cohesion: 0.06
Nodes (35): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, keygen() (+27 more)

### Community 14 - "ViewerModel"
Cohesion: 0.12
Nodes (5): ChatItem, Friend, parseChatItems(), Status, ViewerModel

### Community 15 - "speedtest.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (35): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), card(), chevron_icon() (+27 more)

### Community 17 - "Screen"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 18 - "gpu.rs"
Cohesion: 0.08
Nodes (3): create_device(), Sendable, winrt_device()

### Community 19 - "host/web.rs"
Cohesion: 0.06
Nodes (12): BROWSER_PREDICT, CONNECT_TIMEOUT, drop_unsafe_candidates(), drop_unsafe_candidates_keeps_lan(), HOST_PREDICT, INIT_CRYPTO, MAX_SDP, MAX_WEB_CHAT (+4 more)

### Community 20 - "Host"
Cohesion: 0.10
Nodes (4): Feeds, Host, invite_addr(), Overload

### Community 21 - "with"
Cohesion: 0.06
Nodes (38): runs-on, steps, GIT_CLIFF_TAG_PATTERN, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release (+30 more)

### Community 23 - "capture.rs"
Cohesion: 0.05
Nodes (40): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+32 more)

### Community 24 - "share_view.rs"
Cohesion: 0.07
Nodes (20): AUTO_QUALITY_COOLDOWN, begin_start(), MAX_ROOM, Picker, poll_starting(), Preview, spawn_share_job(), Starting (+12 more)

### Community 25 - "ShareSession"
Cohesion: 0.10
Nodes (18): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+10 more)

### Community 26 - ".start"
Cohesion: 0.22
Nodes (3): Hosting, Callback, Run

### Community 27 - "share.rs"
Cohesion: 0.09
Nodes (13): EncodeParams, is_reduction(), cut_feed(), pack_size(), PendingReconf, Reconf, Resources, ShareSession (+5 more)

### Community 28 - "steps"
Cohesion: 0.08
Nodes (32): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+24 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (assistir e transmitir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "share.rs"
Cohesion: 0.14
Nodes (7): codec_roundtrip(), H, N, pattern(), patterns(), TAU, W

### Community 32 - "wgc.rs"
Cohesion: 0.06
Nodes (20): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, CountSink, FirstFrameSink (+12 more)

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
Nodes (3): App, classify(), own_hwnd()

### Community 39 - "viewer.rs"
Cohesion: 0.16
Nodes (10): BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64, CLOSE_KICKED_U64, CLOSE_PROTOCOL_U64, close_reason(), CLOSE_STOPPED_U64, DecodeStats (+2 more)

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

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.20
Nodes (5): AbortJoinSet<T>, Target, Friend, Invite, verify_room_proof()

### Community 50 - "e2e_window.rs"
Cohesion: 0.18
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 51 - "SourceId"
Cohesion: 0.22
Nodes (15): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate(), source_size() (+7 more)

### Community 52 - "Theme.kt"
Cohesion: 0.10
Nodes (4): MainActivity, inter(), LiveScreenTheme(), Sinal

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (26): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not(), LIMIT_FPS (+18 more)

### Community 54 - "OutputResult"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 55 - "parse_invite"
Cohesion: 0.25
Nodes (9): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), is_valid_invite(), input() (+1 more)

### Community 58 - "BudgetExceeded"
Cohesion: 0.11
Nodes (24): ROOM_KEY_LEN, host_ws_url(), NONCE_LEN, OFFER_VEC, open_offer(), open_with(), SALA, seal_answer() (+16 more)

### Community 59 - "Resources"
Cohesion: 0.12
Nodes (14): HostStats, configure(), CpuCopy, create_mft(), Encoder, Feed, Frame, Idle (+6 more)

### Community 61 - "Codec"
Cohesion: 0.12
Nodes (16): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+8 more)

### Community 62 - "run"
Cohesion: 0.24
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 64 - "super"
Cohesion: 0.10
Nodes (18): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+10 more)

### Community 65 - "HomeScreen"
Cohesion: 0.13
Nodes (20): AddFriendDialog(), App(), Badge(), BtnKind, Danger, Ghost, Primary, Secondary (+12 more)

### Community 66 - "LiveScreen"
Cohesion: 0.18
Nodes (10): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+2 more)

### Community 67 - "DxgiCapture"
Cohesion: 0.18
Nodes (5): Navegador (web), bytesToHex(), hexToBytes(), ICE_SERVERS, Room

### Community 68 - "enumerate_mfts"
Cohesion: 0.22
Nodes (4): enumerate_mfts(), friendly_name(), set_codec_value(), subtype()

### Community 69 - "variant_with"
Cohesion: 0.47
Nodes (3): variant_bool(), variant_u32(), variant_with()

### Community 71 - "handle_frame"
Cohesion: 0.20
Nodes (10): wrangler, devDependencies, wrangler@^4, name, private, scripts, check, deploy (+2 more)

### Community 72 - "WatchStatus"
Cohesion: 0.40
Nodes (5): Phase, Connecting, Ended, Playing, WatchStatus

### Community 73 - "Callback"
Cohesion: 0.11
Nodes (26): addOpusStereo(), appendChat(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect(), connectTips() (+18 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "src/chat.rs"
Cohesion: 0.10
Nodes (19): CHAT_HISTORY, ChatLimiter, ChatLine, ChatLog, ChatRole, Host, Viewer, clean_chat() (+11 more)

### Community 77 - "toast.rs"
Cohesion: 0.13
Nodes (10): I_CIRCLE_CHECK, I_INFO, I_WARN, LIFETIME, MAX_VISIBLE, ToastKind, Info, Success (+2 more)

### Community 78 - "sharing"
Cohesion: 0.21
Nodes (17): ShareStatus, Upload, Done, Failed, Measuring, Unknown, chat_card(), confirm_modal() (+9 more)

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

### Community 90 - "UpdateInfo"
Cohesion: 0.36
Nodes (7): UpdateInfo, Available, Downloading, Failed, Idle, Ready, UpdateState

### Community 91 - "Target"
Cohesion: 0.67
Nodes (3): Target, Nv12, Rgb

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
Cohesion: 0.16
Nodes (11): browser_predictions(), browser_predictions_follow_first_v4_srflx(), drain_outputs(), forbidden_dest(), host_predictions(), host_predictions_radmin_rule(), maps(), ports() (+3 more)

### Community 100 - "run"
Cohesion: 0.18
Nodes (9): base_url(), default_v4(), init_crypto(), is_physical(), looks_virtual(), route_info(), RouteInfo, run() (+1 more)

### Community 103 - "inspect_window"
Cohesion: 0.33
Nodes (4): exe_name(), inspect_window(), window_cb(), window_rect()

### Community 106 - "Dib"
Cohesion: 0.31
Nodes (3): Dib, fit(), thumbnail_window()

### Community 111 - "peer"
Cohesion: 0.17
Nodes (5): rtt_congested(), local_v6(), peer(), WebPeer, room_id()

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

## Knowledge Gaps
- **119 isolated node(s):** `livescreen`, `Primary`, `Secondary`, `Ghost`, `Danger` (+114 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 878 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **28 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Regras` connect `Native` to `LiveScreen`, `DxgiCapture`, `ViewerModel`, `.start`, `share.rs`, `HostService.kt`?**
  _High betweenness centrality (0.082) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Primary`, `Secondary` to the rest of the system?**
  _119 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13813813813813813 - nodes in this community are weakly interconnected._
- **Why does `Preset` connect `ShareSession` to `ViewerMsg`, `super`, `rust/src/lib.rs`, `host/mod.rs`, `settings.rs`, `viewer.rs`, `sharing`, `Host`, `preset.rs`, `share_view.rs`, `share.rs`, `share.rs`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Should `watch_view.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0812807881773399 - nodes in this community are weakly interconnected._
- **Why does `App` connect `App` to `super`, `home.rs`, `watch_view.rs`, `settings.rs`, `update.rs`, `sharing`, `toast.rs`, `Screen`, `SourceId`, `parse_invite`, `ui/friends.rs`, `ui/mod.rs`, `share_view.rs`, `WatchSession`, `share.rs`, `Codec`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._