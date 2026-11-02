# Graph Report - LiveScreen-publish  (2026-11-01)

## Corpus Check
- 77 files · ~104,300 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 55 file(s) not represented in the graph (top: .xml 25, .woff2 10, .ttf 7)

## Summary
- 2397 nodes · 4968 edges · 106 communities (81 shown, 25 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 146 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `68a58e2b`
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
- hdr.rs
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
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- wgc.rs
- codec_roundtrip.rs
- e2e_window.rs
- SourceId
- Theme.kt
- preset.rs
- OutputResult
- parse_invite
- WatchSession
- AudioEncoder
- BudgetExceeded
- Resources
- Callback
- Codec
- run
- host/mod.rs
- super
- HomeScreen
- LiveScreen
- DxgiCapture
- enumerate_mfts
- variant_with
- ScreenEncoder
- handle_frame
- MainActivity.kt
- Callback
- HostMsg
- src/chat.rs
- ToastKind
- sharing
- pages.yml
- run
- Android (`android/`)
- BtnKind
- ui/friends.rs
- peer
- DxgiCapture
- SinalSnack
- UpdateInfo
- ChatLimiter
- WebState
- AudioState
- CallState
- FieldTone
- ViewerMsg
- VideoPts
- SocketAddr
- Interface
- BudgetExceeded
- Dib
- EndpointId
- WebPeer
- SignalIn

## God Nodes (most connected - your core abstractions)
1. `App` - 74 edges
2. `Preset` - 53 edges
3. `ViewerModel` - 45 edges
4. `Native` - 42 edges
5. `Converter` - 33 edges
6. `Codec` - 31 edges
7. `SourceId` - 29 edges
8. `Shared` - 28 edges
9. `Host` - 27 edges
10. `EncodeParams` - 26 edges

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

## Communities (106 total, 25 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.13
Nodes (21): EncodedFrame, DecodeStats, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain() (+13 more)

### Community 1 - "watch_view.rs"
Cohesion: 0.07
Nodes (19): I_CHAT, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY (+11 more)

### Community 2 - "Hosting.kt"
Cohesion: 0.07
Nodes (7): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull(), ChatItem, parseChatItems()

### Community 3 - "Shared"
Cohesion: 0.15
Nodes (9): admit(), answer_presence(), CountGuard, handle_viewer(), Peer, PeerConn, Iroh, Web (+1 more)

### Community 4 - "ToastKind"
Cohesion: 0.12
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "settings.rs"
Cohesion: 0.13
Nodes (27): button(), diagnostics(), experimental(), FORCED, hint(), icon_button(), open_help(), open_logs() (+19 more)

### Community 8 - "sources.rs"
Cohesion: 0.05
Nodes (4): exe_name(), inspect_window(), PW_RENDERFULLCONTENT, window_cb()

### Community 9 - "Ui"
Cohesion: 0.12
Nodes (27): button(), button_full(), call_button(), CallTone, Active, Alert, Neutral, Plain (+19 more)

### Community 11 - "protocol.rs"
Cohesion: 0.06
Nodes (24): ALPN, budget_released_on_drop(), Budgeted, check_len(), CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL (+16 more)

### Community 12 - "steps"
Cohesion: 0.09
Nodes (22): runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy, name, permissions (+14 more)

### Community 13 - "update.rs"
Cohesion: 0.08
Nodes (30): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+22 more)

### Community 14 - "ViewerModel"
Cohesion: 0.12
Nodes (3): Friend, Status, ViewerModel

### Community 15 - "speedtest.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (35): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), card(), chevron_icon() (+27 more)

### Community 17 - "Screen"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 19 - "host/web.rs"
Cohesion: 0.06
Nodes (12): BROWSER_PREDICT, CONNECT_TIMEOUT, drop_unsafe_candidates(), drop_unsafe_candidates_keeps_lan(), HOST_PREDICT, INIT_CRYPTO, MAX_SDP, MAX_WEB_CHAT (+4 more)

### Community 20 - "Host"
Cohesion: 0.11
Nodes (4): Host, invite_addr(), Overload, rtt_congested()

### Community 21 - "with"
Cohesion: 0.06
Nodes (38): runs-on, steps, GIT_CLIFF_TAG_PATTERN, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release (+30 more)

### Community 23 - "capture.rs"
Cohesion: 0.05
Nodes (40): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+32 more)

### Community 24 - "share_view.rs"
Cohesion: 0.09
Nodes (21): SourceInfo, AUTO_QUALITY_COOLDOWN, begin_start(), Picker, picker(), poll_starting(), Preview, source_card() (+13 more)

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (15): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+7 more)

### Community 26 - ".start"
Cohesion: 0.15
Nodes (3): Hosting, Callback, Run

### Community 27 - "share.rs"
Cohesion: 0.09
Nodes (14): EncodeParams, is_reduction(), cut_feed(), pack_size(), PendingReconf, pick_encoder(), Reconf, Resources (+6 more)

### Community 28 - "steps"
Cohesion: 0.08
Nodes (32): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+24 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (assistir e transmitir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "hdr.rs"
Cohesion: 0.05
Nodes (14): compile_shader(), DEFAULT_SDR_WHITE_NITS, detect_hdr(), find_output_for_monitor(), get_monitor_gdi_name(), HdrConverter, HdrInfo, HLSL_SOURCE (+6 more)

### Community 32 - "wgc.rs"
Cohesion: 0.11
Nodes (13): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, CountSink, FirstFrameSink (+5 more)

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.07
Nodes (58): AbortOnDrop, __android_log_write(), audio_loop(), get_host(), get_session(), HostSession, init_logging(), INIT_LOGGING (+50 more)

### Community 35 - "Native"
Cohesion: 0.10
Nodes (5): Amigos e presença, Chat, Regras, Native, Presence

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 38 - "App"
Cohesion: 0.10
Nodes (4): App, classify(), own_hwnd(), Frame

### Community 39 - "viewer.rs"
Cohesion: 0.09
Nodes (15): BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64, CLOSE_KICKED_U64, CLOSE_PROTOCOL_U64, close_reason(), CLOSE_STOPPED_U64, Phase (+7 more)

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.06
Nodes (9): Mmcss, Display, GpuFrame, Imported, NEXT_SURFACES_ID, Producer, SLOTS, Surfaces (+1 more)

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "wgc.rs"
Cohesion: 0.09
Nodes (8): border_removable(), DEFAULT_PIXEL_FORMAT, handle_frame(), property_present(), SESSION_TYPE, start(), WgcCapture, Sendable

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.20
Nodes (5): AbortJoinSet<T>, Target, Friend, Invite, verify_room_proof()

### Community 50 - "e2e_window.rs"
Cohesion: 0.18
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 51 - "SourceId"
Cohesion: 0.21
Nodes (12): hmonitor_for_source(), device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate() (+4 more)

### Community 52 - "Theme.kt"
Cohesion: 0.18
Nodes (3): inter(), LiveScreenTheme(), Sinal

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (26): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not(), LIMIT_FPS (+18 more)

### Community 54 - "OutputResult"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 55 - "parse_invite"
Cohesion: 0.23
Nodes (10): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), is_valid_invite() (+2 more)

### Community 57 - "AudioEncoder"
Cohesion: 0.06
Nodes (4): AudioEncoder, AudioPlayer, CodecState, VideoDecoder

### Community 58 - "BudgetExceeded"
Cohesion: 0.11
Nodes (24): ROOM_KEY_LEN, host_ws_url(), NONCE_LEN, OFFER_VEC, open_offer(), open_with(), SALA, seal_answer() (+16 more)

### Community 59 - "Resources"
Cohesion: 0.12
Nodes (12): configure(), CpuCopy, create_mft(), Encoder, Feed, Idle, Stop, Nv12Frame (+4 more)

### Community 61 - "Codec"
Cohesion: 0.10
Nodes (17): Codec, Av1, H264, Hevc, supported_codecs(), EncoderKind, Hardware, Software (+9 more)

### Community 62 - "run"
Cohesion: 0.24
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 63 - "host/mod.rs"
Cohesion: 0.10
Nodes (10): Abr, ABR_FLOOR, decide_target_fps(), Feeds, HANDSHAKE_TIMEOUT, HostStats, MAX_HANDSHAKES, retire() (+2 more)

### Community 64 - "super"
Cohesion: 0.09
Nodes (18): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+10 more)

### Community 65 - "HomeScreen"
Cohesion: 0.22
Nodes (16): AddFriendDialog(), App(), Badge(), CallButton(), ChatSheet(), clipboardText(), FriendsSection(), HomeScreen() (+8 more)

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

### Community 73 - "Callback"
Cohesion: 0.11
Nodes (26): addOpusStereo(), appendChat(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect(), connectTips() (+18 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "src/chat.rs"
Cohesion: 0.11
Nodes (17): CHAT_HISTORY, ChatLine, ChatLog, ChatRole, Host, Viewer, clean_chat(), DONATE_AMOUNTS (+9 more)

### Community 77 - "ToastKind"
Cohesion: 0.25
Nodes (6): spawn_share_job(), ToastKind, Info, Success, Warn, Toasts

### Community 78 - "sharing"
Cohesion: 0.17
Nodes (19): ShareStatus, viewers_for(), Upload, Done, Failed, Measuring, Unknown, chat_card() (+11 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 83 - "run"
Cohesion: 0.22
Nodes (5): base_url(), drain_outputs(), init_crypto(), run(), super::Host

### Community 84 - "Android (`android/`)"
Cohesion: 0.27
Nodes (4): Android (`android/`), GhAsset, Updater, UpdaterTest

### Community 85 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 86 - "ui/friends.rs"
Cohesion: 0.07
Nodes (22): Draft, MAX_LIST, ended(), footer(), home(), left_column(), pill(), share_tile() (+14 more)

### Community 87 - "peer"
Cohesion: 0.12
Nodes (13): browser_predictions(), browser_predictions_follow_first_v4_srflx(), default_v4(), forbidden_dest(), local_v6(), peer(), room_id(), SignalOut (+5 more)

### Community 89 - "SinalSnack"
Cohesion: 0.40
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 90 - "UpdateInfo"
Cohesion: 0.36
Nodes (7): UpdateInfo, Available, Downloading, Failed, Idle, Ready, UpdateState

### Community 92 - "WebState"
Cohesion: 0.40
Nodes (5): WebState, Connecting, Off, Online, Retrying

### Community 93 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

### Community 94 - "CallState"
Cohesion: 0.40
Nodes (5): CallState, Active, Alert, HangUp, Neutral

### Community 95 - "FieldTone"
Cohesion: 0.50
Nodes (4): FieldTone, Error, Normal, Warn

### Community 96 - "ViewerMsg"
Cohesion: 0.33
Nodes (6): ViewerMsg, Chat, Hello, RequestKeyframe, SetQuality, SetVisible

### Community 98 - "SocketAddr"
Cohesion: 0.33
Nodes (7): host_predictions(), host_predictions_radmin_rule(), maps(), ports(), sa(), Socks, stun_mappings()

### Community 100 - "Interface"
Cohesion: 0.33
Nodes (4): is_physical(), looks_virtual(), route_info(), RouteInfo

### Community 106 - "Dib"
Cohesion: 0.29
Nodes (6): Dib, fit(), thumbnail(), thumbnail_monitor(), thumbnail_window(), window_rect()

### Community 107 - "EndpointId"
Cohesion: 0.20
Nodes (5): HostConfig, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), ViewerInfo

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

## Knowledge Gaps
- **119 isolated node(s):** `livescreen`, `Primary`, `Secondary`, `Ghost`, `Danger` (+114 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 902 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **25 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Regras` connect `Native` to `LiveScreen`, `DxgiCapture`, `ViewerModel`, `.start`, `share.rs`, `HostService.kt`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Primary`, `Secondary` to the rest of the system?**
  _119 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13090418353576247 - nodes in this community are weakly interconnected._
- **Why does `Preset` connect `ShareSession` to `ViewerMsg`, `super`, `rust/src/lib.rs`, `Shared`, `viewer.rs`, `sharing`, `Host`, `preset.rs`, `share_view.rs`, `share.rs`, `host/mod.rs`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._
- **Should `watch_view.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07258064516129033 - nodes in this community are weakly interconnected._
- **Why does `App` connect `App` to `super`, `watch_view.rs`, `settings.rs`, `ToastKind`, `sharing`, `update.rs`, `Screen`, `SourceId`, `parse_invite`, `ui/friends.rs`, `ui/mod.rs`, `share_view.rs`, `WatchSession`, `share.rs`, `Codec`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._