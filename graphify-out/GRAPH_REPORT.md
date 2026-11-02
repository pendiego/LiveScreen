# Graph Report - LiveScreen-publish  (2026-11-01)

## Corpus Check
- 77 files · ~104,953 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 55 file(s) not represented in the graph (top: .xml 25, .woff2 10, .ttf 7)

## Summary
- 2402 nodes · 4987 edges · 116 communities (86 shown, 30 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 148 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `0faa6d12`
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
- Updater.kt
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
- VideoDecoder
- BudgetExceeded
- Resources
- Callback
- Codec
- run
- host/mod.rs
- super
- HomeScreen
- playback.rs
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
- home.rs
- BtnKind
- ui/friends.rs
- peer
- DxgiCapture
- SinalSnack
- capture/mod.rs
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
- AudioEncoder
- read_video_frame
- SourceId
- EndpointId
- frame_from_body
- Feeds
- write_msg
- WebPeer
- SignalIn
- AudioPlayer
- room_proof
- WatchStatus

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
- `Android (`android/`)` --references--> `AudioEncoder`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/AudioEncoder.kt
- `Android (`android/`)` --references--> `AudioPlayer`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/AudioPlayer.kt
- `Android (`android/`)` --references--> `HostService`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/HostService.kt
- `Regras` --references--> `HostService`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/HostService.kt
- `Android (`android/`)` --references--> `Hosting`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/Hosting.kt

## Import Cycles
- None detected.

## Communities (116 total, 30 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.13
Nodes (21): EncodedFrame, DecodeStats, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain() (+13 more)

### Community 1 - "watch_view.rs"
Cohesion: 0.08
Nodes (18): I_CHAT, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY (+10 more)

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
Cohesion: 0.13
Nodes (28): button(), diagnostics(), experimental(), FORCED, hint(), icon_button(), open_help(), open_logs() (+20 more)

### Community 8 - "sources.rs"
Cohesion: 0.05
Nodes (4): exe_name(), inspect_window(), PW_RENDERFULLCONTENT, window_cb()

### Community 9 - "Ui"
Cohesion: 0.12
Nodes (27): button(), button_full(), call_button(), CallTone, Active, Alert, Neutral, Plain (+19 more)

### Community 11 - "protocol.rs"
Cohesion: 0.08
Nodes (12): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, LEGACY_PROTOCOL_VERSION (+4 more)

### Community 12 - "steps"
Cohesion: 0.09
Nodes (22): runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy, name, permissions (+14 more)

### Community 13 - "update.rs"
Cohesion: 0.08
Nodes (30): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+22 more)

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
Cohesion: 0.17
Nodes (12): classify(), EndKind, Error, Info, Warn, Screen, Ended, Home (+4 more)

### Community 18 - "gpu.rs"
Cohesion: 0.08
Nodes (3): create_device(), Sendable, winrt_device()

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
Cohesion: 0.08
Nodes (30): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+22 more)

### Community 24 - "share_view.rs"
Cohesion: 0.08
Nodes (22): SourceInfo, AUTO_QUALITY_COOLDOWN, begin_start(), Picker, picker(), poll_starting(), Preview, source_card() (+14 more)

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (15): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+7 more)

### Community 26 - ".start"
Cohesion: 0.22
Nodes (3): Hosting, Callback, Run

### Community 27 - "share.rs"
Cohesion: 0.09
Nodes (11): cut_feed(), pack_size(), PendingReconf, Reconf, Resources, ShareSession, Sink, SinkTx (+3 more)

### Community 28 - "steps"
Cohesion: 0.08
Nodes (32): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+24 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (assistir e transmitir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "hdr.rs"
Cohesion: 0.05
Nodes (10): compile_shader(), DEFAULT_SDR_WHITE_NITS, get_monitor_gdi_name(), HdrConverter, HLSL_SOURCE, linear_to_srgb(), linear_to_srgb_properties(), query_sdr_white_level() (+2 more)

### Community 32 - "wgc.rs"
Cohesion: 0.15
Nodes (4): CountSink, FirstFrameSink, FrameSink, SinkHandle

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.07
Nodes (58): AbortOnDrop, __android_log_write(), audio_loop(), get_host(), get_session(), HostSession, init_logging(), INIT_LOGGING (+50 more)

### Community 35 - "Native"
Cohesion: 0.09
Nodes (5): Amigos e presença, Chat, Regras, Native, Presence

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 37 - "Updater.kt"
Cohesion: 0.08
Nodes (11): Android (`android/`), GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle (+3 more)

### Community 38 - "App"
Cohesion: 0.11
Nodes (3): App, own_hwnd(), Frame

### Community 39 - "viewer.rs"
Cohesion: 0.10
Nodes (10): BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64, CLOSE_KICKED_U64, CLOSE_PROTOCOL_U64, close_reason(), CLOSE_STOPPED_U64, run() (+2 more)

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
Cohesion: 0.10
Nodes (7): border_removable(), DEFAULT_PIXEL_FORMAT, handle_frame(), property_present(), SESSION_TYPE, start(), WgcCapture

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.24
Nodes (4): AbortJoinSet<T>, Target, Friend, Invite

### Community 50 - "e2e_window.rs"
Cohesion: 0.18
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 51 - "SourceId"
Cohesion: 0.16
Nodes (14): device_name(), Dib, enumerate(), fit(), monitor_cb(), monitor_info(), monitors(), primary_monitor() (+6 more)

### Community 52 - "Theme.kt"
Cohesion: 0.18
Nodes (3): inter(), LiveScreenTheme(), Sinal

### Community 53 - "preset.rs"
Cohesion: 0.11
Nodes (29): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), is_reduction(), label(), light_presets_are_reductions_and_fall_back() (+21 more)

### Community 54 - "OutputResult"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 55 - "parse_invite"
Cohesion: 0.27
Nodes (8): find_invite(), find_invite_in_pasted_text(), format_invite(), new_room_key(), parse_invite(), is_valid_invite(), input(), normalize_ticket()

### Community 58 - "BudgetExceeded"
Cohesion: 0.11
Nodes (24): ROOM_KEY_LEN, host_ws_url(), NONCE_LEN, OFFER_VEC, open_offer(), open_with(), SALA, seal_answer() (+16 more)

### Community 59 - "Resources"
Cohesion: 0.14
Nodes (10): HostStats, configure(), CpuCopy, create_mft(), Encoder, Nv12Frame, read_seq_header(), spawn() (+2 more)

### Community 61 - "Codec"
Cohesion: 0.09
Nodes (21): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+13 more)

### Community 62 - "run"
Cohesion: 0.26
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 63 - "host/mod.rs"
Cohesion: 0.12
Nodes (11): Abr, ABR_FLOOR, decide_target_fps(), HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), MAX_HANDSHAKES (+3 more)

### Community 64 - "super"
Cohesion: 0.10
Nodes (18): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+10 more)

### Community 65 - "HomeScreen"
Cohesion: 0.22
Nodes (16): AddFriendDialog(), App(), Badge(), CallButton(), ChatSheet(), clipboardText(), FriendsSection(), HomeScreen() (+8 more)

### Community 66 - "playback.rs"
Cohesion: 0.14
Nodes (10): conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, output_loop(), PlaybackHandle, PREBUFFER_FLOATS (+2 more)

### Community 67 - "DxgiCapture"
Cohesion: 0.09
Nodes (15): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Navegador (web) (+7 more)

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
Nodes (35): addOpusStereo(), appendChat(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect(), connectTips() (+27 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "src/chat.rs"
Cohesion: 0.13
Nodes (13): CHAT_HISTORY, ChatLine, ChatLog, clean_chat(), DONATE_AMOUNTS, HOST_SENDER, incoming_donation_rules(), labels_never_impersonate_host() (+5 more)

### Community 77 - "ToastKind"
Cohesion: 0.29
Nodes (5): ToastKind, Info, Success, Warn, Toasts

### Community 78 - "sharing"
Cohesion: 0.20
Nodes (17): ShareStatus, viewers_for(), Upload, Done, Failed, Measuring, Unknown, chat_card() (+9 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 83 - "run"
Cohesion: 0.22
Nodes (5): base_url(), drain_outputs(), init_crypto(), run(), super::Host

### Community 84 - "home.rs"
Cohesion: 0.16
Nodes (12): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+4 more)

### Community 85 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 86 - "ui/friends.rs"
Cohesion: 0.07
Nodes (15): ChatRole, Host, Viewer, panel(), Draft, MAX_LIST, I_ADD_FRIEND, I_CIRCLE_CHECK (+7 more)

### Community 87 - "peer"
Cohesion: 0.12
Nodes (13): browser_predictions(), browser_predictions_follow_first_v4_srflx(), default_v4(), forbidden_dest(), local_v6(), peer(), room_id(), SignalOut (+5 more)

### Community 89 - "SinalSnack"
Cohesion: 0.40
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 90 - "capture/mod.rs"
Cohesion: 0.28
Nodes (9): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+1 more)

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

### Community 104 - "read_video_frame"
Cohesion: 0.28
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 105 - "SourceId"
Cohesion: 0.28
Nodes (8): detect_hdr(), find_output_for_monitor(), HdrInfo, hmonitor_for_source(), is_output_hdr(), SourceId, Monitor, Window

### Community 108 - "frame_from_body"
Cohesion: 0.29
Nodes (4): frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 110 - "write_msg"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 114 - "room_proof"
Cohesion: 0.40
Nodes (3): invite_roundtrip_and_proof_binding(), room_proof(), verify_room_proof()

### Community 115 - "WatchStatus"
Cohesion: 0.40
Nodes (5): Phase, Connecting, Ended, Playing, WatchStatus

## Knowledge Gaps
- **119 isolated node(s):** `livescreen`, `Primary`, `Secondary`, `Ghost`, `Danger` (+114 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 898 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `super`, `watch_view.rs`, `settings.rs`, `SourceId`, `ToastKind`, `sharing`, `update.rs`, `Screen`, `home.rs`, `parse_invite`, `ui/friends.rs`, `ui/mod.rs`, `share_view.rs`, `WatchSession`, `share.rs`, `Codec`?**
  _High betweenness centrality (0.050) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Primary`, `Secondary` to the rest of the system?**
  _119 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13090418353576247 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `ViewerModel` to `HomeScreen`, `Hosting.kt`, `Native`, `Updater.kt`, `MainActivity.kt`, `AudioPlayer`, `VideoDecoder`?**
  _High betweenness centrality (0.048) - this node is a cross-community bridge._
- **Should `watch_view.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0812807881773399 - nodes in this community are weakly interconnected._
- **Why does `Regras` connect `Native` to `DxgiCapture`, `ViewerModel`, `.start`, `share.rs`, `HostService.kt`?**
  _High betweenness centrality (0.048) - this node is a cross-community bridge._