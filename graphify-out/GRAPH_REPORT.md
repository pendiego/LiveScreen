# Graph Report - LiveScreen  (2026-10-28)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 1940 nodes · 3912 edges · 100 communities (75 shown, 25 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 72 edges (avg confidence: 0.87)
- Token cost: 80,312 input · 1,209 output

## Graph Freshness
- Built from commit: `c5672128`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Video Decoder
- Config and Identity
- Audio Playback
- Share Status UI
- Codec Roundtrip Tests
- D3D11 Video Conversion
- Desktop App Logic
- Window Capture GDI
- E2E Window Tests
- Media Foundation Enumeration
- Wire Protocol
- Encoder Feed Sink
- Build Script Icons
- Android Viewer Model
- Speed Test
- UI Theme Styling
- Invite Parsing Layout
- D3D11 Device Setup
- WebRTC Signaling
- Video Frame Wire
- Release Workflow
- Window Monitor Enumeration
- UI Buttons
- Window Inspection
- Encode Presets
- Monitor Enumeration
- Share Room Management
- CI Workflow
- README Documentation
- Codec Selection
- Settings Modal
- Capture Frame Sinks
- Changelog Test Script
- Android JNI Bridge
- Native JNI Declarations
- Friend Presence Protocol
- Project Documentation Guide
- WGC Interop Setup
- DXGI Desktop Duplication
- Main Entry Allocator
- GPU Fence Synchronization
- Icon Rendering
- Workspace Crates
- Sendable D3D Device
- Message Stream IO
- Invite Target Parsing
- App Screen States
- Video Encoder
- Main Activity Theme
- Quality Presets
- Thumbnail Bitmap Capture
- Invite Format Proof
- Viewer Session Messages
- Capture Backend Selection
- Web STUN Signaling
- Async Encoder Events
- Viewer Admission Handling
- Android Audio Player
- Surface Callbacks
- Audio Capture Mixing
- Abort Join Set
- Compose Home Screen
- Share Session Control
- Cloudflare Worker Rooms
- Encoder Handle Resources
- Frame Budget Limits
- Android Video Decoder
- Wrangler Package Config
- Capture Source Info
- Web Viewer Script
- Host Message Types
- Microphone Capture
- Toast Notifications
- Share View UI
- UI Icons
- Audio Capture Loop
- RTC Peer Events
- Host WebSocket URL
- Signal Outgoing Messages
- Widget Painting
- Capture Frame Handler
- Host Endpoint Info
- DXGI Capture Handle
- Audio Handle Shutdown
- Signal Incoming Messages
- WGC Capture Session
- Web Connection State
- Web Peer Notify
- Audio Loopback Target
- Border Removal Check
- Audio State

## God Nodes (most connected - your core abstractions)
1. `App` - 73 edges
2. `Preset` - 49 edges
3. `ViewerModel` - 42 edges
4. `Converter` - 33 edges
5. `ShareSession` - 32 edges
6. `Codec` - 29 edges
7. `EncoderKind` - 26 edges
8. `SourceId` - 25 edges
9. `EncodeParams` - 24 edges
10. `Shared` - 24 edges

## Surprising Connections (you probably didn't know these)
- `Android (`android/`)` --references--> `ViewerModel`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/ViewerModel.kt
- `Android (`android/`)` --references--> `Native`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/Native.kt
- `Android (`android/`)` --references--> `MainActivity`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/MainActivity.kt
- `Android (`android/`)` --references--> `AudioPlayer`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/AudioPlayer.kt
- `Android (`android/`)` --references--> `VideoDecoder`  [INFERRED]
  AGENTS.md → android/app/src/main/java/io/github/pendiego/livescreen/VideoDecoder.kt

## Import Cycles
- None detected.

## Communities (100 total, 25 thin omitted)

### Community 0 - "Video Decoder"
Cohesion: 0.13
Nodes (20): EncodedFrame, as_bytes(), attach_gpu(), convert_planes(), create(), create_mft(), drain(), frame_size() (+12 more)

### Community 1 - "Config and Identity"
Cohesion: 0.10
Nodes (17): app_dir(), Config, Identity, identity(), identity_persistence_and_bad_recovery(), load(), load_identity(), Overrides (+9 more)

### Community 2 - "Audio Playback"
Cohesion: 0.12
Nodes (10): conceal(), conceal_matches_lost_duration(), DECODE_FLOATS, decode_loop(), MAX_BUFFER_FLOATS, output_loop(), PlaybackHandle, PREBUFFER_FLOATS (+2 more)

### Community 3 - "Share Status UI"
Cohesion: 0.22
Nodes (16): ShareStatus, Upload, Done, Failed, Measuring, Unknown, confirm_modal(), invite_card() (+8 more)

### Community 4 - "Codec Roundtrip Tests"
Cohesion: 0.12
Nodes (18): bgra(), codec_roundtrip(), collect(), create_bgra(), fingerprint(), forward(), H, luma() (+10 more)

### Community 5 - "D3D11 Video Conversion"
Cohesion: 0.05
Nodes (7): Converter, create_output(), letterbox(), POOL, Target, Nv12, Rgb

### Community 6 - "Desktop App Logic"
Cohesion: 0.11
Nodes (4): App, classify(), own_hwnd(), open_help()

### Community 9 - "E2E Window Tests"
Cohesion: 0.16
Nodes (9): ChildGuard, e2e_android(), e2e_window(), find(), find_text(), PY, tap_text(), tap_xy() (+1 more)

### Community 10 - "Media Foundation Enumeration"
Cohesion: 0.07
Nodes (17): enumerate_mfts(), friendly_name(), is_async(), OutputResult, NeedMoreInput, Sample, StreamChange, OutSample (+9 more)

### Community 11 - "Wire Protocol"
Cohesion: 0.08
Nodes (13): ALPN, BudgetExceeded, CLOSE_DENIED, CLOSE_FULL, CLOSE_KICKED, CLOSE_NORMAL, CLOSE_PROTOCOL, CLOSE_STOPPED (+5 more)

### Community 12 - "Encoder Feed Sink"
Cohesion: 0.20
Nodes (3): cut_feed(), pack_size(), Sink

### Community 13 - "Build Script Icons"
Cohesion: 0.10
Nodes (25): dib(), main(), push_entry(), res_file(), RT_GROUP_ICON, RT_ICON, SIZES, API (+17 more)

### Community 14 - "Android Viewer Model"
Cohesion: 0.08
Nodes (4): Friend, Presence, Status, ViewerModel

### Community 15 - "Speed Test"
Cohesion: 0.10
Nodes (21): conn(), connection_dropped_mid_upload_fails_measurement(), http_error_fails_measurement(), MAX_BYTES, MAX_CHUNK, measure(), measure_upload(), MIN_CHUNK (+13 more)

### Community 16 - "UI Theme Styling"
Cohesion: 0.05
Nodes (33): app_icon(), badge(), call_bar(), CALL_BAR_BOTTOM, CALL_BAR_CLEARANCE, call_title(), card(), field() (+25 more)

### Community 17 - "Invite Parsing Layout"
Cohesion: 0.06
Nodes (22): Draft, MAX_LIST, ended(), footer(), home(), left_column(), share_tile(), update_prompt() (+14 more)

### Community 19 - "WebRTC Signaling"
Cohesion: 0.07
Nodes (5): CONNECT_TIMEOUT, INIT_CRYPTO, MAX_SDP, STUN_SERVER, WEB_BASE

### Community 20 - "Video Frame Wire"
Cohesion: 0.29
Nodes (4): frame_from_body(), parse_video_body(), roundtrip(), video_frame_header()

### Community 21 - "Release Workflow"
Cohesion: 0.07
Nodes (32): runs-on, steps, KS, LIVESCREEN_KEYSTORE_PASSWORD, jobs, android, release, name (+24 more)

### Community 23 - "UI Buttons"
Cohesion: 0.12
Nodes (27): button(), button_full(), call_button(), CallTone, Active, Alert, Neutral, Plain (+19 more)

### Community 24 - "Window Inspection"
Cohesion: 0.32
Nodes (5): exe_name(), inspect_window(), source_size(), window_cb(), window_rect()

### Community 25 - "Encode Presets"
Cohesion: 0.12
Nodes (18): EncodeParams, Preset, .ALL, Native60, NativeHigh, P1080p30, P1080p60, P1440p60 (+10 more)

### Community 26 - "Monitor Enumeration"
Cohesion: 0.40
Nodes (7): device_name(), enumerate(), monitor_cb(), monitor_info(), monitors(), primary_monitor(), refresh_rate()

### Community 27 - "Share Room Management"
Cohesion: 0.10
Nodes (9): Abr, ABR_FLOOR, HANDSHAKE_TIMEOUT, ids(), invite_addr_keeps_only_relay(), invite_addr_without_relay_keeps_ips(), MAX_HANDSHAKES, retire() (+1 more)

### Community 28 - "CI Workflow"
Cohesion: 0.10
Nodes (23): runs-on, steps, runs-on, steps, runs-on, steps, jobs, android (+15 more)

### Community 29 - "README Documentation"
Cohesion: 0.15
Nodes (12): Android (só para assistir), Avançado, Como funciona, Como usar, Conexões que o app faz, Configurações, Download e atualização, Licença (+4 more)

### Community 30 - "Codec Selection"
Cohesion: 0.14
Nodes (17): Codec, Av1, H264, Hevc, pick_encoder(), supported_codecs(), EncoderKind, Hardware (+9 more)

### Community 31 - "Settings Modal"
Cohesion: 0.17
Nodes (22): button(), diagnostics(), FORCED, hint(), icon_button(), open_logs(), SETTINGS_LABEL, share() (+14 more)

### Community 32 - "Capture Frame Sinks"
Cohesion: 0.15
Nodes (5): CountSink, FirstFrameSink, FrameSink, selftest(), SinkHandle

### Community 33 - "Changelog Test Script"
Cohesion: 0.70
Nodes (4): commit(), has(), lacks(), test-changelog.sh script

### Community 34 - "Android JNI Bridge"
Cohesion: 0.08
Nodes (32): AbortOnDrop, __android_log_write(), audio_loop(), get_session(), init_logging(), INIT_LOGGING, Java_io_github_pendiego_livescreen_Native_decoderFailed(), Java_io_github_pendiego_livescreen_Native_findInvite() (+24 more)

### Community 35 - "Native JNI Declarations"
Cohesion: 0.12
Nodes (3): Amigos e presença, Regras, Native

### Community 36 - "Friend Presence Protocol"
Cohesion: 0.06
Nodes (32): AbortJoinSet, denied_reason(), dial_endpoint(), friend_code(), friend_code_roundtrip_and_normalization(), new_secret_key(), parse_friend_code(), parse_secret_key() (+24 more)

### Community 37 - "Project Documentation Guide"
Cohesion: 0.06
Nodes (20): Android (`android/`), CI, Comandos, Desktop (`src/`), Estilo, graphify, LiveScreen, Mapa do código (+12 more)

### Community 39 - "DXGI Desktop Duplication"
Cohesion: 0.24
Nodes (6): Composer, create_duplication(), GrabSink, run(), start(), texture()

### Community 40 - "Main Entry Allocator"
Cohesion: 0.50
Nodes (3): GLOBAL, init_logging(), main()

### Community 41 - "GPU Fence Synchronization"
Cohesion: 0.06
Nodes (9): Mmcss, Display, GpuFrame, Imported, NEXT_SURFACES_ID, Producer, SLOTS, Surfaces (+1 more)

### Community 44 - "Workspace Crates"
Cohesion: 0.67
Nodes (3): livescreen, livescreen-android, livescreen-core

### Community 48 - "Message Stream IO"
Cohesion: 0.40
Nodes (3): encode_msg(), read_msg(), write_msg()

### Community 49 - "Invite Target Parsing"
Cohesion: 0.29
Nodes (4): Target, Friend, Invite, verify_room_proof()

### Community 50 - "App Screen States"
Cohesion: 0.18
Nodes (11): EndKind, Error, Info, Warn, Screen, Ended, Home, SharePicker (+3 more)

### Community 51 - "Video Encoder"
Cohesion: 0.17
Nodes (7): configure(), create_mft(), EncoderStats, read_seq_header(), spawn(), stream_ids(), vbv_bytes()

### Community 52 - "Main Activity Theme"
Cohesion: 0.09
Nodes (4): MainActivity, colorScheme(), LiveScreenTheme(), StatusColors

### Community 53 - "Quality Presets"
Cohesion: 0.12
Nodes (26): bitrate(), encode_params(), extreme_shapes_never_break_limits(), fit_level(), label(), light_presets_are_reductions_and_fall_back(), lighter_is_accepted_equal_and_heavier_are_not(), LIMIT_FPS (+18 more)

### Community 54 - "Thumbnail Bitmap Capture"
Cohesion: 0.29
Nodes (5): Dib, fit(), thumbnail(), thumbnail_monitor(), thumbnail_window()

### Community 55 - "Invite Format Proof"
Cohesion: 0.23
Nodes (10): find_invite(), find_invite_in_pasted_text(), format_invite(), invite_roundtrip_and_proof_binding(), new_room_key(), parse_invite(), room_proof(), is_valid_invite() (+2 more)

### Community 56 - "Viewer Session Messages"
Cohesion: 0.05
Nodes (21): ViewerMsg, Hello, RequestKeyframe, SetQuality, SetVisible, BUDGET_MSG, CLOSE_DENIED_U64, CLOSE_FULL_U64 (+13 more)

### Community 57 - "Capture Backend Selection"
Cohesion: 0.30
Nodes (8): Backend, Dxgi, GraphicsCapture, CaptureHandle, Dxgi, Wgc, run_backend(), start()

### Community 58 - "Web STUN Signaling"
Cohesion: 0.11
Nodes (10): ROOM_KEY_LEN, stun_binding_request(), STUN_MAGIC_COOKIE, stun_mapped_addr(), test_stun_binding_request(), test_stun_rfc5769_vector(), test_viewer_url_no_duplicate_slash(), test_web_proof_verification() (+2 more)

### Community 59 - "Async Encoder Events"
Cohesion: 0.18
Nodes (7): CpuCopy, Encoder, Feed, Frame, Idle, Stop, Nv12Frame

### Community 60 - "Viewer Admission Handling"
Cohesion: 0.13
Nodes (10): sanitize(), admit(), answer_presence(), CountGuard, handle_viewer(), Peer, PeerConn, Iroh (+2 more)

### Community 63 - "Audio Capture Mixing"
Cohesion: 0.12
Nodes (11): FRAME_BYTES, FRAME_FLOATS, FRAME_SAMPLES, MAX_LAG_BYTES, mix(), mix_clamps(), OPUS_MAX_PACKET, take_frame() (+3 more)

### Community 65 - "Compose Home Screen"
Cohesion: 0.19
Nodes (14): AddFriendDialog(), App(), CallButton(), CallToggle(), CallTone, Active, Alert, Neutral (+6 more)

### Community 66 - "Share Session Control"
Cohesion: 0.19
Nodes (3): is_reduction(), ShareSession, unpack_size()

### Community 67 - "Cloudflare Worker Rooms"
Cohesion: 0.23
Nodes (4): bytesToHex(), hexToBytes(), iceServers(), Room

### Community 68 - "Encoder Handle Resources"
Cohesion: 0.20
Nodes (4): Reconf, Resources, start_encoder(), EncoderHandle

### Community 69 - "Frame Budget Limits"
Cohesion: 0.24
Nodes (5): budget_released_on_drop(), Budgeted, check_len(), FrameBudget, read_video_frame()

### Community 71 - "Wrangler Package Config"
Cohesion: 0.18
Nodes (10): wrangler, devDependencies, wrangler, name, private, scripts, check, deploy (+2 more)

### Community 72 - "Capture Source Info"
Cohesion: 0.50
Nodes (5): SourceId, Monitor, Window, SourceInfo, Update

### Community 73 - "Web Viewer Script"
Cohesion: 0.40
Nodes (10): addOpusStereo(), bytesToHex(), cleanup(), computeProof(), connect(), hexToBytes(), sdpFingerprint(), showStatus() (+2 more)

### Community 74 - "Host Message Types"
Cohesion: 0.40
Nodes (5): HostMsg, Rejected, StreamEnded, StreamParams, Welcome

### Community 75 - "Microphone Capture"
Cohesion: 0.22
Nodes (7): Mic, Default, Device, open(), open_mic(), pump(), Source

### Community 77 - "Toast Notifications"
Cohesion: 0.29
Nodes (5): ToastKind, Info, Success, Warn, Toasts

### Community 78 - "Share View UI"
Cohesion: 0.08
Nodes (17): AUTO_QUALITY_COOLDOWN, begin_start(), Picker, poll_starting(), Preview, spawn_share_job(), Starting, to_image() (+9 more)

### Community 83 - "UI Icons"
Cohesion: 0.08
Nodes (19): I_EXIT_FULLSCREEN, I_FULLSCREEN, I_HANGUP, I_MUTE, I_PASTE, I_PERSON, I_QUALITY, I_SETTINGS (+11 more)

### Community 84 - "Audio Capture Loop"
Cohesion: 0.50
Nodes (6): AudioRequest, capture_loop(), mic_error_text(), Opened, run(), start()

### Community 85 - "RTC Peer Events"
Cohesion: 0.25
Nodes (3): rtt_congested(), drain_outputs(), peer()

### Community 86 - "Host WebSocket URL"
Cohesion: 0.33
Nodes (6): host_ws_url(), test_host_ws_url(), base_url(), init_crypto(), run(), super::ShareSession

### Community 87 - "Signal Outgoing Messages"
Cohesion: 0.33
Nodes (6): SignalOut, Answer, Close, Host, Ping, Reject

### Community 90 - "Host Endpoint Info"
Cohesion: 0.20
Nodes (3): invite_addr(), Overload, ViewerInfo

### Community 93 - "Signal Incoming Messages"
Cohesion: 0.40
Nodes (5): SignalIn, Left, Offer, Pong, Ready

### Community 95 - "Web Connection State"
Cohesion: 0.40
Nodes (5): WebState, Connecting, Off, Online, Retrying

### Community 97 - "Audio Loopback Target"
Cohesion: 0.50
Nodes (4): AudioTarget, Process, System, open_loopback()

### Community 98 - "Border Removal Check"
Cohesion: 0.67
Nodes (3): border_removable(), property_present(), start()

### Community 99 - "Audio State"
Cohesion: 0.50
Nodes (4): AudioState, Off, On, Unavailable

## Knowledge Gaps
- **105 isolated node(s):** `Readback`, `Gpu`, `NeedMoreInput`, `StreamChange`, `CloseMidBody` (+100 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 754 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **25 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `Desktop App Logic` to `Config and Identity`, `Share Session Control`, `Share Status UI`, `Friend Presence Protocol`, `Capture Source Info`, `Toast Notifications`, `Share View UI`, `Build Script Icons`, `Invite Parsing Layout`, `App Screen States`, `UI Icons`, `Window Monitor Enumeration`, `Invite Format Proof`, `Viewer Session Messages`, `Codec Selection`, `Settings Modal`?**
  _High betweenness centrality (0.101) - this node is a cross-community bridge._
- **What connects `Readback`, `Gpu`, `NeedMoreInput` to the rest of the system?**
  _105 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Video Decoder` be split into smaller, more focused modules?**
  _Cohesion score 0.1337126600284495 - nodes in this community are weakly interconnected._
- **Why does `SourceId` connect `Capture Source Info` to `Capture Frame Sinks`, `Border Removal Check`, `Share Status UI`, `WGC Interop Setup`, `DXGI Desktop Duplication`, `Window Capture GDI`, `GDI Cursor Capture`, `E2E Window Tests`, `Desktop App Logic`, `Share View UI`, `Thumbnail Bitmap Capture`, `Window Monitor Enumeration`, `Window Inspection`, `Capture Backend Selection`, `Monitor Enumeration`, `Share Room Management`?**
  _High betweenness centrality (0.049) - this node is a cross-community bridge._
- **Should `Config and Identity` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `Preset` connect `Encode Presets` to `Config and Identity`, `Share Session Control`, `Share Status UI`, `Encoder Handle Resources`, `Share View UI`, `Quality Presets`, `Viewer Session Messages`, `Host Endpoint Info`, `Share Room Management`, `Viewer Admission Handling`, `Settings Modal`?**
  _High betweenness centrality (0.042) - this node is a cross-community bridge._
- **Should `Audio Playback` be split into smaller, more focused modules?**
  _Cohesion score 0.1225071225071225 - nodes in this community are weakly interconnected._