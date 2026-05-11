// =============================================================================
// FILE: crates/gargantua-video/tests/codec_support.rs
// CRATE: gargantua-video
// TYPE: Integration test
// LINES: ~100
// PLATFORM: Mac + Windows (runtime hardware check)
// =============================================================================
//
// PURPOSE:
//   Verifies that the hardware codec availability detection functions work
//   correctly at runtime. Checks that at least one encoder is always available
//   for each codec type (H.264, H.265, AV1), and validates that the
//   best_encoder() dispatcher returns a working encoder without panicking.
//
// WHAT THIS FILE CONTAINS:
//
//   --- IMPORTS ---
//   use gargantua_video::encode::mod::best_encoder;
//   use gargantua_video::config::{OfflineConfig, VideoCodec, BitDepth};
//   #[cfg(target_os = "macos")]
//   use gargantua_video::encode::mac::{
//       videotoolbox_h264::VideoToolboxH264,
//       videotoolbox_h265::VideoToolboxH265,
//       videotoolbox_av1::VideoToolboxAv1,
//   };
//   #[cfg(target_os = "windows")]
//   use gargantua_video::encode::windows::{
//       nvenc_h264::NvencH264,
//       nvenc_h265::NvencH265,
//       amf_h264::AmfH264,
//       qsv_h264::QsvH264,
//   };
//   use gargantua_video::encode::universal::{
//       x264::X264Encoder,
//       x265::X265Encoder,
//       rav1e::Rav1eEncoder,
//   };
//
//   --- TESTS ---
//
//   #[test]
//   fn test_h264_encoder_always_available()
//         Constructs a minimal OfflineConfig with VideoCodec::H264, 1920×1080,
//         calls best_encoder(&config), asserts the returned encoder is not an
//         error. Verifies that even on a headless CI machine (no GPU), the
//         software x264 fallback is selected and does not panic.
//         Calls encoder.codec_name() and asserts it contains "H.264" or "H264".
//
//   #[test]
//   fn test_h265_encoder_available_or_fallback()
//         Same as above but for VideoCodec::H265.
//         On Mac with VideoToolbox: expects "VideoToolbox H.265".
//         On Windows NVIDIA: expects "NVENC H.265".
//         Fallback: expects "x265".
//         Does NOT assert a specific name — only asserts Ok(encoder) returned.
//
//   #[cfg(target_os = "macos")]
//   #[test]
//   fn test_videotoolbox_h264_is_available_on_apple_silicon()
//         Calls VideoToolboxH264::new() with a default config.
//         Asserts Ok(_) — VideoToolbox H.264 must always succeed on Apple Silicon.
//         This test is excluded from Windows CI via cfg gate.
//
//   #[cfg(target_os = "macos")]
//   #[test]
//   fn test_videotoolbox_h265_10bit_available()
//         Constructs OfflineConfig with bit_depth = BitDepth::Ten.
//         Calls VideoToolboxH265::new() and asserts Ok(_).
//         Verifies 10-bit HEVC encoding is supported on M1+.
//
//   #[cfg(target_os = "macos")]
//   #[test]
//   fn test_videotoolbox_av1_reports_correct_availability()
//         Calls VideoToolboxAv1::is_hardware_available().
//         On M3/M4/M5 hardware: should return true.
//         On M1/M2 or Intel: should return false.
//         Either result is acceptable — the test just verifies the function
//         returns without panicking and matches the rav1e fallback creation:
//           if !is_hardware_available() { assert!(VideoToolboxAv1::new().rav1e_fallback.is_some()) }
//
//   #[test]
//   fn test_software_x264_always_available()
//         Creates X264Encoder with a minimal config (640×360, 1 Mbps).
//         Encodes a single blank YUV I420 frame (all zeros, 640×360).
//         Asserts encode_frame() returns Ok(bytes) where bytes.len() > 0.
//         Calls flush() and asserts Ok(packets).
//         This test runs on all platforms including CI without GPU.
//
//   #[test]
//   fn test_rav1e_encoder_produces_valid_av1_bitstream()
//         Creates Rav1eEncoder with 320×240, 8-bit, speed = 10 (fastest).
//         Sends one blank frame and receives one packet.
//         Asserts the returned bytes start with the AV1 temporal unit header
//         (0x0A 0x0E 0x00 0x00 — OBU_TEMPORAL_DELIMITER magic bytes).
//         This validates the rav1e integration produces a well-formed bitstream.
//
//   #[test]
//   fn test_best_encoder_exr_seq_always_succeeds()
//         Constructs OfflineConfig with VideoCodec::ExrSequence.
//         Creates a temp directory via tempfile::tempdir().
//         Calls best_encoder(&config) — should always return ExrSeqEncoder.
//         Asserts encoder.codec_name() == "EXR Sequence".
//
// OUTBOUND DEPENDENCIES (imports used in tests):
//   - gargantua_video::encode::mod    → best_encoder()
//   - gargantua_video::config         → OfflineConfig, VideoCodec, BitDepth
//   - gargantua_video::encode::mac::* → VideoToolbox encoders (Mac only)
//   - gargantua_video::encode::universal::* → x264, x265, rav1e, exr_seq
//   - tempfile (dev-dependency)       → tempdir() for EXR sequence output path
//
// INBOUND (who runs these tests):
//   - cargo test -p gargantua-video        → runs all tests in this file
//   - .github/workflows/ci.yml             → runs on Mac runner and Windows runner
//
// NOTES:
//   - Hardware encoder tests (VideoToolbox, NVENC, AMF) are gated with
//     #[cfg(target_os = "macos")] or #[cfg(target_os = "windows")] so they
//     only run on the appropriate CI runner.
//   - Tests that require physical hardware encoder (NVENC, AMF) are additionally
//     gated with #[ignore] in CI and marked to run only on self-hosted runners
//     with real GPU hardware. Software encoder tests (x264, rav1e, EXR) run
//     on all platforms including headless CI agents.
//   - This test file does NOT test encode quality or bitrate accuracy —
//     only that encoders initialise and produce non-empty output bytes.
//     Quality validation is done manually via visual inspection + VMAF scores.
// =============================================================================