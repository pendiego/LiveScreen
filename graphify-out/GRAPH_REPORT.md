# Graph Report - LiveScreen-publish  (2026-11-02)

## Corpus Check
- 95 files · ~138,591 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 56 file(s) not represented in the graph (top: .xml 25, .woff2 10, .ttf 7)

## Summary
- 2917 nodes · 5978 edges · 138 communities (107 shown, 31 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 172 edges (avg confidence: 0.78)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c9363947`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- src/avatar.rs
- WebKeys
- host/mod.rs
- codec_roundtrip.rs
- convert.rs
- HdrConverter
- sources.rs
- Avatar.kt
- mf.rs
- protocol.rs
- steps
- update.rs
- detect_hdr
- speedtest.rs
- theme.rs
- Ui
- gpu.rs
- host/web.rs
- Host
- with
- HWND
- page_check.rs
- Picker
- ShareSession
- start_encoder
- share.rs
- steps
- LiveScreen
- hdr.rs
- RFC: sinalização web v3 — ECDH efêmero + híbrido pós-quântico
- pq.js
- test-changelog.sh
- frame_from_body
- write_msg
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
- encoder.rs
- capture/mod.rs
- preset.rs
- ui/friends.rs
- WatchSession
- Converter
- App
- src/web.rs
- super
- inspect_window
- share.rs
- Mic
- clipboard.rs
- playback.rs
- linear_to_srgb
- HostService.kt
- DxgiCapture
- capture.rs
- config.rs
- compile_shader
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
- Display
- Theme.kt
- AudioState
- CallState
- capture_loop
- montgomery
- Dib
- SocketAddr
- SignalOut
- files
- VideoDecoder
- ID3D11Device
- OutputResult
- .ui
- ui/friends.rs
- Release
- build.rs
- EncodeParams
- run
- enumerate_mfts
- SignalIn
- AudioEncoder
- HMONITOR
- isPosBig
- Tab
- duration
- bitReversalInplace
- AudioPlayer
- WebState
- Screen
- ViewerMsg
- controls
- AvatarTest.kt
- Ink
- WebVerifyStatus
- AudioHandle
- variant_with
- SinalSnack
- Mmcss
- Stroke
- FieldTone
- json
- Target

## God Nodes (most connected - your core abstractions)
1. `App` - 75 edges
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

## Communities (138 total, 31 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.14
Nodes (19): as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size(), Gpu (+11 more)

### Community 1 - "src/avatar.rs"
Cohesion: 0.10
Nodes (25): ALL, all(), auto(), Avatar, BACK, COELHO_SHAPES, COUNT, DARK (+17 more)

### Community 2 - "WebKeys"
Cohesion: 0.07
Nodes (5): HostRequest, HostStatus, HostViewer, parseHostStatus(), strOrNull()

### Community 3 - "host/mod.rs"
Cohesion: 0.07
Nodes (22): Abr, ABR_FLOOR, admit(), answer_presence(), CountGuard, decide_target_fps(), Feeds, handle_viewer() (+14 more)

### Community 4 - "codec_roundtrip.rs"
Cohesion: 0.14
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 6 - "HdrConverter"
Cohesion: 0.13
Nodes (3): App, connect(), sync_visible()

### Community 9 - "Avatar.kt"
Cohesion: 0.07
Nodes (4): Avatar(), AvatarData, AvatarDef, AvatarPicker()

### Community 11 - "protocol.rs"
Cohesion: 0.05
Nodes (27): ALPN, budget_released_on_drop(), Budgeted, BudgetExceeded, check_len(), CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED (+19 more)

### Community 12 - "steps"
Cohesion: 0.08
Nodes (24): environment, if, runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy (+16 more)

### Community 13 - "update.rs"
Cohesion: 0.16
Nodes (13): API, ASSET, cleanup(), GhAsset, GhRelease, install(), install_at(), install_rolls_back_when_launch_fails() (+5 more)

### Community 14 - "detect_hdr"
Cohesion: 0.11
Nodes (5): ChatItem, Friend, parseChatItems(), Status, ViewerModel

### Community 15 - "speedtest.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.06
Nodes (34): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), call_title_with_avatar(), card() (+26 more)

### Community 17 - "Ui"
Cohesion: 0.12
Nodes (27): button(), button_full(), call_button(), CallTone, Active, Alert, Neutral, Plain (+19 more)

### Community 18 - "gpu.rs"
Cohesion: 0.09
Nodes (3): create_device(), Sendable, winrt_device()

### Community 19 - "host/web.rs"
Cohesion: 0.05
Nodes (21): BROWSER_PREDICT, browser_predictions(), browser_predictions_follow_first_v4_srflx(), CONNECT_TIMEOUT, default_v4(), drop_unsafe_candidates(), drop_unsafe_candidates_keeps_lan(), forbidden_dest() (+13 more)

### Community 20 - "Host"
Cohesion: 0.08
Nodes (5): Host, HostConfig, invite_addr(), Overload, ViewerInfo

### Community 21 - "with"
Cohesion: 0.06
Nodes (38): runs-on, steps, GIT_CLIFF_TAG_PATTERN, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release (+30 more)

### Community 22 - "HWND"
Cohesion: 0.18
Nodes (10): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+2 more)

### Community 23 - "page_check.rs"
Cohesion: 0.08
Nodes (36): keygen(), main(), MANIFEST_PREFIX, pubkey(), sign(), sign_web(), usage(), check() (+28 more)

### Community 24 - "Picker"
Cohesion: 0.16
Nodes (9): SourceInfo, begin_start(), Picker, poll_starting(), Preview, Starting, Thumb, to_image() (+1 more)

### Community 25 - "ShareSession"
Cohesion: 0.11
Nodes (15): preset_id(), Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+7 more)

### Community 26 - "start_encoder"
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
Nodes (18): compile_shader(), DEFAULT_SDR_WHITE_NITS, detect_hdr(), find_output_for_monitor(), get_monitor_gdi_name(), HdrConverter, HdrInfo, HLSL_SOURCE (+10 more)

### Community 31 - "RFC: sinalização web v3 — ECDH efêmero + híbrido pós-quântico"
Cohesion: 0.09
Nodes (21): 10. Riscos que continuam, 1. Como é hoje (v2), 3. Primitivas e bibliotecas, 4.1 Material do host, 4.2 Link, 4.3 Publicação da chave pública, 4.4 Offer (espectador → host), 4.5 Answer (host → espectador) (+13 more)

### Community 32 - "pq.js"
Cohesion: 0.07
Nodes (29): RFC-8032, add(), BaseCaseMultiply(), byteSwap(), byteSwap32(), checkOpts(), clone(), _cloneInto() (+21 more)

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "frame_from_body"
Cohesion: 0.07
Nodes (59): AbortOnDrop, __android_log_write(), audio_loop(), get_host(), get_session(), HostSession, init_logging(), INIT_LOGGING (+51 more)

### Community 35 - "write_msg"
Cohesion: 0.09
Nodes (5): Amigos e presença, Chat, Regras, Native, Presence

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 37 - "Updater.kt"
Cohesion: 0.08
Nodes (11): Android (`android/`), GhAsset, UpdateInfo, Updater, Available, Downloading, Failed, Idle (+3 more)

### Community 38 - "ui/avatar.rs"
Cohesion: 0.14
Nodes (6): avatar(), color_from_u32(), draw_avatar_item(), ink_color(), paint_avatar(), picker()

### Community 39 - "viewer.rs"
Cohesion: 0.09
Nodes (17): BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64, CLOSE_KICKED_U64, CLOSE_PROTOCOL_U64, close_reason(), CLOSE_STOPPED_U64, DecodeStats (+9 more)

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.09
Nodes (5): NEXT_SURFACES_ID, Producer, SLOTS, Surfaces, Target

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "wgc.rs"
Cohesion: 0.11
Nodes (7): border_removable(), DEFAULT_PIXEL_FORMAT, handle_frame(), property_present(), SESSION_TYPE, start(), WgcCapture

### Community 49 - "Target"
Cohesion: 0.13
Nodes (13): AbortJoinSet<T>, find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), Target (+5 more)

### Community 50 - "e2e_window.rs"
Cohesion: 0.16
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 51 - "encoder.rs"
Cohesion: 0.10
Nodes (14): EncodedFrame, configure(), CpuCopy, create_mft(), Encoder, FAST_REPEATS, Feed, Idle (+6 more)

### Community 52 - "capture/mod.rs"
Cohesion: 0.15
Nodes (5): CountSink, FirstFrameSink, FrameSink, selftest(), SinkHandle

### Community 53 - "preset.rs"
Cohesion: 0.11
Nodes (27): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not(), lighter_than_stream() (+19 more)

### Community 54 - "ui/friends.rs"
Cohesion: 0.06
Nodes (23): ChatRole, Host, Viewer, panel(), Draft, MAX_LIST, AUTO_QUALITY_COOLDOWN, COPY_FAILED (+15 more)

### Community 57 - "App"
Cohesion: 0.22
Nodes (20): audio(), button(), device_combo(), Devices, diagnostics(), experimental(), FORCED, hint() (+12 more)

### Community 58 - "src/web.rs"
Cohesion: 0.06
Nodes (40): ROOM_KEY_LEN, AnswerSealer, check_pre(), compute_pre(), constant_time_eq(), cross_language_fixture_test(), derive_ctx(), derive_k_pre() (+32 more)

### Community 59 - "super"
Cohesion: 0.15
Nodes (6): avatar_defaults_to_zero_and_parses(), Experimental, file_overrides_win(), old_ui_json_keeps_new_defaults(), partial_json_keeps_defaults(), Prefs

### Community 61 - "share.rs"
Cohesion: 0.11
Nodes (17): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+9 more)

### Community 62 - "Mic"
Cohesion: 0.26
Nodes (13): AvatarShape, Circle, Ellipse, Ink, Accent, Back, Dark, Light (+5 more)

### Community 63 - "clipboard.rs"
Cohesion: 0.09
Nodes (6): ClipboardGuard, copy_sensitive(), create_message_window(), publish(), set_data(), set_sensitive_clipboard()

### Community 64 - "playback.rs"
Cohesion: 0.12
Nodes (13): Device, conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, open_output(), output_loop() (+5 more)

### Community 65 - "linear_to_srgb"
Cohesion: 0.20
Nodes (16): AddFriendDialog(), App(), Badge(), CallButton(), ChatSheet(), clipboardText(), FriendsSection(), HomeScreen() (+8 more)

### Community 67 - "DxgiCapture"
Cohesion: 0.18
Nodes (5): Navegador (web), bytesToHex(), hexToBytes(), ICE_SERVERS, Room

### Community 68 - "capture.rs"
Cohesion: 0.09
Nodes (21): AudioTarget, Process, System, DEFAULT_CHECK, default_render_id(), follow_default_render(), FRAME_BYTES, FRAME_FLOATS (+13 more)

### Community 69 - "config.rs"
Cohesion: 0.23
Nodes (13): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+5 more)

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
Cohesion: 0.10
Nodes (16): CHAT_HISTORY, chat_line_avatar_resolution_and_serde(), ChatLimiter, ChatLine, ChatLog, clean_chat(), DONATE_AMOUNTS, HOST_SENDER (+8 more)

### Community 77 - "ui/mod.rs"
Cohesion: 0.09
Nodes (7): classify(), EndKind, Error, Info, Warn, on_screen(), run()

### Community 78 - "sharing"
Cohesion: 0.20
Nodes (17): ShareStatus, Upload, Done, Failed, Measuring, Unknown, chat_card(), confirm_modal() (+9 more)

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
Cohesion: 0.10
Nodes (12): I_CHAT, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY (+4 more)

### Community 87 - "mulAddUnsafe"
Cohesion: 0.13
Nodes (21): aarray(), abool(), abool2(), afunction(), aobject2(), asafenumber(), bitLen(), cmov() (+13 more)

### Community 88 - "edwards"
Cohesion: 0.14
Nodes (16): aoddModulus(), assertIsSquare(), createCurveFields(), createField(), edwards(), Field(), FpLegendre(), FpSqrt() (+8 more)

### Community 89 - "start"
Cohesion: 0.30
Nodes (8): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), start()

### Community 90 - "mulUnsafe"
Cohesion: 0.14
Nodes (19): assertPoint(), buildWnafTable(), fixedWindowCT(), getWindowSize(), getWnafPrecomputes(), hasWindowSize(), mulCT(), mulCTBlinded() (+11 more)

### Community 91 - "Display"
Cohesion: 0.18
Nodes (3): Display, GpuFrame, Imported

### Community 92 - "Theme.kt"
Cohesion: 0.18
Nodes (3): inter(), LiveScreenTheme(), Sinal

### Community 93 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

### Community 94 - "CallState"
Cohesion: 0.40
Nodes (5): CallState, Active, Alert, HangUp, Neutral

### Community 95 - "capture_loop"
Cohesion: 0.33
Nodes (9): AudioRequest, capture_loop(), Mic, Default, mic_error_text(), open_mic(), Opened, run() (+1 more)

### Community 96 - "montgomery"
Cohesion: 0.19
Nodes (18): aInRange(), bytesToHex(), bytesToNumberBE(), bytesToNumberLE(), cmask(), copyBytes(), createKeygen(), cswap() (+10 more)

### Community 97 - "Dib"
Cohesion: 0.29
Nodes (5): Dib, fit(), thumbnail(), thumbnail_monitor(), thumbnail_window()

### Community 98 - "SocketAddr"
Cohesion: 0.20
Nodes (8): drain_outputs(), host_predictions(), host_predictions_radmin_rule(), maps(), ports(), sa(), Socks, stun_mappings()

### Community 100 - "SignalOut"
Cohesion: 0.11
Nodes (11): rtt_congested(), local_v6(), peer(), WebPeer, SignalOut, Answer, Close, Host (+3 more)

### Community 101 - "files"
Cohesion: 0.12
Nodes (15): files, avatars.json, fonts/inter-400.woff2, fonts/inter-500.woff2, fonts/inter-600.woff2, fonts/LICENSE-Lucide.txt, fonts/OFL-Inter.txt, fonts/OFL-SpaceGrotesk.txt (+7 more)

### Community 103 - "ID3D11Device"
Cohesion: 0.27
Nodes (4): Composer, Grab, GrabSink, texture()

### Community 104 - "OutputResult"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 106 - "ui/friends.rs"
Cohesion: 0.17
Nodes (12): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+4 more)

### Community 107 - "Release"
Cohesion: 0.24
Nodes (9): check(), client(), download(), pick(), Release, signature_binds_version_and_hash(), signature_ok(), signed_message() (+1 more)

### Community 108 - "build.rs"
Cohesion: 0.31
Nodes (7): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES

### Community 109 - "EncodeParams"
Cohesion: 0.39
Nodes (5): EncodeParams, is_reduction(), viewers_for(), Choices, pixel_rate()

### Community 110 - "run"
Cohesion: 0.31
Nodes (5): create_duplication(), dxgi_draws_cursor(), DxgiCapture, run(), start()

### Community 111 - "enumerate_mfts"
Cohesion: 0.22
Nodes (4): enumerate_mfts(), friendly_name(), set_codec_value(), subtype()

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 114 - "HMONITOR"
Cohesion: 0.22
Nodes (12): device_name(), enumerate(), exe_name(), inspect_window(), monitor_cb(), monitor_info(), monitors(), primary_monitor() (+4 more)

### Community 116 - "isPosBig"
Cohesion: 0.29
Nodes (8): abignumber(), asciiToBase16(), hexToBytes(), isPosBig(), numberToBytesBE(), numberToBytesLE(), toBytes(), validateMSMScalars()

### Community 117 - "Tab"
Cohesion: 0.25
Nodes (8): Tab, Audio, Diagnostics, Experimental, Share, Updates, Video, TABS

### Community 118 - "duration"
Cohesion: 0.12
Nodes (11): spawn_share_job(), I_CIRCLE_CHECK, I_INFO, I_WARN, LIFETIME, MAX_VISIBLE, ToastKind, Info (+3 more)

### Community 119 - "bitReversalInplace"
Cohesion: 0.47
Nodes (6): bitReversalInplace(), checkU32(), getZettas(), isPowerOfTwo(), log2(), reverseBits()

### Community 121 - "WebState"
Cohesion: 0.40
Nodes (5): WebState, Connecting, Off, Online, Retrying

### Community 122 - "Screen"
Cohesion: 0.29
Nodes (7): Screen, Ended, Home, SharePicker, Sharing, Watching, WatchInput

### Community 123 - "ViewerMsg"
Cohesion: 0.33
Nodes (6): ViewerMsg, Chat, Hello, RequestKeyframe, SetQuality, SetVisible

### Community 124 - "controls"
Cohesion: 0.33
Nodes (6): ControlAction, Leave, None, ToggleFullscreen, controls(), watching()

### Community 126 - "Ink"
Cohesion: 0.33
Nodes (5): Ink, Accent, Back, Dark, Light

### Community 128 - "WebVerifyStatus"
Cohesion: 0.40
Nodes (5): WebVerifyStatus, Checking, Failed, Idle, Verified

### Community 130 - "variant_with"
Cohesion: 0.47
Nodes (3): variant_bool(), variant_u32(), variant_with()

### Community 131 - "SinalSnack"
Cohesion: 0.40
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 135 - "FieldTone"
Cohesion: 0.50
Nodes (4): FieldTone, Error, Normal, Warn

### Community 136 - "json"
Cohesion: 0.50
Nodes (3): json(), num(), test_json_matches_web_public_file()

### Community 137 - "Target"
Cohesion: 0.67
Nodes (3): Target, Nv12, Rgb

## Knowledge Gaps
- **139 isolated node(s):** `livescreen`, `Accent`, `Dark`, `Light`, `Back` (+134 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1047 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **31 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Preset` connect `ShareSession` to `frame_from_body`, `host/mod.rs`, `viewer.rs`, `share.rs`, `super`, `EncodeParams`, `sharing`, `Host`, `preset.rs`, `ui/friends.rs`, `Picker`, `App`, `ViewerMsg`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Accent`, `Dark` to the rest of the system?**
  _139 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13513513513513514 - nodes in this community are weakly interconnected._
- **Why does `Regras` connect `write_msg` to `HostService.kt`, `DxgiCapture`, `detect_hdr`, `HWND`, `start_encoder`, `share.rs`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **Should `src/avatar.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09852216748768473 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `detect_hdr` to `linear_to_srgb`, `WebKeys`, `write_msg`, `Updater.kt`, `VideoDecoder`, `MainActivity.kt`, `AudioPlayer`?**
  _High betweenness centrality (0.036) - this node is a cross-community bridge._