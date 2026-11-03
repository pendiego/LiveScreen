# Graph Report - LiveScreen-publish  (2026-11-02)

## Corpus Check
- 74 files · ~118,120 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 57 file(s) not represented in the graph (top: .so 14, .woff2 10, .bin 7)

## Summary
- 2318 nodes · 4627 edges · 115 communities (97 shown, 18 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 123 edges (avg confidence: 0.73)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f2174e25`
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
- Converter
- with
- settings.rs
- page_check.rs
- Picker
- ShareSession
- watch_view.rs
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
- Stroke
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
- config.rs
- App
- src/web.rs
- super
- ShareSession
- share.rs
- ViewerSession
- clipboard.rs
- playback.rs
- seal_offer_for_test
- handle_viewer
- DxgiCapture
- capture.rs
- AnswerSealer
- Tab
- signal3.js
- compute_pre
- Callback
- HostMsg
- src/chat.rs
- controls
- ui/mod.rs
- sharing
- pages.yml
- ViewerMsg
- WatchStatus
- run
- abytes
- CallTone
- mulAddUnsafe
- edwards
- start
- mulUnsafe
- Display
- AudioState
- montgomery
- SocketAddr
- SignalOut
- files
- ID3D11Device
- OutputResult
- .ui
- ui/friends.rs
- run
- enumerate_mfts
- SignalIn
- HMONITOR
- isPosBig
- duration
- bitReversalInplace
- WebState
- Codec
- controls
- Ink
- WebVerifyStatus
- variant_with

## God Nodes (most connected - your core abstractions)
1. `App` - 75 edges
2. `Preset` - 50 edges
3. `Converter` - 33 edges
4. `Codec` - 31 edges
5. `Shared` - 30 edges
6. `SourceId` - 29 edges
7. `ShareSession` - 26 edges
8. `EncoderKind` - 26 edges
9. `Host` - 25 edges
10. `EncodeParams` - 24 edges

## Surprising Connections (you probably didn't know these)
- `Web (`web/`)` --references--> `Room`  [INFERRED]
  AGENTS.md → web/src/worker.js
- `Navegador (web)` --references--> `Room`  [INFERRED]
  AGENTS.md → web/src/worker.js
- `run()` --calls--> `outputs()`  [INFERRED]
  crates/livescreen-core/src/viewer.rs → src/audio/playback.rs
- `chat_card()` --references--> `ChatLog`  [EXTRACTED]
  src/ui/share_view.rs → crates/livescreen-core/src/chat.rs
- `e2e_friends()` --calls--> `new_secret_key()`  [INFERRED]
  src/e2e_window.rs → crates/livescreen-core/src/friends.rs

## Import Cycles
- None detected.

## Communities (115 total, 18 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.13
Nodes (20): EncodedFrame, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size() (+12 more)

### Community 1 - "src/avatar.rs"
Cohesion: 0.09
Nodes (28): ALL, all(), auto(), Avatar, BACK, COELHO_SHAPES, COUNT, DARK (+20 more)

### Community 2 - "WebKeys"
Cohesion: 0.20
Nodes (5): derive_ctx(), is_all_zero(), replayed_header_rejected_without_decapsulation(), ValidatedOfferHeader, WebKeys

### Community 3 - "host/mod.rs"
Cohesion: 0.05
Nodes (21): Abr, ABR_FLOOR, answer_presence(), decide_target_fps(), Feeds, HANDSHAKE_TIMEOUT, Host, HostConfig (+13 more)

### Community 4 - "codec_roundtrip.rs"
Cohesion: 0.15
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "convert.rs"
Cohesion: 0.06
Nodes (5): letterbox(), POOL, Target, Nv12, Rgb

### Community 9 - "Avatar.kt"
Cohesion: 0.13
Nodes (13): Device, conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, open_output(), output_loop() (+5 more)

### Community 11 - "protocol.rs"
Cohesion: 0.07
Nodes (12): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, LEGACY_PROTOCOL_VERSION (+4 more)

### Community 12 - "steps"
Cohesion: 0.08
Nodes (24): environment, if, runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy (+16 more)

### Community 13 - "update.rs"
Cohesion: 0.09
Nodes (29): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+21 more)

### Community 14 - "detect_hdr"
Cohesion: 0.24
Nodes (7): detect_hdr(), find_output_for_monitor(), get_monitor_gdi_name(), HdrInfo, hmonitor_for_source(), is_output_hdr(), query_sdr_white_level()

### Community 15 - "speedtest.rs"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (30): app_icon(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), call_title_with_avatar(), card(), field() (+22 more)

### Community 17 - "Ui"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 18 - "gpu.rs"
Cohesion: 0.09
Nodes (3): create_device(), Sendable, winrt_device()

### Community 19 - "host/web.rs"
Cohesion: 0.04
Nodes (25): rtt_congested(), BROWSER_PREDICT, browser_predictions(), browser_predictions_follow_first_v4_srflx(), CONNECT_TIMEOUT, default_v4(), drop_unsafe_candidates(), drop_unsafe_candidates_keeps_lan() (+17 more)

### Community 21 - "with"
Cohesion: 0.08
Nodes (24): GIT_CLIFF_TAG_PATTERN, jobs, release, name, permissions, contents, tags, permissions (+16 more)

### Community 22 - "settings.rs"
Cohesion: 0.24
Nodes (19): audio(), button(), device_combo(), Devices, diagnostics(), experimental(), FORCED, hint() (+11 more)

### Community 23 - "page_check.rs"
Cohesion: 0.08
Nodes (36): keygen(), main(), MANIFEST_PREFIX, pubkey(), sign(), sign_web(), usage(), check() (+28 more)

### Community 24 - "Picker"
Cohesion: 0.08
Nodes (19): SourceInfo, AUTO_QUALITY_COOLDOWN, COPY_FAILED, MAX_ROOM, Picker, Starting, Thumb, to_image() (+11 more)

### Community 25 - "ShareSession"
Cohesion: 0.12
Nodes (14): Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60, P2160p60 (+6 more)

### Community 26 - "watch_view.rs"
Cohesion: 0.10
Nodes (12): I_CHAT, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY (+4 more)

### Community 27 - "share.rs"
Cohesion: 0.09
Nodes (9): cut_feed(), pack_size(), PendingReconf, Reconf, Resources, Sink, SinkTx, start_encoder() (+1 more)

### Community 28 - "steps"
Cohesion: 0.09
Nodes (26): runs-on, steps, runs-on, steps, jobs, build, changelog, web (+18 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (descontinuado), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "hdr.rs"
Cohesion: 0.06
Nodes (7): compile_shader(), DEFAULT_SDR_WHITE_NITS, HLSL_SOURCE, linear_to_srgb(), linear_to_srgb_properties(), sc_rgb_to_srgb(), sc_rgb_to_srgb_keeps_sdr_and_clips_highlights()

### Community 31 - "RFC: sinalização web v3 — ECDH efêmero + híbrido pós-quântico"
Cohesion: 0.09
Nodes (22): 10. Riscos que continuam, 1. Como é hoje (v2), 2. Quem usa e o que cada um arrisca, 3. Primitivas e bibliotecas, 4.1 Material do host, 4.2 Link, 4.3 Publicação da chave pública, 4.4 Offer (espectador → host) (+14 more)

### Community 32 - "pq.js"
Cohesion: 0.07
Nodes (29): RFC-8032, add(), BaseCaseMultiply(), byteSwap(), byteSwap32(), checkOpts(), clone(), _cloneInto() (+21 more)

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "frame_from_body"
Cohesion: 0.29
Nodes (4): frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 35 - "write_msg"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (32): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), parse_secret_key() (+24 more)

### Community 37 - "Updater.kt"
Cohesion: 0.29
Nodes (5): Dib, fit(), thumbnail(), thumbnail_monitor(), thumbnail_window()

### Community 38 - "ui/avatar.rs"
Cohesion: 0.11
Nodes (11): Ink, Accent, Back, Dark, Light, avatar(), color_from_u32(), draw_avatar_item() (+3 more)

### Community 39 - "viewer.rs"
Cohesion: 0.14
Nodes (11): BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64, CLOSE_KICKED_U64, CLOSE_PROTOCOL_U64, close_reason(), CLOSE_STOPPED_U64, DecodeStats (+3 more)

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.06
Nodes (9): Mmcss, Display, GpuFrame, Imported, NEXT_SURFACES_ID, Producer, SLOTS, Surfaces (+1 more)

### Community 48 - "wgc.rs"
Cohesion: 0.11
Nodes (7): border_removable(), DEFAULT_PIXEL_FORMAT, handle_frame(), property_present(), SESSION_TYPE, start(), WgcCapture

### Community 49 - "Target"
Cohesion: 0.20
Nodes (5): AbortJoinSet<T>, Target, Friend, Invite, verify_room_proof()

### Community 50 - "e2e_window.rs"
Cohesion: 0.16
Nodes (5): ChildGuard, e2e_window(), find(), PY, within()

### Community 51 - "encoder.rs"
Cohesion: 0.18
Nodes (10): pick_encoder(), create_mft(), EncoderKind, Hardware, Software, read_seq_header(), spawn(), stream_ids() (+2 more)

### Community 52 - "capture/mod.rs"
Cohesion: 0.15
Nodes (3): CountSink, FirstFrameSink, FrameSink

### Community 53 - "preset.rs"
Cohesion: 0.11
Nodes (27): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not(), lighter_than_stream() (+19 more)

### Community 54 - "ui/friends.rs"
Cohesion: 0.12
Nodes (6): Draft, MAX_LIST, I_ADD_FRIEND, I_COPY, I_PEOPLE, I_TRASH

### Community 56 - "config.rs"
Cohesion: 0.23
Nodes (13): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+5 more)

### Community 57 - "App"
Cohesion: 0.09
Nodes (7): App, classify(), own_hwnd(), open_help(), connect(), sync_visible(), Frame

### Community 58 - "src/web.rs"
Cohesion: 0.08
Nodes (16): ROOM_KEY_LEN, HOST_PK_LEN, lenient_u8(), MIN_SEALED_LEN, MLKEM_CT_LEN, MLKEM_EK_LEN, MLKEM_SK_LEN, NONCE_LEN (+8 more)

### Community 59 - "super"
Cohesion: 0.19
Nodes (6): CpuCopy, Encoder, Feed, Idle, Stop, Nv12Frame

### Community 60 - "ShareSession"
Cohesion: 0.25
Nodes (5): EncodeParams, is_reduction(), ShareSession, unpack_size(), pixel_rate()

### Community 61 - "share.rs"
Cohesion: 0.13
Nodes (6): FAST_REPEATS, black_nv12(), candidates(), hardware_kinds(), run_child(), try_encoder()

### Community 63 - "clipboard.rs"
Cohesion: 0.09
Nodes (6): ClipboardGuard, copy_sensitive(), create_message_window(), publish(), set_data(), set_sensitive_clipboard()

### Community 64 - "playback.rs"
Cohesion: 0.13
Nodes (6): avatar_defaults_to_zero_and_parses(), Experimental, file_overrides_win(), old_ui_json_keeps_new_defaults(), partial_json_keeps_defaults(), Prefs

### Community 65 - "seal_offer_for_test"
Cohesion: 0.36
Nodes (8): padding_makes_sealed_length_equal_for_different_name_lengths(), pre_failure_returns_before_decapsulation(), replay_id_stable_for_same_offer(), roundtrip_offer_and_answer(), seal_offer_for_test(), short_or_invalid_hex_fails(), tamper_each_field_fails(), wrong_web_key_or_different_webkeys_fails()

### Community 66 - "handle_viewer"
Cohesion: 0.15
Nodes (7): admit(), CountGuard, handle_viewer(), Peer, PeerConn, Iroh, Web

### Community 67 - "DxgiCapture"
Cohesion: 0.09
Nodes (18): Amigos e presença, Chat, CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen (+10 more)

### Community 68 - "capture.rs"
Cohesion: 0.08
Nodes (32): AudioHandle, AudioRequest, AudioTarget, Process, System, capture_loop(), DEFAULT_CHECK, default_render_id() (+24 more)

### Community 69 - "AnswerSealer"
Cohesion: 0.28
Nodes (5): AnswerSealer, cross_language_fixture_test(), OpenedOffer, regenerate_cross_language_fixture(), SealedOffer

### Community 70 - "Tab"
Cohesion: 0.25
Nodes (8): Tab, Audio, Diagnostics, Experimental, Share, Updates, Video, TABS

### Community 71 - "signal3.js"
Cohesion: 0.05
Nodes (47): esbuild, @noble/curves, @noble/post-quantum, wrangler, devDependencies, esbuild@0.28.2, @noble/curves@2.4.0, @noble/post-quantum@0.7.1 (+39 more)

### Community 72 - "compute_pre"
Cohesion: 0.29
Nodes (6): check_pre(), compute_pre(), constant_time_eq(), derive_k_pre(), pad_plaintext(), zero_or_low_order_vx_pk_rejected()

### Community 73 - "Callback"
Cohesion: 0.09
Nodes (47): addOpusStereo(), appendChat(), autoAvatar(), bytesToHex(), cleanup(), collectStats(), computeProof(), connect() (+39 more)

### Community 74 - "HostMsg"
Cohesion: 0.29
Nodes (7): chat_msg_json_shape(), HostMsg, Chat, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "src/chat.rs"
Cohesion: 0.09
Nodes (20): CHAT_HISTORY, chat_line_avatar_resolution_and_serde(), ChatLimiter, ChatLine, ChatLog, ChatRole, Host, Viewer (+12 more)

### Community 76 - "controls"
Cohesion: 0.33
Nodes (6): ControlAction, Leave, None, ToggleFullscreen, controls(), watching()

### Community 78 - "sharing"
Cohesion: 0.14
Nodes (22): viewers_for(), Upload, Done, Failed, Measuring, Unknown, begin_start(), chat_card() (+14 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 80 - "ViewerMsg"
Cohesion: 0.33
Nodes (6): ViewerMsg, Chat, Hello, RequestKeyframe, SetQuality, SetVisible

### Community 81 - "WatchStatus"
Cohesion: 0.40
Nodes (5): Phase, Connecting, Ended, Playing, WatchStatus

### Community 83 - "run"
Cohesion: 0.39
Nodes (5): base_url(), init_crypto(), run(), super::Host, host_ws_url()

### Community 84 - "abytes"
Cohesion: 0.10
Nodes (26): aarray2(), abytes(), aexists(), anumber(), aoutput(), clean(), cleanBytes(), copyBytes2() (+18 more)

### Community 85 - "CallTone"
Cohesion: 0.40
Nodes (5): CallTone, Active, Alert, Neutral, Plain

### Community 87 - "mulAddUnsafe"
Cohesion: 0.13
Nodes (21): aarray(), abool(), abool2(), afunction(), aobject2(), asafenumber(), bitLen(), cmov() (+13 more)

### Community 88 - "edwards"
Cohesion: 0.14
Nodes (16): aoddModulus(), assertIsSquare(), createCurveFields(), createField(), edwards(), Field(), FpLegendre(), FpSqrt() (+8 more)

### Community 89 - "start"
Cohesion: 0.22
Nodes (12): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+4 more)

### Community 90 - "mulUnsafe"
Cohesion: 0.14
Nodes (19): assertPoint(), buildWnafTable(), fixedWindowCT(), getWindowSize(), getWnafPrecomputes(), hasWindowSize(), mulCT(), mulCTBlinded() (+11 more)

### Community 91 - "Display"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 93 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

### Community 96 - "montgomery"
Cohesion: 0.19
Nodes (18): aInRange(), bytesToHex(), bytesToNumberBE(), bytesToNumberLE(), cmask(), copyBytes(), createKeygen(), cswap() (+10 more)

### Community 98 - "SocketAddr"
Cohesion: 0.20
Nodes (8): drain_outputs(), host_predictions(), host_predictions_radmin_rule(), maps(), ports(), sa(), Socks, stun_mappings()

### Community 100 - "SignalOut"
Cohesion: 0.29
Nodes (7): SignalOut, Answer, Close, Host, Ping, Pk, Reject

### Community 101 - "files"
Cohesion: 0.12
Nodes (15): files, avatars.json, fonts/inter-400.woff2, fonts/inter-500.woff2, fonts/inter-600.woff2, fonts/LICENSE-Lucide.txt, fonts/OFL-Inter.txt, fonts/OFL-SpaceGrotesk.txt (+7 more)

### Community 103 - "ID3D11Device"
Cohesion: 0.27
Nodes (4): Composer, Grab, GrabSink, texture()

### Community 104 - "OutputResult"
Cohesion: 0.25
Nodes (9): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), input() (+1 more)

### Community 105 - ".ui"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 106 - "ui/friends.rs"
Cohesion: 0.17
Nodes (12): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+4 more)

### Community 110 - "run"
Cohesion: 0.26
Nodes (6): create_duplication(), dxgi_draws_cursor(), DxgiCapture, run(), start(), SinkHandle

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

### Community 118 - "duration"
Cohesion: 0.12
Nodes (10): I_CIRCLE_CHECK, I_INFO, I_WARN, LIFETIME, MAX_VISIBLE, ToastKind, Info, Success (+2 more)

### Community 119 - "bitReversalInplace"
Cohesion: 0.47
Nodes (6): bitReversalInplace(), checkU32(), getZettas(), isPowerOfTwo(), log2(), reverseBits()

### Community 121 - "WebState"
Cohesion: 0.40
Nodes (5): WebState, Connecting, Off, Online, Retrying

### Community 123 - "Codec"
Cohesion: 0.20
Nodes (8): Codec, Av1, H264, Hevc, supported_codecs(), configure(), vbv_bytes(), has_param_sets()

### Community 124 - "controls"
Cohesion: 0.13
Nodes (26): badge(), button(), button_full(), call_button(), checkbox(), draw_button(), ghost_icon_button(), hangup_button() (+18 more)

### Community 126 - "Ink"
Cohesion: 0.47
Nodes (3): variant_bool(), variant_u32(), variant_with()

### Community 128 - "WebVerifyStatus"
Cohesion: 0.40
Nodes (5): WebVerifyStatus, Checking, Failed, Idle, Verified

## Knowledge Gaps
- **122 isolated node(s):** `livescreen`, `livescreen-core`, `Accent`, `Dark`, `Light` (+117 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 867 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **18 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `update.rs`, `Ui`, `settings.rs`, `Picker`, `watch_view.rs`, `src/friends.rs`, `encoder.rs`, `ui/friends.rs`, `WatchSession`, `config.rs`, `ShareSession`, `playback.rs`, `Tab`, `controls`, `ui/mod.rs`, `sharing`, `start`, `OutputResult`, `ui/friends.rs`, `duration`?**
  _High betweenness centrality (0.067) - this node is a cross-community bridge._
- **Are the 12 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 12 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `livescreen-core`, `Accent` to the rest of the system?**
  _122 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1337126600284495 - nodes in this community are weakly interconnected._
- **Why does `SourceId` connect `start` to `Updater.kt`, `dxgi.rs`, `sources.rs`, `ui/mod.rs`, `detect_hdr`, `run`, `wgc.rs`, `sharing`, `HMONITOR`, `e2e_window.rs`, `Picker`, `App`, `share.rs`, `hdr.rs`?**
  _High betweenness centrality (0.038) - this node is a cross-community bridge._
- **Are the 5 inferred relationships involving `Codec` (e.g. with `.take_viewer_decoders()` and `Peer`) actually correct?**
  _`Codec` has 5 INFERRED edges - model-reasoned connections that need verification._
- **Should `src/avatar.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08712121212121213 - nodes in this community are weakly interconnected._