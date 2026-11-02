# Graph Report - LiveScreen-publish  (2026-11-02)

## Corpus Check
- 82 files · ~110,892 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 724 file(s) not represented in the graph (top: (none) 642, .xml 25, .sample 14)

## Summary
- 2525 nodes · 5236 edges · 115 communities (85 shown, 30 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 156 edges (avg confidence: 0.78)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- decoder.rs
- src/avatar.rs
- Hosting.kt
- Shared
- ToastKind
- convert.rs
- App
- sources.rs
- playback.rs
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
- Avatar.kt
- test-changelog.sh
- rust/src/lib.rs
- Native
- src/friends.rs
- ui/avatar.rs
- viewer.rs
- main.rs
- present.rs
- Screens.kt
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- wgc.rs
- codec_roundtrip.rs
- e2e_window.rs
- Converter
- capture/mod.rs
- preset.rs
- OutputResult
- parse_invite
- WatchSession
- settings.rs
- BudgetExceeded
- Resources
- Callback
- Codec
- Ink
- host/mod.rs
- super
- HomeScreen
- AudioEncoder
- DxgiCapture
- enumerate_mfts
- variant_with
- ScreenEncoder
- handle_frame
- MainActivity.kt
- Callback
- HostMsg
- src/chat.rs
- Android (`android/`)
- sharing
- pages.yml
- run
- home.rs
- BtnKind
- watch_view.rs
- peer
- .auto
- UpdateInfo
- Mic
- EncodedFrame
- capture_loop
- AudioState
- CallState
- FieldTone
- ViewerMsg
- VideoPts
- SocketAddr
- SignalOut
- BudgetExceeded
- VideoDecoder
- ID3D11Device
- AudioHandle
- ChatLimiter
- ui/friends.rs
- EndpointId
- AudioTarget
- LiveScreen
- run
- Target
- SignalIn
- AudioPlayer.kt
- Interface

## God Nodes (most connected - your core abstractions)
1. `App` - 74 edges
2. `Preset` - 53 edges
3. `ViewerModel` - 46 edges
4. `Native` - 43 edges
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

## Communities (115 total, 30 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.13
Nodes (20): EncodedFrame, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size() (+12 more)

### Community 1 - "src/avatar.rs"
Cohesion: 0.07
Nodes (33): ALL, all(), auto(), Avatar, BACK, COELHO_SHAPES, COUNT, DARK (+25 more)

### Community 2 - "Hosting.kt"
Cohesion: 0.07
Nodes (7): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull(), ChatItem, parseChatItems()

### Community 3 - "Shared"
Cohesion: 0.14
Nodes (9): admit(), answer_presence(), CountGuard, handle_viewer(), Peer, PeerConn, Iroh, Web (+1 more)

### Community 4 - "ToastKind"
Cohesion: 0.14
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 6 - "App"
Cohesion: 0.10
Nodes (4): App, classify(), own_hwnd(), Frame

### Community 8 - "sources.rs"
Cohesion: 0.05
Nodes (18): device_name(), Dib, enumerate(), exe_name(), fit(), inspect_window(), monitor_cb(), monitor_info() (+10 more)

### Community 9 - "playback.rs"
Cohesion: 0.12
Nodes (10): conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, output_loop(), PlaybackHandle, PREBUFFER_FLOATS (+2 more)

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
Cohesion: 0.13
Nodes (4): Friend, Presence, Status, ViewerModel

### Community 15 - "speedtest.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.04
Nodes (65): app_icon(), badge(), button(), button_full(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_button() (+57 more)

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
Nodes (4): Feeds, Host, Overload, rtt_congested()

### Community 21 - "with"
Cohesion: 0.06
Nodes (38): runs-on, steps, GIT_CLIFF_TAG_PATTERN, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release (+30 more)

### Community 23 - "capture.rs"
Cohesion: 0.12
Nodes (11): FRAME_BYTES, FRAME_FLOATS, FRAME_SAMPLES, MAX_LAG_BYTES, mix(), mix_clamps(), OPUS_MAX_PACKET, take_frame() (+3 more)

### Community 24 - "share_view.rs"
Cohesion: 0.11
Nodes (14): SourceInfo, AUTO_QUALITY_COOLDOWN, begin_start(), Picker, poll_starting(), Preview, source_card(), spawn_share_job() (+6 more)

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (15): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+7 more)

### Community 26 - ".start"
Cohesion: 0.22
Nodes (3): Hosting, Callback, Run

### Community 27 - "share.rs"
Cohesion: 0.07
Nodes (16): WebState, Connecting, Off, Online, Retrying, cut_feed(), pack_size(), PendingReconf (+8 more)

### Community 28 - "steps"
Cohesion: 0.08
Nodes (32): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+24 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (assistir e transmitir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "hdr.rs"
Cohesion: 0.05
Nodes (15): compile_shader(), DEFAULT_SDR_WHITE_NITS, detect_hdr(), find_output_for_monitor(), get_monitor_gdi_name(), HdrConverter, HdrInfo, HLSL_SOURCE (+7 more)

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.07
Nodes (59): AbortOnDrop, __android_log_write(), audio_loop(), get_host(), get_session(), HostSession, init_logging(), INIT_LOGGING (+51 more)

### Community 35 - "Native"
Cohesion: 0.09
Nodes (4): Amigos e presença, Chat, Regras, Native

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 38 - "ui/avatar.rs"
Cohesion: 0.14
Nodes (6): avatar(), color_from_u32(), draw_avatar_item(), ink_color(), paint_avatar(), picker()

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
Cohesion: 0.03
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "wgc.rs"
Cohesion: 0.09
Nodes (10): SourceId, Monitor, Window, border_removable(), DEFAULT_PIXEL_FORMAT, handle_frame(), property_present(), SESSION_TYPE (+2 more)

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.20
Nodes (5): AbortJoinSet<T>, Target, Friend, Invite, verify_room_proof()

### Community 50 - "e2e_window.rs"
Cohesion: 0.18
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 52 - "capture/mod.rs"
Cohesion: 0.11
Nodes (13): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, CountSink, FirstFrameSink (+5 more)

### Community 53 - "preset.rs"
Cohesion: 0.11
Nodes (29): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), is_reduction(), label(), light_presets_are_reductions_and_fall_back() (+21 more)

### Community 54 - "OutputResult"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 55 - "parse_invite"
Cohesion: 0.23
Nodes (10): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), is_valid_invite() (+2 more)

### Community 57 - "settings.rs"
Cohesion: 0.13
Nodes (28): button(), diagnostics(), experimental(), FORCED, hint(), icon_button(), open_help(), open_logs() (+20 more)

### Community 58 - "BudgetExceeded"
Cohesion: 0.11
Nodes (23): ROOM_KEY_LEN, lenient_u8(), NONCE_LEN, OFFER_VEC, open_offer(), open_with(), SALA, seal_answer() (+15 more)

### Community 59 - "Resources"
Cohesion: 0.19
Nodes (6): CpuCopy, Encoder, Feed, Idle, Stop, Nv12Frame

### Community 61 - "Codec"
Cohesion: 0.10
Nodes (17): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+9 more)

### Community 62 - "Ink"
Cohesion: 0.19
Nodes (15): AvatarData, AvatarDef, AvatarShape, Circle, Ellipse, Ink, Accent, Back (+7 more)

### Community 63 - "host/mod.rs"
Cohesion: 0.10
Nodes (13): Abr, ABR_FLOOR, decide_target_fps(), HANDSHAKE_TIMEOUT, HostStats, ids(), invite_addr(), invite_addr_keeps_only_relay() (+5 more)

### Community 64 - "super"
Cohesion: 0.09
Nodes (19): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+11 more)

### Community 65 - "HomeScreen"
Cohesion: 0.20
Nodes (16): AddFriendDialog(), App(), Badge(), CallButton(), ChatSheet(), clipboardText(), FriendsSection(), HomeScreen() (+8 more)

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

### Community 72 - "MainActivity.kt"
Cohesion: 0.08
Nodes (4): MainActivity, inter(), LiveScreenTheme(), Sinal

### Community 73 - "Callback"
Cohesion: 0.09
Nodes (44): addOpusStereo(), appendChat(), autoAvatar(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect() (+36 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "src/chat.rs"
Cohesion: 0.10
Nodes (18): CHAT_HISTORY, chat_line_avatar_resolution_and_serde(), ChatLine, ChatLog, ChatRole, Host, Viewer, clean_chat() (+10 more)

### Community 77 - "Android (`android/`)"
Cohesion: 0.27
Nodes (4): Android (`android/`), GhAsset, Updater, UpdaterTest

### Community 78 - "sharing"
Cohesion: 0.19
Nodes (18): ShareStatus, viewers_for(), Upload, Done, Failed, Measuring, Unknown, chat_card() (+10 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 83 - "run"
Cohesion: 0.18
Nodes (7): base_url(), drain_outputs(), init_crypto(), run(), super::Host, host_ws_url(), test_host_ws_url()

### Community 84 - "home.rs"
Cohesion: 0.17
Nodes (12): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+4 more)

### Community 85 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 86 - "watch_view.rs"
Cohesion: 0.07
Nodes (23): CallTone, Active, Alert, Neutral, Plain, I_CHAT, I_EXIT_FULLSCREEN, I_FULLSCREEN (+15 more)

### Community 87 - "peer"
Cohesion: 0.13
Nodes (8): browser_predictions(), browser_predictions_follow_first_v4_srflx(), default_v4(), forbidden_dest(), local_v6(), peer(), WebPeer, room_id()

### Community 89 - "UpdateInfo"
Cohesion: 0.36
Nodes (7): UpdateInfo, Available, Downloading, Failed, Idle, Ready, UpdateState

### Community 90 - "Mic"
Cohesion: 0.22
Nodes (7): Mic, Default, Device, open(), open_mic(), pump(), Source

### Community 91 - "EncodedFrame"
Cohesion: 0.18
Nodes (7): configure(), create_mft(), FAST_REPEATS, read_seq_header(), spawn(), stream_ids(), vbv_bytes()

### Community 92 - "capture_loop"
Cohesion: 0.50
Nodes (6): AudioRequest, capture_loop(), mic_error_text(), Opened, run(), start()

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

### Community 100 - "SignalOut"
Cohesion: 0.33
Nodes (6): SignalOut, Answer, Close, Host, Ping, Reject

### Community 103 - "ID3D11Device"
Cohesion: 0.24
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 106 - "ui/friends.rs"
Cohesion: 0.07
Nodes (16): Draft, MAX_LIST, I_ADD_FRIEND, I_CIRCLE_CHECK, I_COPY, I_INFO, I_PEOPLE, I_TRASH (+8 more)

### Community 108 - "AudioTarget"
Cohesion: 0.50
Nodes (4): AudioTarget, Process, System, open_loopback()

### Community 109 - "LiveScreen"
Cohesion: 0.18
Nodes (10): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+2 more)

### Community 111 - "Target"
Cohesion: 0.67
Nodes (3): Target, Nv12, Rgb

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 118 - "Interface"
Cohesion: 0.33
Nodes (4): is_physical(), looks_virtual(), route_info(), RouteInfo

## Knowledge Gaps
- **134 isolated node(s):** `livescreen`, `Accent`, `Dark`, `Light`, `Back` (+129 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 921 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Regras` connect `Native` to `DxgiCapture`, `LiveScreen`, `.start`, `share.rs`, `HostService.kt`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Accent`, `Dark` to the rest of the system?**
  _134 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1337126600284495 - nodes in this community are weakly interconnected._
- **Why does `Preset` connect `ShareSession` to `ViewerMsg`, `super`, `rust/src/lib.rs`, `Shared`, `viewer.rs`, `sharing`, `Host`, `preset.rs`, `share_view.rs`, `settings.rs`, `share.rs`, `host/mod.rs`?**
  _High betweenness centrality (0.054) - this node is a cross-community bridge._
- **Should `src/avatar.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07152496626180836 - nodes in this community are weakly interconnected._
- **Why does `App` connect `App` to `super`, `ui/friends.rs`, `update.rs`, `sharing`, `wgc.rs`, `Screen`, `home.rs`, `parse_invite`, `ui/mod.rs`, `watch_view.rs`, `share_view.rs`, `settings.rs`, `WatchSession`, `share.rs`, `Codec`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._