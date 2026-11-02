# Graph Report - LiveScreen-publish  (2026-11-01)

## Corpus Check
- 82 files · ~110,652 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 55 file(s) not represented in the graph (top: .xml 25, .woff2 10, .ttf 7)

## Summary
- 2522 nodes · 5233 edges · 121 communities (94 shown, 27 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 156 edges (avg confidence: 0.78)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `fa4342c2`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- src/avatar.rs
- Hosting.kt
- Shared
- ToastKind
- convert.rs
- App
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
- Avatar.kt
- test-changelog.sh
- rust/src/lib.rs
- Native
- src/friends.rs
- Updater.kt
- ui/avatar.rs
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
- capture/mod.rs
- preset.rs
- OutputResult
- format_invite
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
- App
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
- watch_view.rs
- peer
- .auto
- SinalSnack
- capture/mod.rs
- EncodedFrame
- WebState
- AudioState
- CallState
- FieldTone
- ViewerMsg
- VideoPts
- SocketAddr
- ChatLine
- BudgetExceeded
- AudioEncoder
- ID3D11Device
- read_video_frame
- Stroke
- ui/friends.rs
- EndpointId
- Prober
- LiveScreen
- run
- WebPeer
- SignalIn
- AudioPlayer
- SourceId
- frame_from_body
- Tab
- PresenceState
- Interface
- GpuOut
- write_msg

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

## Communities (121 total, 27 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.15
Nodes (16): as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size(), Gpu (+8 more)

### Community 1 - "src/avatar.rs"
Cohesion: 0.07
Nodes (33): ALL, all(), auto(), Avatar, BACK, COELHO_SHAPES, COUNT, DARK (+25 more)

### Community 2 - "Hosting.kt"
Cohesion: 0.07
Nodes (7): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull(), ChatItem, parseChatItems()

### Community 3 - "Shared"
Cohesion: 0.14
Nodes (5): admit(), CountGuard, Feeds, handle_viewer(), Shared

### Community 4 - "ToastKind"
Cohesion: 0.15
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "App"
Cohesion: 0.10
Nodes (3): App, classify(), own_hwnd()

### Community 8 - "sources.rs"
Cohesion: 0.05
Nodes (7): enumerate(), exe_name(), inspect_window(), monitors(), PW_RENDERFULLCONTENT, SourceInfo, window_cb()

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
Cohesion: 0.09
Nodes (30): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+22 more)

### Community 14 - "ViewerModel"
Cohesion: 0.11
Nodes (3): Friend, Status, ViewerModel

### Community 15 - "speedtest.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.06
Nodes (34): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), call_title_with_avatar(), card() (+26 more)

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
Cohesion: 0.11
Nodes (4): Host, invite_addr(), Overload, rtt_congested()

### Community 21 - "with"
Cohesion: 0.06
Nodes (38): runs-on, steps, GIT_CLIFF_TAG_PATTERN, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release (+30 more)

### Community 23 - "capture.rs"
Cohesion: 0.08
Nodes (30): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+22 more)

### Community 24 - "share_view.rs"
Cohesion: 0.09
Nodes (20): AUTO_QUALITY_COOLDOWN, begin_start(), Picker, picker(), poll_starting(), Preview, source_card(), Starting (+12 more)

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (16): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+8 more)

### Community 26 - ".start"
Cohesion: 0.15
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

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.07
Nodes (59): AbortOnDrop, __android_log_write(), audio_loop(), get_host(), get_session(), HostSession, init_logging(), INIT_LOGGING (+51 more)

### Community 35 - "Native"
Cohesion: 0.10
Nodes (5): Amigos e presença, Chat, Regras, Native, Presence

### Community 36 - "src/friends.rs"
Cohesion: 0.15
Nodes (16): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), Presence, PRESENCE_ALPN (+8 more)

### Community 37 - "Updater.kt"
Cohesion: 0.08
Nodes (11): Android (`android/`), GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle (+3 more)

### Community 38 - "ui/avatar.rs"
Cohesion: 0.13
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

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "wgc.rs"
Cohesion: 0.11
Nodes (7): border_removable(), DEFAULT_PIXEL_FORMAT, handle_frame(), property_present(), SESSION_TYPE, start(), WgcCapture

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.19
Nodes (6): AbortJoinSet<T>, parse_invite(), Target, Friend, Invite, is_valid_invite()

### Community 50 - "e2e_window.rs"
Cohesion: 0.18
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 51 - "SourceId"
Cohesion: 0.18
Nodes (12): device_name(), Dib, fit(), monitor_cb(), monitor_info(), primary_monitor(), refresh_rate(), source_size() (+4 more)

### Community 52 - "capture/mod.rs"
Cohesion: 0.15
Nodes (5): CountSink, FirstFrameSink, FrameSink, selftest(), SinkHandle

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (28): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), is_reduction(), label(), light_presets_are_reductions_and_fall_back() (+20 more)

### Community 54 - "OutputResult"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 55 - "format_invite"
Cohesion: 0.25
Nodes (6): find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), room_proof(), verify_room_proof()

### Community 57 - "settings.rs"
Cohesion: 0.20
Nodes (20): button(), diagnostics(), experimental(), FORCED, hint(), icon_button(), open_help(), open_logs() (+12 more)

### Community 58 - "BudgetExceeded"
Cohesion: 0.11
Nodes (24): ROOM_KEY_LEN, host_ws_url(), NONCE_LEN, OFFER_VEC, open_offer(), open_with(), SALA, seal_answer() (+16 more)

### Community 59 - "Resources"
Cohesion: 0.18
Nodes (7): CpuCopy, Encoder, Feed, Frame, Idle, Stop, Nv12Frame

### Community 61 - "Codec"
Cohesion: 0.10
Nodes (18): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+10 more)

### Community 62 - "Ink"
Cohesion: 0.19
Nodes (15): AvatarData, AvatarDef, AvatarShape, Circle, Ellipse, Ink, Accent, Back (+7 more)

### Community 63 - "host/mod.rs"
Cohesion: 0.09
Nodes (16): Abr, ABR_FLOOR, answer_presence(), decide_target_fps(), HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips() (+8 more)

### Community 64 - "super"
Cohesion: 0.06
Nodes (29): conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, output_loop(), PlaybackHandle, PREBUFFER_FLOATS (+21 more)

### Community 65 - "HomeScreen"
Cohesion: 0.20
Nodes (16): AddFriendDialog(), App(), Badge(), CallButton(), ChatSheet(), clipboardText(), FriendsSection(), HomeScreen() (+8 more)

### Community 66 - "App"
Cohesion: 0.20
Nodes (6): parse_friend_code(), add_friend_modal(), App, card(), presence_of(), save()

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
Cohesion: 0.13
Nodes (13): CHAT_HISTORY, chat_line_avatar_resolution_and_serde(), ChatLimiter, clean_chat(), DONATE_AMOUNTS, HOST_SENDER, incoming_donation_rules(), labels_never_impersonate_host() (+5 more)

### Community 77 - "ToastKind"
Cohesion: 0.25
Nodes (6): spawn_share_job(), ToastKind, Info, Success, Warn, Toasts

### Community 78 - "sharing"
Cohesion: 0.18
Nodes (18): ShareStatus, viewers_for(), Upload, Done, Failed, Measuring, Unknown, chat_card() (+10 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 83 - "run"
Cohesion: 0.22
Nodes (5): base_url(), drain_outputs(), init_crypto(), run(), super::Host

### Community 84 - "home.rs"
Cohesion: 0.09
Nodes (17): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+9 more)

### Community 85 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 86 - "watch_view.rs"
Cohesion: 0.08
Nodes (21): find_invite(), I_CHAT, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON (+13 more)

### Community 87 - "peer"
Cohesion: 0.12
Nodes (13): browser_predictions(), browser_predictions_follow_first_v4_srflx(), default_v4(), forbidden_dest(), local_v6(), peer(), room_id(), SignalOut (+5 more)

### Community 89 - "SinalSnack"
Cohesion: 0.40
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 90 - "capture/mod.rs"
Cohesion: 0.30
Nodes (8): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), start()

### Community 91 - "EncodedFrame"
Cohesion: 0.22
Nodes (8): HostStats, EncodedFrame, configure(), create_mft(), read_seq_header(), spawn(), stream_ids(), vbv_bytes()

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

### Community 100 - "ChatLine"
Cohesion: 0.19
Nodes (7): ChatLine, ChatLog, ChatRole, Host, Viewer, log_caps_and_since(), panel()

### Community 102 - "AudioEncoder"
Cohesion: 0.07
Nodes (3): AudioEncoder, CodecState, VideoDecoder

### Community 103 - "ID3D11Device"
Cohesion: 0.27
Nodes (4): Composer, Grab, GrabSink, texture()

### Community 104 - "read_video_frame"
Cohesion: 0.20
Nodes (6): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_msg(), read_video_frame()

### Community 106 - "ui/friends.rs"
Cohesion: 0.15
Nodes (8): Draft, Friend, load(), MAX_LIST, I_ADD_FRIEND, I_COPY, I_PEOPLE, I_TRASH

### Community 109 - "LiveScreen"
Cohesion: 0.18
Nodes (10): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+2 more)

### Community 110 - "run"
Cohesion: 0.31
Nodes (5): create_duplication(), dxgi_draws_cursor(), DxgiCapture, run(), start()

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 114 - "SourceId"
Cohesion: 0.28
Nodes (8): detect_hdr(), find_output_for_monitor(), HdrInfo, hmonitor_for_source(), is_output_hdr(), SourceId, Monitor, Window

### Community 115 - "frame_from_body"
Cohesion: 0.29
Nodes (4): frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 116 - "Tab"
Cohesion: 0.29
Nodes (7): Tab, Diagnostics, Experimental, Share, Updates, Video, TABS

### Community 117 - "PresenceState"
Cohesion: 0.33
Nodes (6): PresenceState, Live, NotFriend, Offline, Removed, Unknown

### Community 118 - "Interface"
Cohesion: 0.33
Nodes (4): is_physical(), looks_virtual(), route_info(), RouteInfo

### Community 119 - "GpuOut"
Cohesion: 0.40
Nodes (3): GpuOut, Readback, Shared

## Knowledge Gaps
- **134 isolated node(s):** `livescreen`, `Accent`, `Dark`, `Light`, `Back` (+129 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 919 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **27 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Regras` connect `Native` to `DxgiCapture`, `LiveScreen`, `ViewerModel`, `.start`, `share.rs`, `HostService.kt`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Accent`, `Dark` to the rest of the system?**
  _134 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1497326203208556 - nodes in this community are weakly interconnected._
- **Why does `Preset` connect `ShareSession` to `ViewerMsg`, `super`, `rust/src/lib.rs`, `Shared`, `viewer.rs`, `sharing`, `Host`, `preset.rs`, `share_view.rs`, `share.rs`, `host/mod.rs`?**
  _High betweenness centrality (0.050) - this node is a cross-community bridge._
- **Should `src/avatar.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07152496626180836 - nodes in this community are weakly interconnected._
- **Why does `App` connect `App` to `super`, `ui/friends.rs`, `ToastKind`, `sharing`, `update.rs`, `Screen`, `SourceId`, `home.rs`, `Tab`, `ui/mod.rs`, `watch_view.rs`, `share_view.rs`, `settings.rs`, `WatchSession`, `share.rs`, `Codec`?**
  _High betweenness centrality (0.049) - this node is a cross-community bridge._