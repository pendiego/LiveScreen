# Graph Report - LiveScreen-publish  (2026-10-28)

## Corpus Check
- 68 files · ~82,748 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 51 file(s) not represented in the graph (top: .xml 22, .woff2 10, .ttf 7)

## Summary
- 2022 nodes · 4030 edges · 98 communities (76 shown, 22 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 110 edges (avg confidence: 0.76)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `515e1576`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- decoder.rs
- config.rs
- capture.rs
- steps
- ToastKind
- convert.rs
- App
- dxgi.rs
- sources.rs
- mf.rs
- protocol.rs
- Display
- update.rs
- ViewerModel
- playback.rs
- theme.rs
- watch_view.rs
- gpu.rs
- run
- probe.rs
- with
- ui/mod.rs
- Android (`android/`)
- SourceInfo
- ShareSession
- Shared
- share.rs
- steps
- LiveScreen
- Encoder
- HMONITOR
- VideoDecoder
- test-changelog.sh
- rust/src/lib.rs
- Native
- src/friends.rs
- frame_from_body
- LiveScreen
- main.rs
- present.rs
- icon.rs
- livescreen-core
- Sendable<IDirect3DDevice>
- wgc.rs
- codec_roundtrip.rs
- Native
- encoder.rs
- Theme.kt
- preset.rs
- Dib
- format_invite
- viewer.rs
- capture/mod.rs
- BudgetExceeded
- Resources
- encoder.rs
- Preset
- Callback
- omp config sync via private GitHub repo
- UpdateInfo
- HomeScreen
- ViewerModel
- DxgiCapture
- BtnKind
- read_video_frame
- handle_frame
- AudioPlayer
- Callback
- HostMsg
- WgcCapture
- start
- share_view.rs
- pages.yml
- watch_view.rs
- capture_loop
- peer
- run
- SignalOut
- CallState
- SinalSnack
- .start
- Mmcss
- Target
- SignalIn
- FieldTone
- BudgetExceeded
- WebPeer
- AudioTarget

## God Nodes (most connected - your core abstractions)
1. `App` - 73 edges
2. `Preset` - 48 edges
3. `ViewerModel` - 41 edges
4. `Converter` - 33 edges
5. `ShareSession` - 32 edges
6. `Codec` - 29 edges
7. `EncoderKind` - 26 edges
8. `SourceId` - 25 edges
9. `EncodeParams` - 24 edges
10. `Shared` - 24 edges

## Surprising Connections (you probably didn't know these)
- `Android (`android/`)` --references--> `AudioPlayer`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/AudioPlayer.kt
- `Android (`android/`)` --references--> `MainActivity`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/MainActivity.kt
- `Android (`android/`)` --references--> `Native`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/Native.kt
- `Android (`android/`)` --references--> `VideoDecoder`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/VideoDecoder.kt
- `Android (`android/`)` --references--> `ViewerModel`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/ViewerModel.kt

## Import Cycles
- None detected.

## Communities (98 total, 22 thin omitted)

### Community 0 - "decoder.rs"
Cohesion: 0.13
Nodes (21): EncodedFrame, DecodeStats, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain() (+13 more)

### Community 1 - "config.rs"
Cohesion: 0.10
Nodes (17): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+9 more)

### Community 2 - "capture.rs"
Cohesion: 0.12
Nodes (10): conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, output_loop(), PlaybackHandle, PREBUFFER_FLOATS (+2 more)

### Community 3 - "steps"
Cohesion: 0.10
Nodes (20): runs-on, steps, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, jobs, deploy, name, branches (+12 more)

### Community 4 - "ToastKind"
Cohesion: 0.14
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "convert.rs"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 7 - "dxgi.rs"
Cohesion: 0.05
Nodes (13): Composer, create_duplication(), dxgi_draws_cursor(), DxgiCapture, Grab, GrabSink, run(), start() (+5 more)

### Community 10 - "mf.rs"
Cohesion: 0.07
Nodes (17): enumerate_mfts(), friendly_name(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample (+9 more)

### Community 11 - "protocol.rs"
Cohesion: 0.09
Nodes (12): ALPN, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED, LEGACY_PROTOCOL_VERSION (+4 more)

### Community 12 - "Display"
Cohesion: 0.18
Nodes (3): Display, GpuFrame, Imported

### Community 13 - "update.rs"
Cohesion: 0.10
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "ViewerModel"
Cohesion: 0.18
Nodes (3): Friend, Status, ViewerModel

### Community 15 - "playback.rs"
Cohesion: 0.06
Nodes (30): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+22 more)

### Community 16 - "theme.rs"
Cohesion: 0.05
Nodes (57): app_icon(), badge(), button(), button_full(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_button() (+49 more)

### Community 17 - "watch_view.rs"
Cohesion: 0.06
Nodes (23): Draft, MAX_LIST, ended(), footer(), home(), left_column(), pill(), share_tile() (+15 more)

### Community 18 - "gpu.rs"
Cohesion: 0.08
Nodes (3): create_device(), Sendable, winrt_device()

### Community 19 - "run"
Cohesion: 0.07
Nodes (5): CONNECT_TIMEOUT, INIT_CRYPTO, MAX_SDP, STUN_SERVER, WEB_BASE

### Community 20 - "probe.rs"
Cohesion: 0.29
Nodes (4): frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 21 - "with"
Cohesion: 0.06
Nodes (33): runs-on, steps, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release, name (+25 more)

### Community 23 - "Android (`android/`)"
Cohesion: 0.27
Nodes (4): Android (`android/`), GhAsset, Updater, UpdaterTest

### Community 24 - "SourceInfo"
Cohesion: 0.32
Nodes (5): exe_name(), inspect_window(), source_size(), window_cb(), window_rect()

### Community 25 - "ShareSession"
Cohesion: 0.12
Nodes (14): Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60, P2160p60 (+6 more)

### Community 26 - "Shared"
Cohesion: 0.40
Nodes (7): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate()

### Community 27 - "share.rs"
Cohesion: 0.08
Nodes (15): Abr, ABR_FLOOR, cut_feed(), HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), MAX_HANDSHAKES (+7 more)

### Community 28 - "steps"
Cohesion: 0.09
Nodes (29): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+21 more)

### Community 29 - "LiveScreen"
Cohesion: 0.15
Nodes (12): Android (só para assistir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "Encoder"
Cohesion: 0.16
Nodes (12): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+4 more)

### Community 31 - "HMONITOR"
Cohesion: 0.15
Nodes (24): button(), diagnostics(), FORCED, hint(), icon_button(), open_help(), open_logs(), SETTINGS_LABEL (+16 more)

### Community 33 - "test-changelog.sh"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "rust/src/lib.rs"
Cohesion: 0.08
Nodes (33): AbortOnDrop, __android_log_write(), audio_loop(), get_session(), init_logging(), INIT_LOGGING, Java_io_github_pendiego_livescreen_Native_decoderFailed(), Java_io_github_pendiego_livescreen_Native_findInvite() (+25 more)

### Community 35 - "Native"
Cohesion: 0.12
Nodes (4): Amigos e presença, Regras, Native, Presence

### Community 36 - "src/friends.rs"
Cohesion: 0.06
Nodes (31): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), Presence (+23 more)

### Community 38 - "frame_from_body"
Cohesion: 0.10
Nodes (7): border_removable(), handle_frame(), PIXEL_FORMAT, property_present(), SESSION_TYPE, start(), WgcCapture

### Community 39 - "LiveScreen"
Cohesion: 0.20
Nodes (9): CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código, Núcleo (`crates/livescreen-core/src/`) (+1 more)

### Community 40 - "main.rs"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "present.rs"
Cohesion: 0.10
Nodes (5): NEXT_SURFACES_ID, Producer, SLOTS, Surfaces, Target

### Community 44 - "livescreen-core"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "wgc.rs"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 49 - "codec_roundtrip.rs"
Cohesion: 0.24
Nodes (4): AbortJoinSet<T>, Target, Friend, Invite

### Community 50 - "Native"
Cohesion: 0.17
Nodes (12): classify(), EndKind, Error, Info, Warn, Screen, Ended, Home (+4 more)

### Community 51 - "encoder.rs"
Cohesion: 0.15
Nodes (6): EncoderStats, black_nv12(), candidates(), hardware_kinds(), run_child(), try_encoder()

### Community 52 - "Theme.kt"
Cohesion: 0.10
Nodes (4): MainActivity, inter(), LiveScreenTheme(), Sinal

### Community 53 - "preset.rs"
Cohesion: 0.12
Nodes (26): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not(), LIMIT_FPS (+18 more)

### Community 54 - "Dib"
Cohesion: 0.29
Nodes (5): Dib, fit(), thumbnail(), thumbnail_monitor(), thumbnail_window()

### Community 55 - "format_invite"
Cohesion: 0.19
Nodes (11): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), verify_room_proof() (+3 more)

### Community 56 - "viewer.rs"
Cohesion: 0.05
Nodes (21): ViewerMsg, Hello, RequestKeyframe, SetQuality, SetVisible, BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64 (+13 more)

### Community 57 - "capture/mod.rs"
Cohesion: 0.30
Nodes (9): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), selftest() (+1 more)

### Community 58 - "BudgetExceeded"
Cohesion: 0.11
Nodes (10): ROOM_KEY_LEN, stun_binding_request(), STUN_MAGIC_COOKIE, stun_mapped_addr(), test_stun_binding_request(), test_stun_rfc5769_vector(), test_viewer_url_no_duplicate_slash(), test_web_proof_verification() (+2 more)

### Community 59 - "Resources"
Cohesion: 0.12
Nodes (14): EncodeParams, configure(), CpuCopy, create_mft(), Encoder, Feed, Frame, Idle (+6 more)

### Community 60 - "encoder.rs"
Cohesion: 0.10
Nodes (12): sanitize(), admit(), answer_presence(), CountGuard, handle_viewer(), Peer, PeerConn, Iroh (+4 more)

### Community 63 - "omp config sync via private GitHub repo"
Cohesion: 0.12
Nodes (11): FRAME_BYTES, FRAME_FLOATS, FRAME_SAMPLES, MAX_LAG_BYTES, mix(), mix_clamps(), OPUS_MAX_PACKET, take_frame() (+3 more)

### Community 64 - "UpdateInfo"
Cohesion: 0.36
Nodes (7): UpdateInfo, Available, Downloading, Failed, Idle, Ready, UpdateState

### Community 65 - "HomeScreen"
Cohesion: 0.23
Nodes (13): AddFriendDialog(), App(), Badge(), CallButton(), clipboardText(), FriendsSection(), HomeScreen(), SinalButton() (+5 more)

### Community 66 - "ViewerModel"
Cohesion: 0.19
Nodes (3): is_reduction(), ShareSession, unpack_size()

### Community 67 - "DxgiCapture"
Cohesion: 0.23
Nodes (4): bytesToHex(), hexToBytes(), iceServers(), Room

### Community 68 - "BtnKind"
Cohesion: 0.40
Nodes (5): BtnKind, Danger, Ghost, Primary, Secondary

### Community 69 - "read_video_frame"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 71 - "handle_frame"
Cohesion: 0.20
Nodes (10): wrangler, devDependencies, wrangler@^4, name, private, scripts, check, deploy (+2 more)

### Community 72 - "AudioPlayer"
Cohesion: 0.53
Nodes (6): SourceId, Monitor, Window, SourceInfo, Picker, Update

### Community 73 - "Callback"
Cohesion: 0.19
Nodes (16): addOpusStereo(), bytesToHex(), cleanup(), computeProof(), connect(), hexToBytes(), refreshStats(), row() (+8 more)

### Community 74 - "HostMsg"
Cohesion: 0.40
Nodes (5): HostMsg, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "WgcCapture"
Cohesion: 0.22
Nodes (7): Mic, Default, Device, open(), open_mic(), pump(), Source

### Community 77 - "start"
Cohesion: 0.25
Nodes (6): spawn_share_job(), ToastKind, Info, Success, Warn, Toasts

### Community 78 - "share_view.rs"
Cohesion: 0.06
Nodes (44): viewers_for(), AudioState, Off, On, Unavailable, ShareStatus, WebState, Connecting (+36 more)

### Community 79 - "pages.yml"
Cohesion: 0.08
Nodes (24): concurrency, cancel-in-progress, group, environment, runs-on, steps, name, url (+16 more)

### Community 83 - "watch_view.rs"
Cohesion: 0.07
Nodes (24): CallTone, Active, Alert, Neutral, Plain, I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP (+16 more)

### Community 84 - "capture_loop"
Cohesion: 0.50
Nodes (6): AudioRequest, capture_loop(), mic_error_text(), Opened, run(), start()

### Community 85 - "peer"
Cohesion: 0.25
Nodes (3): rtt_congested(), drain_outputs(), peer()

### Community 86 - "run"
Cohesion: 0.33
Nodes (6): host_ws_url(), test_host_ws_url(), base_url(), init_crypto(), run(), super::ShareSession

### Community 87 - "SignalOut"
Cohesion: 0.33
Nodes (6): SignalOut, Answer, Close, Host, Ping, Reject

### Community 88 - "CallState"
Cohesion: 0.40
Nodes (5): CallState, Active, Alert, HangUp, Neutral

### Community 89 - "SinalSnack"
Cohesion: 0.40
Nodes (4): SinalSnack, SnackTone, Error, Success

### Community 90 - ".start"
Cohesion: 0.15
Nodes (4): invite_addr(), Overload, pack_size(), ViewerInfo

### Community 93 - "SignalIn"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 94 - "FieldTone"
Cohesion: 0.50
Nodes (4): FieldTone, Error, Normal, Warn

### Community 97 - "AudioTarget"
Cohesion: 0.50
Nodes (4): AudioTarget, Process, System, open_loopback()

## Knowledge Gaps
- **114 isolated node(s):** `livescreen`, `Primary`, `Secondary`, `Ghost`, `Danger` (+109 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 795 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **22 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App` to `config.rs`, `ViewerModel`, `AudioPlayer`, `start`, `share_view.rs`, `update.rs`, `watch_view.rs`, `Native`, `watch_view.rs`, `ui/mod.rs`, `format_invite`, `viewer.rs`, `Encoder`, `HMONITOR`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `App` (e.g. with `Upload` and `Draft`) actually correct?**
  _`App` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `livescreen`, `Primary`, `Secondary` to the rest of the system?**
  _114 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `decoder.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12692307692307692 - nodes in this community are weakly interconnected._
- **Why does `ViewerModel` connect `ViewerModel` to `VideoDecoder`, `HomeScreen`, `UpdateInfo`, `Native`, `ViewerModel.kt`, `Theme.kt`, `Android (`android/`)`, `Preset`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **Should `config.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09841269841269841 - nodes in this community are weakly interconnected._
- **Why does `Preset` connect `ShareSession` to `config.rs`, `ViewerModel`, `AudioPlayer`, `share_view.rs`, `preset.rs`, `viewer.rs`, `.start`, `share.rs`, `encoder.rs`, `HMONITOR`?**
  _High betweenness centrality (0.037) - this node is a cross-community bridge._