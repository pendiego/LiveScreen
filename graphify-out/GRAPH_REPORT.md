# Graph Report - LiveScreen-publish  (2026-10-30)

## Corpus Check
- 83 files · ~99,902 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 60 file(s) not represented in the graph (top: .xml 24, .ttf 12, .woff2 10)

## Summary
- 2389 nodes · 4931 edges · 109 communities (87 shown, 22 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 150 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `81fd6620`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- Ui
- Hosting.kt
- host/mod.rs
- ToastKind
- convert.rs
- settings.rs
- sources.rs
- gsap.min.js
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
- VideoDecoder
- share_view.rs
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
- App
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
- encoder.rs
- run
- capture.rs
- config.rs
- HomeScreen
- LiveScreen
- DxgiCapture
- Tween
- read_video_frame
- AudioEncoder
- handle_frame
- e2e_window.rs
- Callback
- HostMsg
- src/chat.rs
- home.rs
- sharing
- pages.yml
- ScreenEncoder
- build.mjs
- toast.rs
- ce
- SignalOut
- DxgiCapture
- BtnKind
- ids
- na
- WebState
- la
- CallState
- FieldTone
- fb
- run
- SinalSnack
- he
- frame_from_body
- BudgetExceeded
- write_msg
- Za
- Rd
- Áudio LiveScreen v0.6.0
- Capturas reais do LiveScreen 0.6.0
- AudioState

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

## Communities (109 total, 22 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.12
Nodes (19): as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size(), Gpu (+11 more)

### Community 1 - "Ui"
Cohesion: 0.13
Nodes (26): button(), button_full(), call_button(), CallTone, Active, Alert, Neutral, Plain (+18 more)

### Community 2 - "Hosting.kt"
Cohesion: 0.07
Nodes (7): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull(), ChatItem, parseChatItems()

### Community 3 - "host/mod.rs"
Cohesion: 0.08
Nodes (16): Abr, ABR_FLOOR, admit(), answer_presence(), CountGuard, Feeds, handle_viewer(), HANDSHAKE_TIMEOUT (+8 more)

### Community 4 - "ToastKind"
Cohesion: 0.11
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "settings.rs"
Cohesion: 0.13
Nodes (26): button(), diagnostics(), FORCED, hint(), icon_button(), open_help(), open_logs(), SETTINGS_LABEL (+18 more)

### Community 9 - "gsap.min.js"
Cohesion: 0.07
Nodes (15): Gc(), Hc(), ia(), ja(), Lc(), Nc(), oa(), pa() (+7 more)

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
Cohesion: 0.11
Nodes (3): Friend, Status, ViewerModel

### Community 15 - "playback.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (36): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), card(), chevron_icon() (+28 more)

### Community 17 - "share_view.rs"
Cohesion: 0.06
Nodes (20): ChatRole, Host, Viewer, panel(), Draft, MAX_LIST, I_ADD_FRIEND, I_CHAT (+12 more)

### Community 18 - "gpu.rs"
Cohesion: 0.08
Nodes (3): create_device(), Sendable, winrt_device()

### Community 19 - "host/web.rs"
Cohesion: 0.06
Nodes (10): rtt_congested(), CONNECT_TIMEOUT, drain_outputs(), INIT_CRYPTO, MAX_SDP, MAX_WEB_CHAT, peer(), STUN_SERVER (+2 more)

### Community 20 - "Host"
Cohesion: 0.12
Nodes (3): Host, invite_addr(), Overload

### Community 21 - "with"
Cohesion: 0.06
Nodes (34): runs-on, steps, GIT_CLIFF_TAG_PATTERN, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release (+26 more)

### Community 24 - "share_view.rs"
Cohesion: 0.08
Nodes (18): AUTO_QUALITY_COOLDOWN, begin_start(), Picker, poll_starting(), Preview, Starting, Thumb, to_image() (+10 more)

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (19): HostSession, EncodeParams, Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60 (+11 more)

### Community 26 - ".start"
Cohesion: 0.20
Nodes (3): Hosting, Callback, Run

### Community 27 - "share.rs"
Cohesion: 0.08
Nodes (13): is_reduction(), FrameSink, cut_feed(), pack_size(), PendingReconf, Reconf, Resources, ShareSession (+5 more)

### Community 28 - "steps"
Cohesion: 0.09
Nodes (29): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+21 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (assistir e transmitir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "encoder.rs"
Cohesion: 0.14
Nodes (17): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+9 more)

### Community 32 - "wgc.rs"
Cohesion: 0.10
Nodes (7): border_removable(), handle_frame(), PIXEL_FORMAT, property_present(), SESSION_TYPE, start(), WgcCapture

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.07
Nodes (58): AbortOnDrop, __android_log_write(), audio_loop(), get_host(), get_session(), init_logging(), INIT_LOGGING, Java_io_github_pendiego_livescreen_Native_chatSince() (+50 more)

### Community 35 - "Native"
Cohesion: 0.10
Nodes (5): Amigos e presença, Chat, Regras, Native, Presence

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 37 - "Updater.kt"
Cohesion: 0.08
Nodes (11): Android (`android/`), GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle (+3 more)

### Community 38 - "App"
Cohesion: 0.07
Nodes (12): App, classify(), own_hwnd(), connect(), ControlAction, Leave, None, ToggleFullscreen (+4 more)

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
Cohesion: 0.21
Nodes (13): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+5 more)

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.20
Nodes (5): AbortJoinSet<T>, Target, Friend, Invite, verify_room_proof()

### Community 50 - "Native"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 51 - "SourceId"
Cohesion: 0.20
Nodes (12): device_name(), enumerate(), exe_name(), inspect_window(), monitor_cb(), monitor_info(), monitors(), primary_monitor() (+4 more)

### Community 52 - "Theme.kt"
Cohesion: 0.18
Nodes (3): inter(), LiveScreenTheme(), Sinal

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (26): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not(), LIMIT_FPS (+18 more)

### Community 54 - "Dib"
Cohesion: 0.33
Nodes (5): Dib, fit(), thumbnail(), thumbnail_monitor(), thumbnail_window()

### Community 55 - "format_invite"
Cohesion: 0.23
Nodes (10): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), is_valid_invite() (+2 more)

### Community 56 - "viewer.rs"
Cohesion: 0.05
Nodes (23): ViewerMsg, Chat, Hello, RequestKeyframe, SetQuality, SetVisible, BUDGET_MSG, CLOSE_DENIED_U64 (+15 more)

### Community 58 - "BudgetExceeded"
Cohesion: 0.08
Nodes (20): ROOM_KEY_LEN, host_ws_url(), sdp_fingerprint(), SignalIn, Left, Offer, Pong, Ready (+12 more)

### Community 59 - "Resources"
Cohesion: 0.19
Nodes (6): CpuCopy, Encoder, Feed, Idle, Stop, Nv12Frame

### Community 61 - "encoder.rs"
Cohesion: 0.16
Nodes (7): HostStats, configure(), create_mft(), read_seq_header(), spawn(), stream_ids(), vbv_bytes()

### Community 62 - "run"
Cohesion: 0.27
Nodes (4): Composer, Grab, GrabSink, texture()

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
Cohesion: 0.18
Nodes (10): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+2 more)

### Community 67 - "DxgiCapture"
Cohesion: 0.19
Nodes (5): Navegador (web), bytesToHex(), hexToBytes(), iceServers(), Room

### Community 68 - "Tween"
Cohesion: 0.18
Nodes (21): _a(), Ao(), _assertThisInitialized(), cb(), cc(), ga(), gb(), hb() (+13 more)

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
Cohesion: 0.15
Nodes (19): addOpusStereo(), appendChat(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect(), hexToBytes() (+11 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "src/chat.rs"
Cohesion: 0.11
Nodes (12): CHAT_HISTORY, ChatLimiter, ChatLine, ChatLog, clean_chat(), HOST_SENDER, labels_never_impersonate_host(), limiter_burst_and_refill() (+4 more)

### Community 77 - "home.rs"
Cohesion: 0.16
Nodes (12): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+4 more)

### Community 78 - "sharing"
Cohesion: 0.20
Nodes (17): ShareStatus, Upload, Done, Failed, Measuring, Unknown, chat_card(), confirm_modal() (+9 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 84 - "build.mjs"
Cohesion: 0.11
Nodes (13): assetsDir, audioDir, caps, dims, here, icons, lucide, missing (+5 more)

### Community 85 - "toast.rs"
Cohesion: 0.12
Nodes (11): spawn_share_job(), I_CIRCLE_CHECK, I_INFO, I_WARN, LIFETIME, MAX_VISIBLE, ToastKind, Info (+3 more)

### Community 86 - "ce"
Cohesion: 0.22
Nodes (14): Ae(), ce(), $d(), ee(), ha(), ka(), le(), me() (+6 more)

### Community 87 - "SignalOut"
Cohesion: 0.33
Nodes (6): SignalOut, Answer, Close, Host, Ping, Reject

### Community 88 - "DxgiCapture"
Cohesion: 0.26
Nodes (6): create_duplication(), dxgi_draws_cursor(), DxgiCapture, run(), start(), SinkHandle

### Community 89 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 90 - "ids"
Cohesion: 0.20
Nodes (5): HostConfig, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), ViewerInfo

### Community 91 - "na"
Cohesion: 0.25
Nodes (6): Aa(), Ca(), na(), Vb(), wb(), Xb()

### Community 92 - "WebState"
Cohesion: 0.40
Nodes (5): WebState, Connecting, Off, Online, Retrying

### Community 93 - "la"
Cohesion: 0.47
Nodes (6): Animation(), Da(), la(), ma(), Ua(), Va()

### Community 94 - "CallState"
Cohesion: 0.40
Nodes (5): CallState, Active, Alert, HangUp, Neutral

### Community 95 - "FieldTone"
Cohesion: 0.50
Nodes (4): FieldTone, Error, Normal, Warn

### Community 96 - "fb"
Cohesion: 0.40
Nodes (5): Context(), Db(), Eb(), fb(), Gw()

### Community 97 - "run"
Cohesion: 0.50
Nodes (4): base_url(), init_crypto(), run(), super::Host

### Community 98 - "SinalSnack"
Cohesion: 0.40
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 99 - "he"
Cohesion: 0.40
Nodes (5): he(), Md(), Nd(), Od(), Pd()

### Community 100 - "frame_from_body"
Cohesion: 0.36
Nodes (5): EncodedFrame, frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 102 - "write_msg"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 103 - "Za"
Cohesion: 0.40
Nodes (5): kb(), ob(), ra(), rb(), Za()

### Community 104 - "Rd"
Cohesion: 0.40
Nodes (5): Rd(), Vd(), Wd(), yd(), ye()

### Community 105 - "Áudio LiveScreen v0.6.0"
Cohesion: 0.50
Nodes (3): Linhas (início real da fala / fim real, em s) vs janela, Medições, Áudio LiveScreen v0.6.0

### Community 109 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

## Knowledge Gaps
- **118 isolated node(s):** `livescreen`, `Primary`, `Secondary`, `Ghost`, `Danger` (+113 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 894 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **22 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Preset` connect `ShareSession` to `config.rs`, `rust/src/lib.rs`, `host/mod.rs`, `settings.rs`, `sharing`, `Host`, `preset.rs`, `viewer.rs`, `share_view.rs`, `share.rs`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Primary`, `Secondary` to the rest of the system?**
  _118 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12280701754385964 - nodes in this community are weakly interconnected._
- **Why does `Regras` connect `Native` to `LiveScreen`, `DxgiCapture`, `ViewerModel`, `.start`, `share.rs`, `HostService.kt`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Should `Ui` be split into smaller, more focused modules?**
  _Cohesion score 0.12643678160919541 - nodes in this community are weakly interconnected._
- **Why does `App` connect `App` to `config.rs`, `settings.rs`, `home.rs`, `sharing`, `update.rs`, `capture/mod.rs`, `share_view.rs`, `Native`, `toast.rs`, `ui/mod.rs`, `format_invite`, `share_view.rs`, `viewer.rs`, `share.rs`, `encoder.rs`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._