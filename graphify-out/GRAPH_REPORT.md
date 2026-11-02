# Graph Report - LiveScreen-publish  (2026-11-02)

## Corpus Check
- 95 files · ~137,253 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 56 file(s) not represented in the graph (top: .xml 25, .woff2 10, .ttf 7)

## Summary
- 2904 nodes · 5940 edges · 123 communities (94 shown, 29 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 169 edges (avg confidence: 0.78)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `4c29e345`
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
- capture.rs
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
- Avatar.kt
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
- Updater.kt
- ui/avatar.rs
- viewer.rs
- main.rs
- present.rs
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- wgc.rs
- Target
- e2e_window.rs
- Converter
- capture/mod.rs
- preset.rs
- ui/friends.rs
- WebKeys
- HostService.kt
- settings.rs
- src/web.rs
- Resources
- Callback
- share.rs
- Ink
- clipboard.rs
- super
- HomeScreen
- handle_viewer
- DxgiCapture
- Display
- src/chat.rs
- ScreenEncoder
- signal3.js
- MainActivity.kt
- Callback
- HostMsg
- ChatLine
- Producer
- sharing
- pages.yml
- run
- abytes
- BtnKind
- watch_view.rs
- mulAddUnsafe
- edwards
- detect_hdr
- mulUnsafe
- sanitize
- config.rs
- AudioState
- CallState
- FieldTone
- montgomery
- .resolve
- SocketAddr
- SignalOut
- files
- VideoDecoder
- ID3D11Device
- Dib
- HMONITOR
- ui/friends.rs
- EndpointId
- HdrConverter
- SinalSnack
- run
- Target
- SignalIn
- AudioEncoder.kt
- HWND
- isPosBig
- bitReversalInplace
- AudioPlayer
- WebState
- Mmcss
- Ui

## God Nodes (most connected - your core abstractions)
1. `App` - 74 edges
2. `Preset` - 53 edges
3. `ViewerModel` - 46 edges
4. `Native` - 43 edges
5. `Converter` - 33 edges
6. `Codec` - 31 edges
7. `Shared` - 30 edges
8. `SourceId` - 29 edges
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
- `2. Quem usa e o que cada um arrisca` --references--> `HostService`  [INFERRED]
  docs/rfc-sinalizacao-web-v3.md → android/app/src/main/java/io/github/pendiego/livescreen/HostService.kt

## Import Cycles
- None detected.

## Communities (123 total, 29 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.13
Nodes (20): DecodeStats, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size() (+12 more)

### Community 1 - "src/avatar.rs"
Cohesion: 0.07
Nodes (33): ALL, all(), auto(), Avatar, BACK, COELHO_SHAPES, COUNT, DARK (+25 more)

### Community 2 - "Hosting.kt"
Cohesion: 0.07
Nodes (7): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull(), ChatItem, parseChatItems()

### Community 3 - "host/mod.rs"
Cohesion: 0.08
Nodes (16): Abr, ABR_FLOOR, CountGuard, decide_target_fps(), Feeds, HANDSHAKE_TIMEOUT, MAX_HANDSHAKES, retire() (+8 more)

### Community 4 - "codec_roundtrip.rs"
Cohesion: 0.06
Nodes (19): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+11 more)

### Community 6 - "App"
Cohesion: 0.06
Nodes (6): App, classify(), on_screen(), own_hwnd(), run(), Frame

### Community 9 - "capture.rs"
Cohesion: 0.05
Nodes (40): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), FRAME_BYTES, FRAME_FLOATS (+32 more)

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (17): enumerate_mfts(), friendly_name(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample (+9 more)

### Community 11 - "protocol.rs"
Cohesion: 0.06
Nodes (24): ALPN, budget_released_on_drop(), Budgeted, BudgetExceeded, check_len(), CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED (+16 more)

### Community 12 - "steps"
Cohesion: 0.08
Nodes (24): environment, if, runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy (+16 more)

### Community 13 - "update.rs"
Cohesion: 0.09
Nodes (29): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+21 more)

### Community 14 - "ViewerModel"
Cohesion: 0.13
Nodes (4): Friend, Presence, Status, ViewerModel

### Community 15 - "speedtest.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (43): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), call_title_with_avatar(), card() (+35 more)

### Community 17 - "Screen"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 18 - "gpu.rs"
Cohesion: 0.08
Nodes (3): create_device(), Sendable, winrt_device()

### Community 19 - "host/web.rs"
Cohesion: 0.04
Nodes (26): rtt_congested(), BROWSER_PREDICT, browser_predictions(), browser_predictions_follow_first_v4_srflx(), CONNECT_TIMEOUT, default_v4(), drain_outputs(), drop_unsafe_candidates() (+18 more)

### Community 20 - "Host"
Cohesion: 0.09
Nodes (4): Host, invite_addr(), Overload, new_room_key()

### Community 21 - "with"
Cohesion: 0.06
Nodes (38): runs-on, steps, GIT_CLIFF_TAG_PATTERN, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release (+30 more)

### Community 23 - "page_check.rs"
Cohesion: 0.08
Nodes (36): keygen(), main(), MANIFEST_PREFIX, pubkey(), sign(), sign_web(), usage(), check() (+28 more)

### Community 24 - "share_view.rs"
Cohesion: 0.12
Nodes (16): SourceId, Monitor, Window, SourceInfo, AUTO_QUALITY_COOLDOWN, begin_start(), COPY_FAILED, Picker (+8 more)

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (15): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+7 more)

### Community 26 - ".start"
Cohesion: 0.21
Nodes (3): Hosting, Callback, Run

### Community 27 - "share.rs"
Cohesion: 0.11
Nodes (10): is_reduction(), cut_feed(), pack_size(), PendingReconf, Resources, ShareSession, Sink, SinkTx (+2 more)

### Community 28 - "steps"
Cohesion: 0.08
Nodes (32): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+24 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (assistir e transmitir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "hdr.rs"
Cohesion: 0.06
Nodes (7): compile_shader(), DEFAULT_SDR_WHITE_NITS, HLSL_SOURCE, linear_to_srgb(), linear_to_srgb_properties(), sc_rgb_to_srgb(), sc_rgb_to_srgb_keeps_sdr_and_clips_highlights()

### Community 31 - "RFC: sinalização web v3 — ECDH efêmero + híbrido pós-quântico"
Cohesion: 0.09
Nodes (21): 10. Riscos que continuam, 1. Como é hoje (v2), 3. Primitivas e bibliotecas, 4.1 Material do host, 4.2 Link, 4.3 Publicação da chave pública, 4.4 Offer (espectador → host), 4.5 Answer (host → espectador) (+13 more)

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
Cohesion: 0.08
Nodes (4): Amigos e presença, Chat, Regras, Native

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 37 - "Updater.kt"
Cohesion: 0.09
Nodes (11): Android (`android/`), GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle (+3 more)

### Community 38 - "ui/avatar.rs"
Cohesion: 0.14
Nodes (6): avatar(), color_from_u32(), draw_avatar_item(), ink_color(), paint_avatar(), picker()

### Community 39 - "viewer.rs"
Cohesion: 0.08
Nodes (21): ViewerMsg, Chat, Hello, RequestKeyframe, SetQuality, SetVisible, BUDGET_MSG, CLOSE_DENIED_U64 (+13 more)

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.11
Nodes (3): NEXT_SURFACES_ID, SLOTS, Target

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "wgc.rs"
Cohesion: 0.10
Nodes (7): border_removable(), DEFAULT_PIXEL_FORMAT, handle_frame(), property_present(), SESSION_TYPE, start(), WgcCapture

### Community 49 - "Target"
Cohesion: 0.13
Nodes (14): AbortJoinSet<T>, find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), parse_invite(), room_proof(), Target (+6 more)

### Community 50 - "e2e_window.rs"
Cohesion: 0.16
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 52 - "capture/mod.rs"
Cohesion: 0.10
Nodes (13): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, CountSink, FirstFrameSink (+5 more)

### Community 53 - "preset.rs"
Cohesion: 0.11
Nodes (28): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not() (+20 more)

### Community 54 - "ui/friends.rs"
Cohesion: 0.12
Nodes (6): Draft, MAX_LIST, I_ADD_FRIEND, I_COPY, I_PEOPLE, I_TRASH

### Community 55 - "WebKeys"
Cohesion: 0.11
Nodes (18): AnswerSealer, cross_language_fixture_test(), derive_ctx(), is_all_zero(), pad_plaintext(), padding_makes_sealed_length_equal_for_different_name_lengths(), pre_failure_returns_before_decapsulation(), regenerate_cross_language_fixture() (+10 more)

### Community 57 - "settings.rs"
Cohesion: 0.13
Nodes (28): button(), diagnostics(), experimental(), FORCED, hint(), icon_button(), open_help(), open_logs() (+20 more)

### Community 58 - "src/web.rs"
Cohesion: 0.07
Nodes (22): ROOM_KEY_LEN, check_pre(), compute_pre(), constant_time_eq(), derive_k_pre(), HOST_PK_LEN, lenient_u8(), MIN_SEALED_LEN (+14 more)

### Community 59 - "Resources"
Cohesion: 0.10
Nodes (15): HostStats, configure(), CpuCopy, create_mft(), Encoder, EncoderHandle, FAST_REPEATS, Feed (+7 more)

### Community 61 - "share.rs"
Cohesion: 0.07
Nodes (20): Codec, Av1, H264, Hevc, Overrides, pick_encoder(), Reconf, start_encoder() (+12 more)

### Community 62 - "Ink"
Cohesion: 0.19
Nodes (15): AvatarData, AvatarDef, AvatarShape, Circle, Ellipse, Ink, Accent, Back (+7 more)

### Community 63 - "clipboard.rs"
Cohesion: 0.09
Nodes (6): ClipboardGuard, copy_sensitive(), create_message_window(), publish(), set_data(), set_sensitive_clipboard()

### Community 64 - "super"
Cohesion: 0.14
Nodes (6): avatar_defaults_to_zero_and_parses(), Experimental, file_overrides_win(), old_ui_json_keeps_new_defaults(), partial_json_keeps_defaults(), Prefs

### Community 65 - "HomeScreen"
Cohesion: 0.20
Nodes (16): AddFriendDialog(), App(), Badge(), CallButton(), ChatSheet(), clipboardText(), FriendsSection(), HomeScreen() (+8 more)

### Community 66 - "handle_viewer"
Cohesion: 0.16
Nodes (9): admit(), answer_presence(), handle_viewer(), Peer, PeerConn, Iroh, Web, encode_msg() (+1 more)

### Community 67 - "DxgiCapture"
Cohesion: 0.09
Nodes (15): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Navegador (web) (+7 more)

### Community 68 - "Display"
Cohesion: 0.18
Nodes (3): Display, GpuFrame, Imported

### Community 69 - "src/chat.rs"
Cohesion: 0.18
Nodes (8): CHAT_HISTORY, ChatLimiter, DONATE_AMOUNTS, HOST_SENDER, limiter_burst_and_refill(), MAX_CHAT_CHARS, MAX_DONATE, MAX_NAME_CHARS

### Community 71 - "signal3.js"
Cohesion: 0.05
Nodes (47): esbuild, @noble/curves, @noble/post-quantum, wrangler, devDependencies, esbuild@0.28.2, @noble/curves@2.4.0, @noble/post-quantum@0.7.1 (+39 more)

### Community 72 - "MainActivity.kt"
Cohesion: 0.09
Nodes (4): MainActivity, inter(), LiveScreenTheme(), Sinal

### Community 73 - "Callback"
Cohesion: 0.09
Nodes (47): addOpusStereo(), appendChat(), autoAvatar(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect() (+39 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "ChatLine"
Cohesion: 0.19
Nodes (7): ChatLine, ChatLog, ChatRole, Host, Viewer, log_caps_and_since(), panel()

### Community 78 - "sharing"
Cohesion: 0.20
Nodes (17): ShareStatus, viewers_for(), Upload, Done, Failed, Measuring, Unknown, chat_card() (+9 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 83 - "run"
Cohesion: 0.27
Nodes (5): base_url(), init_crypto(), run(), super::Host, host_ws_url()

### Community 84 - "abytes"
Cohesion: 0.10
Nodes (26): aarray2(), abytes(), aexists(), anumber(), aoutput(), clean(), cleanBytes(), copyBytes2() (+18 more)

### Community 85 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 86 - "watch_view.rs"
Cohesion: 0.08
Nodes (18): I_CHAT, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY (+10 more)

### Community 87 - "mulAddUnsafe"
Cohesion: 0.13
Nodes (21): aarray(), abool(), abool2(), afunction(), aobject2(), asafenumber(), bitLen(), cmov() (+13 more)

### Community 88 - "edwards"
Cohesion: 0.14
Nodes (16): aoddModulus(), assertIsSquare(), createCurveFields(), createField(), edwards(), Field(), FpLegendre(), FpSqrt() (+8 more)

### Community 89 - "detect_hdr"
Cohesion: 0.24
Nodes (7): detect_hdr(), find_output_for_monitor(), get_monitor_gdi_name(), HdrInfo, hmonitor_for_source(), is_output_hdr(), query_sdr_white_level()

### Community 90 - "mulUnsafe"
Cohesion: 0.14
Nodes (19): assertPoint(), buildWnafTable(), fixedWindowCT(), getWindowSize(), getWnafPrecomputes(), hasWindowSize(), mulCT(), mulCTBlinded() (+11 more)

### Community 91 - "sanitize"
Cohesion: 0.36
Nodes (5): chat_line_avatar_resolution_and_serde(), clean_chat(), incoming_donation_rules(), labels_never_impersonate_host(), sanitize()

### Community 92 - "config.rs"
Cohesion: 0.24
Nodes (12): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), overrides() (+4 more)

### Community 93 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

### Community 94 - "CallState"
Cohesion: 0.40
Nodes (5): CallState, Active, Alert, HangUp, Neutral

### Community 95 - "FieldTone"
Cohesion: 0.50
Nodes (4): FieldTone, Error, Normal, Warn

### Community 96 - "montgomery"
Cohesion: 0.19
Nodes (18): aInRange(), bytesToHex(), bytesToNumberBE(), bytesToNumberLE(), cmask(), copyBytes(), createKeygen(), cswap() (+10 more)

### Community 98 - "SocketAddr"
Cohesion: 0.33
Nodes (7): host_predictions(), host_predictions_radmin_rule(), maps(), ports(), sa(), Socks, stun_mappings()

### Community 100 - "SignalOut"
Cohesion: 0.29
Nodes (7): SignalOut, Answer, Close, Host, Ping, Pk, Reject

### Community 101 - "files"
Cohesion: 0.12
Nodes (15): files, avatars.json, fonts/inter-400.woff2, fonts/inter-500.woff2, fonts/inter-600.woff2, fonts/LICENSE-Lucide.txt, fonts/OFL-Inter.txt, fonts/OFL-SpaceGrotesk.txt (+7 more)

### Community 103 - "ID3D11Device"
Cohesion: 0.24
Nodes (7): Composer, create_duplication(), Grab, GrabSink, run(), start(), texture()

### Community 105 - "HMONITOR"
Cohesion: 0.35
Nodes (7): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate()

### Community 106 - "ui/friends.rs"
Cohesion: 0.08
Nodes (22): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+14 more)

### Community 107 - "EndpointId"
Cohesion: 0.20
Nodes (5): HostConfig, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), ViewerInfo

### Community 109 - "SinalSnack"
Cohesion: 0.40
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 111 - "Target"
Cohesion: 0.67
Nodes (3): Target, Nv12, Rgb

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 114 - "HWND"
Cohesion: 0.26
Nodes (9): exe_name(), fit(), inspect_window(), source_size(), thumbnail(), thumbnail_monitor(), thumbnail_window(), window_cb() (+1 more)

### Community 116 - "isPosBig"
Cohesion: 0.29
Nodes (8): abignumber(), asciiToBase16(), hexToBytes(), isPosBig(), numberToBytesBE(), numberToBytesLE(), toBytes(), validateMSMScalars()

### Community 119 - "bitReversalInplace"
Cohesion: 0.47
Nodes (6): bitReversalInplace(), checkU32(), getZettas(), isPowerOfTwo(), log2(), reverseBits()

### Community 121 - "WebState"
Cohesion: 0.40
Nodes (5): WebState, Connecting, Off, Online, Retrying

### Community 124 - "Ui"
Cohesion: 0.10
Nodes (29): button(), button_full(), call_button(), CallTone, Active, Alert, Neutral, Plain (+21 more)

## Knowledge Gaps
- **139 isolated node(s):** `livescreen`, `Accent`, `Dark`, `Light`, `Back` (+134 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1043 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **29 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Preset` connect `ShareSession` to `super`, `rust/src/lib.rs`, `host/mod.rs`, `handle_viewer`, `viewer.rs`, `sharing`, `Host`, `preset.rs`, `share_view.rs`, `settings.rs`, `share.rs`, `share.rs`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Accent`, `Dark` to the rest of the system?**
  _139 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1282051282051282 - nodes in this community are weakly interconnected._
- **Why does `Regras` connect `Native` to `HostService.kt`, `share.rs`, `DxgiCapture`?**
  _High betweenness centrality (0.047) - this node is a cross-community bridge._
- **Should `src/avatar.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07152496626180836 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `ViewerModel` to `HomeScreen`, `Hosting.kt`, `Native`, `Updater.kt`, `VideoDecoder`, `MainActivity.kt`, `AudioPlayer`?**
  _High betweenness centrality (0.035) - this node is a cross-community bridge._