# Graph Report - LiveScreen-publish  (2026-11-02)

## Corpus Check
- 74 files · ~117,411 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 57 file(s) not represented in the graph (top: .so 14, .woff2 10, .bin 7)

## Summary
- 2307 nodes · 4593 edges · 114 communities (92 shown, 22 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 120 edges (avg confidence: 0.73)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `770eb2bd`
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
- Android (`android/`)
- mf.rs
- protocol.rs
- steps
- update.rs
- detect_hdr
- speedtest.rs
- theme.rs
- AudioEncoder
- gpu.rs
- host/web.rs
- Host
- with
- HWND
- page_check.rs
- share_view.rs
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
- Dib
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
- GpuOut
- App
- src/web.rs
- Mode
- inspect_window
- share.rs
- Mic
- clipboard.rs
- config.rs
- linear_to_srgb
- handle_viewer
- DxgiCapture
- capture.rs
- ChatLimiter
- compile_shader
- signal3.js
- Callback
- HostMsg
- src/chat.rs
- ui/mod.rs
- sharing
- pages.yml
- run
- abytes
- mulAddUnsafe
- edwards
- start
- mulUnsafe
- OutputResult
- Source
- AudioState
- montgomery
- SocketAddr
- SignalOut
- files
- ID3D11Device
- input
- read_video_frame
- ui/friends.rs
- EndpointId
- run
- enumerate_mfts
- SignalIn
- HMONITOR
- isPosBig
- capture_loop
- duration
- bitReversalInplace
- WebState
- Codec
- Ui
- AudioHandle
- variant_with
- WebVerifyStatus
- BudgetExceeded

## God Nodes (most connected - your core abstractions)
1. `App` - 74 edges
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
- `has_param_sets()` --references--> `Codec`  [EXTRACTED]
  src/video/mf.rs → crates/livescreen-core/src/protocol.rs
- `ink_color()` --references--> `Ink`  [EXTRACTED]
  src/ui/avatar.rs → crates/livescreen-core/src/avatar.rs
- `chat_card()` --references--> `ChatLog`  [EXTRACTED]
  src/ui/share_view.rs → crates/livescreen-core/src/chat.rs

## Import Cycles
- None detected.

## Communities (114 total, 22 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.16
Nodes (18): EncodedFrame, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size() (+10 more)

### Community 1 - "src/avatar.rs"
Cohesion: 0.07
Nodes (33): ALL, all(), auto(), Avatar, BACK, COELHO_SHAPES, COUNT, DARK (+25 more)

### Community 2 - "WebKeys"
Cohesion: 0.11
Nodes (18): AnswerSealer, cross_language_fixture_test(), derive_ctx(), is_all_zero(), pad_plaintext(), padding_makes_sealed_length_equal_for_different_name_lengths(), pre_failure_returns_before_decapsulation(), regenerate_cross_language_fixture() (+10 more)

### Community 3 - "host/mod.rs"
Cohesion: 0.09
Nodes (16): Abr, ABR_FLOOR, answer_presence(), decide_target_fps(), HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips() (+8 more)

### Community 4 - "codec_roundtrip.rs"
Cohesion: 0.13
Nodes (17): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+9 more)

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 9 - "Android (`android/`)"
Cohesion: 0.13
Nodes (10): conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, output_loop(), PlaybackHandle, PREBUFFER_FLOATS (+2 more)

### Community 10 - "mf.rs"
Cohesion: 0.12
Nodes (3): has_param_sets(), sample_survives_need_more_input(), startup()

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
Nodes (34): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), call_title_with_avatar(), card() (+26 more)

### Community 17 - "AudioEncoder"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 18 - "gpu.rs"
Cohesion: 0.09
Nodes (3): create_device(), Sendable, winrt_device()

### Community 19 - "host/web.rs"
Cohesion: 0.04
Nodes (25): rtt_congested(), BROWSER_PREDICT, browser_predictions(), browser_predictions_follow_first_v4_srflx(), CONNECT_TIMEOUT, default_v4(), drop_unsafe_candidates(), drop_unsafe_candidates_keeps_lan() (+17 more)

### Community 20 - "Host"
Cohesion: 0.11
Nodes (3): Host, invite_addr(), Overload

### Community 21 - "with"
Cohesion: 0.08
Nodes (24): GIT_CLIFF_TAG_PATTERN, jobs, release, name, permissions, contents, tags, permissions (+16 more)

### Community 22 - "HWND"
Cohesion: 0.36
Nodes (6): fit(), source_size(), thumbnail(), thumbnail_monitor(), thumbnail_window(), window_rect()

### Community 23 - "page_check.rs"
Cohesion: 0.08
Nodes (36): keygen(), main(), MANIFEST_PREFIX, pubkey(), sign(), sign_web(), usage(), check() (+28 more)

### Community 24 - "share_view.rs"
Cohesion: 0.08
Nodes (20): AUTO_QUALITY_COOLDOWN, begin_start(), COPY_FAILED, picker(), poll_starting(), Preview, source_card(), spawn_share_job() (+12 more)

### Community 25 - "ShareSession"
Cohesion: 0.12
Nodes (14): Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60, P2160p60 (+6 more)

### Community 26 - "start_encoder"
Cohesion: 0.22
Nodes (3): Reconf, start_encoder(), EncoderHandle

### Community 27 - "share.rs"
Cohesion: 0.10
Nodes (9): cut_feed(), pack_size(), PendingReconf, pick_encoder(), Resources, ShareSession, Sink, SinkTx (+1 more)

### Community 28 - "steps"
Cohesion: 0.09
Nodes (26): runs-on, steps, runs-on, steps, jobs, build, changelog, web (+18 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (descontinuado), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

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

### Community 48 - "wgc.rs"
Cohesion: 0.10
Nodes (7): border_removable(), DEFAULT_PIXEL_FORMAT, handle_frame(), property_present(), SESSION_TYPE, start(), WgcCapture

### Community 49 - "Target"
Cohesion: 0.20
Nodes (5): AbortJoinSet<T>, Target, Friend, Invite, verify_room_proof()

### Community 50 - "e2e_window.rs"
Cohesion: 0.16
Nodes (5): ChildGuard, e2e_window(), find(), PY, within()

### Community 51 - "encoder.rs"
Cohesion: 0.23
Nodes (7): HostStats, configure(), create_mft(), read_seq_header(), spawn(), stream_ids(), vbv_bytes()

### Community 52 - "capture/mod.rs"
Cohesion: 0.16
Nodes (4): CountSink, FirstFrameSink, FrameSink, SinkHandle

### Community 53 - "preset.rs"
Cohesion: 0.10
Nodes (30): bitrate(), encode_params(), EncodeParams, extreme_shapes_never_break_limits(), fit_level(), is_reduction(), label(), light_presets_are_reductions_and_fall_back() (+22 more)

### Community 54 - "ui/friends.rs"
Cohesion: 0.06
Nodes (16): Draft, MAX_LIST, I_ADD_FRIEND, I_CHAT, I_COPY, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP (+8 more)

### Community 56 - "GpuOut"
Cohesion: 0.40
Nodes (3): GpuOut, Readback, Shared

### Community 57 - "App"
Cohesion: 0.05
Nodes (39): App, classify(), own_hwnd(), button(), diagnostics(), experimental(), FORCED, hint() (+31 more)

### Community 58 - "src/web.rs"
Cohesion: 0.07
Nodes (22): ROOM_KEY_LEN, check_pre(), compute_pre(), constant_time_eq(), derive_k_pre(), HOST_PK_LEN, lenient_u8(), MIN_SEALED_LEN (+14 more)

### Community 59 - "Mode"
Cohesion: 0.18
Nodes (7): CpuCopy, Encoder, Feed, Frame, Idle, Stop, Nv12Frame

### Community 60 - "inspect_window"
Cohesion: 0.50
Nodes (3): exe_name(), inspect_window(), window_cb()

### Community 61 - "share.rs"
Cohesion: 0.12
Nodes (11): EncoderKind, Hardware, Software, FAST_REPEATS, black_nv12(), candidates(), detect(), hardware_kinds() (+3 more)

### Community 62 - "Mic"
Cohesion: 0.50
Nodes (4): Mic, Default, Device, open_mic()

### Community 63 - "clipboard.rs"
Cohesion: 0.09
Nodes (6): ClipboardGuard, copy_sensitive(), create_message_window(), publish(), set_data(), set_sensitive_clipboard()

### Community 64 - "config.rs"
Cohesion: 0.09
Nodes (19): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+11 more)

### Community 65 - "linear_to_srgb"
Cohesion: 0.50
Nodes (4): linear_to_srgb(), linear_to_srgb_properties(), sc_rgb_to_srgb(), sc_rgb_to_srgb_keeps_sdr_and_clips_highlights()

### Community 66 - "handle_viewer"
Cohesion: 0.14
Nodes (5): admit(), CountGuard, Feeds, handle_viewer(), Shared

### Community 67 - "DxgiCapture"
Cohesion: 0.09
Nodes (18): Amigos e presença, Chat, CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen (+10 more)

### Community 68 - "capture.rs"
Cohesion: 0.12
Nodes (11): FRAME_BYTES, FRAME_FLOATS, FRAME_SAMPLES, MAX_LAG_BYTES, mix(), mix_clamps(), OPUS_MAX_PACKET, take_frame() (+3 more)

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
Nodes (18): CHAT_HISTORY, chat_line_avatar_resolution_and_serde(), ChatLine, ChatLog, ChatRole, Host, Viewer, clean_chat() (+10 more)

### Community 78 - "sharing"
Cohesion: 0.20
Nodes (17): ShareStatus, viewers_for(), Upload, Done, Failed, Measuring, Unknown, chat_card() (+9 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 83 - "run"
Cohesion: 0.21
Nodes (6): base_url(), drain_outputs(), init_crypto(), run(), super::Host, host_ws_url()

### Community 84 - "abytes"
Cohesion: 0.10
Nodes (26): aarray2(), abytes(), aexists(), anumber(), aoutput(), clean(), cleanBytes(), copyBytes2() (+18 more)

### Community 87 - "mulAddUnsafe"
Cohesion: 0.13
Nodes (21): aarray(), abool(), abool2(), afunction(), aobject2(), asafenumber(), bitLen(), cmov() (+13 more)

### Community 88 - "edwards"
Cohesion: 0.14
Nodes (16): aoddModulus(), assertIsSquare(), createCurveFields(), createField(), edwards(), Field(), FpLegendre(), FpSqrt() (+8 more)

### Community 89 - "start"
Cohesion: 0.19
Nodes (15): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+7 more)

### Community 90 - "mulUnsafe"
Cohesion: 0.14
Nodes (19): assertPoint(), buildWnafTable(), fixedWindowCT(), getWindowSize(), getWnafPrecomputes(), hasWindowSize(), mulCT(), mulCTBlinded() (+11 more)

### Community 91 - "OutputResult"
Cohesion: 0.25
Nodes (8): is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample, process_output(), sample_bytes()

### Community 92 - "Source"
Cohesion: 0.22
Nodes (7): AudioTarget, Process, System, open(), open_loopback(), pump(), Source

### Community 93 - "AudioState"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

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

### Community 104 - "input"
Cohesion: 0.25
Nodes (9): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), input() (+1 more)

### Community 105 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 106 - "ui/friends.rs"
Cohesion: 0.17
Nodes (12): ended(), footer(), home(), left_column(), pill(), share_tile(), update_prompt(), I_CHEVRON (+4 more)

### Community 111 - "enumerate_mfts"
Cohesion: 0.22
Nodes (4): enumerate_mfts(), friendly_name(), set_codec_value(), subtype()

### Community 112 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 114 - "HMONITOR"
Cohesion: 0.40
Nodes (7): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate()

### Community 116 - "isPosBig"
Cohesion: 0.29
Nodes (8): abignumber(), asciiToBase16(), hexToBytes(), isPosBig(), numberToBytesBE(), numberToBytesLE(), toBytes(), validateMSMScalars()

### Community 117 - "capture_loop"
Cohesion: 0.50
Nodes (6): AudioRequest, capture_loop(), mic_error_text(), Opened, run(), start()

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
Cohesion: 0.17
Nodes (11): Codec, Av1, H264, Hevc, ViewerMsg, Chat, Hello, RequestKeyframe (+3 more)

### Community 124 - "Ui"
Cohesion: 0.12
Nodes (27): button(), button_full(), call_button(), CallTone, Active, Alert, Neutral, Plain (+19 more)

### Community 126 - "variant_with"
Cohesion: 0.47
Nodes (3): variant_bool(), variant_u32(), variant_with()

### Community 128 - "WebVerifyStatus"
Cohesion: 0.40
Nodes (5): WebVerifyStatus, Checking, Failed, Idle, Verified

## Knowledge Gaps
- **122 isolated node(s):** `livescreen`, `livescreen-core`, `Accent`, `Dark`, `Light` (+117 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 864 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **22 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `config.rs`, `src/friends.rs`, `input`, `ui/friends.rs`, `ui/mod.rs`, `sharing`, `update.rs`, `AudioEncoder`, `WatchSession`, `ui/friends.rs`, `duration`, `share_view.rs`, `start`, `share.rs`, `share.rs`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Are the 12 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 12 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `livescreen-core`, `Accent` to the rest of the system?**
  _122 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `src/avatar.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06923076923076923 - nodes in this community are weakly interconnected._
- **Why does `SourceId` connect `start` to `ID3D11Device`, `sources.rs`, `dxgi.rs`, `ui/mod.rs`, `detect_hdr`, `sharing`, `wgc.rs`, `HMONITOR`, `e2e_window.rs`, `HWND`, `share_view.rs`, `App`, `share.rs`, `hdr.rs`?**
  _High betweenness centrality (0.033) - this node is a cross-community bridge._
- **Are the 5 inferred relationships involving `Codec` (e.g. with `.take_viewer_decoders()` and `Peer`) actually correct?**
  _`Codec` has 5 INFERRED edges - model-reasoned connections that need verification._
- **Should `WebKeys` be split into smaller, more focused modules?**
  _Cohesion score 0.11290322580645161 - nodes in this community are weakly interconnected._