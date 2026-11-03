# Graph Report - LiveScreen-publish  (2026-11-02)

## Corpus Check
- 95 files · ~138,220 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 56 file(s) not represented in the graph (top: .xml 25, .woff2 10, .ttf 7)

## Summary
- 2911 nodes · 5963 edges · 119 communities (93 shown, 26 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 170 edges (avg confidence: 0.78)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `9e0942aa`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- src/avatar.rs
- Hosting.kt
- host/mod.rs
- codec_roundtrip.rs
- convert.rs
- App
- sources.rs
- Android (`android/`)
- mf.rs
- protocol.rs
- steps
- update.rs
- ViewerModel
- speedtest.rs
- theme.rs
- AudioEncoder
- gpu.rs
- host/web.rs
- Host
- with
- LiveScreen
- page_check.rs
- share_view.rs
- ShareSession
- .start
- share.rs
- steps
- LiveScreen
- hdr.rs
- RFC: sinalização web v3 — ECDH efêmero + híbrido pós-quântico
- pq.js
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
- Target
- e2e_window.rs
- encoder.rs
- capture/mod.rs
- preset.rs
- ui/friends.rs
- WatchSession
- UpdateInfo
- settings.rs
- src/web.rs
- Mode
- Callback
- share.rs
- Ink
- clipboard.rs
- config.rs
- HomeScreen
- handle_viewer
- DxgiCapture
- capture.rs
- ChatLimiter
- ScreenEncoder
- signal3.js
- MainActivity.kt
- Callback
- HostMsg
- src/chat.rs
- ui/mod.rs
- sharing
- pages.yml
- run
- abytes
- BtnKind
- watch_view.rs
- mulAddUnsafe
- edwards
- start
- mulUnsafe
- AudioState
- CallState
- montgomery
- SocketAddr
- peer
- files
- VideoDecoder
- ID3D11Device
- .start
- read_video_frame
- ui/friends.rs
- EndpointId
- build.rs
- run
- SignalIn
- SourceId
- isPosBig
- duration
- bitReversalInplace
- AudioPlayer
- WebState
- ViewerSession
- WebVerifyStatus
- BudgetExceeded

## God Nodes (most connected - your core abstractions)
1. `App` - 74 edges
2. `Preset` - 54 edges
3. `ViewerModel` - 46 edges
4. `Native` - 43 edges
5. `Converter` - 33 edges
6. `Codec` - 31 edges
7. `Shared` - 30 edges
8. `SourceId` - 29 edges
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

## Communities (119 total, 26 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.13
Nodes (20): EncodedFrame, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size() (+12 more)

### Community 1 - "src/avatar.rs"
Cohesion: 0.07
Nodes (33): ALL, all(), auto(), Avatar, BACK, COELHO_SHAPES, COUNT, DARK (+25 more)

### Community 2 - "Hosting.kt"
Cohesion: 0.07
Nodes (7): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull(), ChatItem, parseChatItems()

### Community 3 - "host/mod.rs"
Cohesion: 0.09
Nodes (16): Abr, ABR_FLOOR, answer_presence(), decide_target_fps(), HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips() (+8 more)

### Community 4 - "codec_roundtrip.rs"
Cohesion: 0.14
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "App"
Cohesion: 0.10
Nodes (4): App, own_hwnd(), Release, Frame

### Community 8 - "sources.rs"
Cohesion: 0.05
Nodes (7): enumerate(), exe_name(), inspect_window(), monitors(), PW_RENDERFULLCONTENT, SourceInfo, window_cb()

### Community 9 - "Android (`android/`)"
Cohesion: 0.27
Nodes (4): Android (`android/`), GhAsset, Updater, UpdaterTest

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (17): enumerate_mfts(), friendly_name(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample (+9 more)

### Community 11 - "protocol.rs"
Cohesion: 0.07
Nodes (19): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, encode_msg() (+11 more)

### Community 12 - "steps"
Cohesion: 0.08
Nodes (24): environment, if, runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy (+16 more)

### Community 13 - "update.rs"
Cohesion: 0.12
Nodes (21): API, ASSET, check(), cleanup(), client(), download(), GhAsset, GhRelease (+13 more)

### Community 14 - "ViewerModel"
Cohesion: 0.12
Nodes (3): Friend, Status, ViewerModel

### Community 15 - "speedtest.rs"
Cohesion: 0.12
Nodes (12): MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK, STREAMS, TAIL, TARGET (+4 more)

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (58): app_icon(), badge(), button(), button_full(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_button() (+50 more)

### Community 18 - "gpu.rs"
Cohesion: 0.08
Nodes (3): create_device(), Sendable, winrt_device()

### Community 19 - "host/web.rs"
Cohesion: 0.05
Nodes (21): BROWSER_PREDICT, browser_predictions(), browser_predictions_follow_first_v4_srflx(), CONNECT_TIMEOUT, default_v4(), drop_unsafe_candidates(), drop_unsafe_candidates_keeps_lan(), forbidden_dest() (+13 more)

### Community 21 - "with"
Cohesion: 0.06
Nodes (38): runs-on, steps, GIT_CLIFF_TAG_PATTERN, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release (+30 more)

### Community 22 - "LiveScreen"
Cohesion: 0.18
Nodes (10): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+2 more)

### Community 23 - "page_check.rs"
Cohesion: 0.08
Nodes (36): keygen(), main(), MANIFEST_PREFIX, pubkey(), sign(), sign_web(), usage(), check() (+28 more)

### Community 24 - "share_view.rs"
Cohesion: 0.09
Nodes (22): AUTO_QUALITY_COOLDOWN, begin_start(), COPY_FAILED, Picker, picker(), poll_starting(), Preview, source_card() (+14 more)

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (15): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+7 more)

### Community 26 - ".start"
Cohesion: 0.20
Nodes (3): Hosting, Callback, Run

### Community 27 - "share.rs"
Cohesion: 0.08
Nodes (11): cut_feed(), pack_size(), PendingReconf, Reconf, Resources, ShareSession, Sink, SinkTx (+3 more)

### Community 28 - "steps"
Cohesion: 0.08
Nodes (32): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+24 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (assistir e transmitir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "hdr.rs"
Cohesion: 0.05
Nodes (15): compile_shader(), DEFAULT_SDR_WHITE_NITS, detect_hdr(), find_output_for_monitor(), get_monitor_gdi_name(), HdrConverter, HdrInfo, HLSL_SOURCE (+7 more)

### Community 31 - "RFC: sinalização web v3 — ECDH efêmero + híbrido pós-quântico"
Cohesion: 0.06
Nodes (23): HostService, 10. Riscos que continuam, 1. Como é hoje (v2), 2. Quem usa e o que cada um arrisca, 3. Primitivas e bibliotecas, 4.1 Material do host, 4.2 Link, 4.3 Publicação da chave pública (+15 more)

### Community 32 - "pq.js"
Cohesion: 0.07
Nodes (29): RFC-8032, add(), BaseCaseMultiply(), byteSwap(), byteSwap32(), checkOpts(), clone(), _cloneInto() (+21 more)

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.07
Nodes (59): AbortOnDrop, __android_log_write(), audio_loop(), get_host(), get_session(), HostSession, init_logging(), INIT_LOGGING (+51 more)

### Community 35 - "Native"
Cohesion: 0.09
Nodes (5): Amigos e presença, Chat, Regras, Native, Presence

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 38 - "ui/avatar.rs"
Cohesion: 0.09
Nodes (9): inter(), LiveScreenTheme(), Sinal, avatar(), color_from_u32(), draw_avatar_item(), ink_color(), paint_avatar() (+1 more)

### Community 39 - "viewer.rs"
Cohesion: 0.11
Nodes (15): BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64, CLOSE_KICKED_U64, CLOSE_PROTOCOL_U64, close_reason(), CLOSE_STOPPED_U64, DecodeStats (+7 more)

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

### Community 48 - "wgc.rs"
Cohesion: 0.09
Nodes (10): SourceId, Monitor, Window, border_removable(), DEFAULT_PIXEL_FORMAT, handle_frame(), property_present(), SESSION_TYPE (+2 more)

### Community 49 - "Target"
Cohesion: 0.15
Nodes (9): AbortJoinSet<T>, invite_roundtrip_and_proof_binding(), parse_invite(), room_proof(), Target, Friend, Invite, verify_room_proof() (+1 more)

### Community 50 - "e2e_window.rs"
Cohesion: 0.16
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 51 - "encoder.rs"
Cohesion: 0.10
Nodes (14): HostStats, configure(), CpuCopy, create_mft(), Encoder, FAST_REPEATS, Feed, Idle (+6 more)

### Community 52 - "capture/mod.rs"
Cohesion: 0.15
Nodes (4): CountSink, FirstFrameSink, FrameSink, SinkHandle

### Community 53 - "preset.rs"
Cohesion: 0.10
Nodes (30): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), is_reduction(), label(), light_presets_are_reductions_and_fall_back() (+22 more)

### Community 54 - "ui/friends.rs"
Cohesion: 0.07
Nodes (17): ChatRole, Host, Viewer, panel(), Draft, MAX_LIST, Screen, Ended (+9 more)

### Community 56 - "UpdateInfo"
Cohesion: 0.36
Nodes (7): UpdateInfo, Available, Downloading, Failed, Idle, Ready, UpdateState

### Community 57 - "settings.rs"
Cohesion: 0.13
Nodes (28): button(), diagnostics(), experimental(), FORCED, hint(), icon_button(), open_help(), open_logs() (+20 more)

### Community 58 - "src/web.rs"
Cohesion: 0.06
Nodes (40): ROOM_KEY_LEN, AnswerSealer, check_pre(), compute_pre(), constant_time_eq(), cross_language_fixture_test(), derive_ctx(), derive_k_pre() (+32 more)

### Community 59 - "Mode"
Cohesion: 0.27
Nodes (9): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), Mode, CloseMidBody, Status, Throttle, serve() (+1 more)

### Community 61 - "share.rs"
Cohesion: 0.10
Nodes (17): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+9 more)

### Community 62 - "Ink"
Cohesion: 0.13
Nodes (16): AvatarData, AvatarDef, AvatarShape, Circle, Ellipse, Ink, Accent, Back (+8 more)

### Community 63 - "clipboard.rs"
Cohesion: 0.09
Nodes (6): ClipboardGuard, copy_sensitive(), create_message_window(), publish(), set_data(), set_sensitive_clipboard()

### Community 64 - "config.rs"
Cohesion: 0.06
Nodes (30): conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, open_output(), output_loop(), PlaybackHandle (+22 more)

### Community 65 - "HomeScreen"
Cohesion: 0.13
Nodes (22): Avatar(), AvatarPicker(), AddFriendDialog(), App(), Badge(), CallButton(), ChatSheet(), clipboardText() (+14 more)

### Community 66 - "handle_viewer"
Cohesion: 0.14
Nodes (5): admit(), CountGuard, Feeds, handle_viewer(), Shared

### Community 67 - "DxgiCapture"
Cohesion: 0.18
Nodes (5): Navegador (web), bytesToHex(), hexToBytes(), ICE_SERVERS, Room

### Community 68 - "capture.rs"
Cohesion: 0.07
Nodes (33): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), DEFAULT_CHECK, default_render_id() (+25 more)

### Community 71 - "signal3.js"
Cohesion: 0.05
Nodes (47): esbuild, @noble/curves, @noble/post-quantum, wrangler, devDependencies, esbuild@0.28.2, @noble/curves@2.4.0, @noble/post-quantum@0.7.1 (+39 more)

### Community 73 - "Callback"
Cohesion: 0.09
Nodes (47): addOpusStereo(), appendChat(), autoAvatar(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect() (+39 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "src/chat.rs"
Cohesion: 0.12
Nodes (14): CHAT_HISTORY, chat_line_avatar_resolution_and_serde(), ChatLine, ChatLog, clean_chat(), DONATE_AMOUNTS, HOST_SENDER, incoming_donation_rules() (+6 more)

### Community 77 - "ui/mod.rs"
Cohesion: 0.10
Nodes (7): classify(), EndKind, Error, Info, Warn, on_screen(), run()

### Community 78 - "sharing"
Cohesion: 0.20
Nodes (17): ShareStatus, viewers_for(), Upload, Done, Failed, Measuring, Unknown, chat_card() (+9 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 83 - "run"
Cohesion: 0.36
Nodes (5): base_url(), init_crypto(), run(), super::Host, host_ws_url()

### Community 84 - "abytes"
Cohesion: 0.10
Nodes (26): aarray2(), abytes(), aexists(), anumber(), aoutput(), clean(), cleanBytes(), copyBytes2() (+18 more)

### Community 85 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 86 - "watch_view.rs"
Cohesion: 0.07
Nodes (27): find_invite(), find_invite_in_pasted_text(), CallTone, Active, Alert, Neutral, Plain, I_CHAT (+19 more)

### Community 87 - "mulAddUnsafe"
Cohesion: 0.13
Nodes (21): aarray(), abool(), abool2(), afunction(), aobject2(), asafenumber(), bitLen(), cmov() (+13 more)

### Community 88 - "edwards"
Cohesion: 0.14
Nodes (16): aoddModulus(), assertIsSquare(), createCurveFields(), createField(), edwards(), Field(), FpLegendre(), FpSqrt() (+8 more)

### Community 89 - "start"
Cohesion: 0.28
Nodes (9): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+1 more)

### Community 90 - "mulUnsafe"
Cohesion: 0.14
Nodes (19): assertPoint(), buildWnafTable(), fixedWindowCT(), getWindowSize(), getWnafPrecomputes(), hasWindowSize(), mulCT(), mulCTBlinded() (+11 more)

### Community 93 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

### Community 94 - "CallState"
Cohesion: 0.40
Nodes (5): CallState, Active, Alert, HangUp, Neutral

### Community 96 - "montgomery"
Cohesion: 0.19
Nodes (18): aInRange(), bytesToHex(), bytesToNumberBE(), bytesToNumberLE(), cmask(), copyBytes(), createKeygen(), cswap() (+10 more)

### Community 98 - "SocketAddr"
Cohesion: 0.20
Nodes (8): drain_outputs(), host_predictions(), host_predictions_radmin_rule(), maps(), ports(), sa(), Socks, stun_mappings()

### Community 100 - "peer"
Cohesion: 0.11
Nodes (11): rtt_congested(), local_v6(), peer(), WebPeer, SignalOut, Answer, Close, Host (+3 more)

### Community 101 - "files"
Cohesion: 0.12
Nodes (15): files, avatars.json, fonts/inter-400.woff2, fonts/inter-500.woff2, fonts/inter-600.woff2, fonts/LICENSE-Lucide.txt, fonts/OFL-Inter.txt, fonts/OFL-SpaceGrotesk.txt (+7 more)

### Community 103 - "ID3D11Device"
Cohesion: 0.26
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 104 - ".start"
Cohesion: 0.22
Nodes (4): invite_addr(), Overload, format_invite(), new_room_key()

### Community 105 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 106 - "ui/friends.rs"
Cohesion: 0.16
Nodes (12): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+4 more)

### Community 108 - "build.rs"
Cohesion: 0.31
Nodes (7): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 114 - "SourceId"
Cohesion: 0.18
Nodes (12): device_name(), Dib, fit(), monitor_cb(), monitor_info(), primary_monitor(), refresh_rate(), source_size() (+4 more)

### Community 116 - "isPosBig"
Cohesion: 0.29
Nodes (8): abignumber(), asciiToBase16(), hexToBytes(), isPosBig(), numberToBytesBE(), numberToBytesLE(), toBytes(), validateMSMScalars()

### Community 118 - "duration"
Cohesion: 0.12
Nodes (10): I_CIRCLE_CHECK, I_INFO, I_WARN, LIFETIME, MAX_VISIBLE, ToastKind, Info, Success (+2 more)

### Community 119 - "bitReversalInplace"
Cohesion: 0.47
Nodes (6): bitReversalInplace(), checkU32(), getZettas(), isPowerOfTwo(), log2(), reverseBits()

### Community 121 - "WebState"
Cohesion: 0.40
Nodes (5): WebState, Connecting, Off, Online, Retrying

### Community 123 - "ViewerSession"
Cohesion: 0.14
Nodes (7): ViewerMsg, Chat, Hello, RequestKeyframe, SetQuality, SetVisible, ViewerSession

### Community 128 - "WebVerifyStatus"
Cohesion: 0.40
Nodes (5): WebVerifyStatus, Checking, Failed, Idle, Verified

## Knowledge Gaps
- **138 isolated node(s):** `livescreen`, `Accent`, `Dark`, `Light`, `Back` (+133 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1045 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **26 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Preset` connect `ShareSession` to `config.rs`, `rust/src/lib.rs`, `host/mod.rs`, `handle_viewer`, `viewer.rs`, `share.rs`, `sharing`, `Host`, `preset.rs`, `share_view.rs`, `settings.rs`, `ViewerSession`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Accent`, `Dark` to the rest of the system?**
  _138 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1337126600284495 - nodes in this community are weakly interconnected._
- **Why does `Regras` connect `Native` to `DxgiCapture`, `ViewerModel`, `LiveScreen`, `.start`, `share.rs`, `RFC: sinalização web v3 — ECDH efêmero + híbrido pós-quântico`?**
  _High betweenness centrality (0.041) - this node is a cross-community bridge._
- **Should `src/avatar.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07152496626180836 - nodes in this community are weakly interconnected._
- **Why does `App` connect `App` to `config.rs`, `ui/friends.rs`, `ui/mod.rs`, `sharing`, `wgc.rs`, `WatchSession`, `ui/friends.rs`, `duration`, `share_view.rs`, `settings.rs`, `watch_view.rs`, `share.rs`, `share.rs`?**
  _High betweenness centrality (0.039) - this node is a cross-community bridge._