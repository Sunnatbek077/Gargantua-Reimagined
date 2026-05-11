// ============================================================
// FILE: crates/gargantua-ui/src/theme.rs
// LINES: ~260
// CATEGORY: UI — Visual theme (colors, fonts, spacing)
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Defines the complete visual theme for the Gargantua UI:
//   color palette, font sizes, spacing constants, and egui Style
//   configuration. Applied once at startup via Hud::new().
//   Dark space-themed aesthetic inspired by the Interstellar film.
//
// CONTENTS (~260 lines):
//   pub struct Theme {
//       // Color palette (linear sRGB stored as egui::Color32)
//       pub bg_primary:    egui::Color32,  // #0D1117 — main background
//       pub bg_secondary:  egui::Color32,  // #161B22 — panel background
//       pub bg_tertiary:   egui::Color32,  // #21262D — widget background
//       pub accent:        egui::Color32,  // #4D9DE0 — blue accent (sliders, tabs)
//       pub accent_green:  egui::Color32,  // #43D19E — positive values, OK
//       pub accent_orange: egui::Color32,  // #E8A84D — warnings, caution
//       pub accent_red:    egui::Color32,  // #E85D6F — errors, danger
//       pub text_primary:  egui::Color32,  // #CDD6E8 — main text
//       pub text_secondary:egui::Color32,  // #8A96A8 — secondary/dimmed text
//       pub text_dim:      egui::Color32,  // #4A5568 — very dim labels
//       pub separator:     egui::Color32,  // #2D3748 — divider lines
//
//       // Font sizes (in logical pixels)
//       pub font_label:    f32,   // 13.0 — widget labels
//       pub font_body:     f32,   // 14.0 — main body text
//       pub font_heading:  f32,   // 16.0 — section headings
//       pub font_mono:     f32,   // 13.0 — monospace (values, code)
//
//       // Spacing
//       pub item_spacing:  egui::Vec2,  // (8, 4) — between widgets
//       pub section_gap:   f32,         // 12.0 — between sections
//       pub panel_padding: egui::Vec2,  // (12, 8) — inside panels
//   }
//
//   impl Theme {
//       pub fn dark() -> Self  // The default Gargantua dark theme
//
//       // Apply this theme to egui Context (call once at startup)
//       pub fn apply(&self, ctx: &egui::Context)
//         // Sets ctx.set_style(self.to_egui_style())
//         // Loads and installs custom fonts (see install_fonts)
//
//       // Convert to egui::Style
//       fn to_egui_style(&self) -> egui::Style
//         // Maps color fields to egui visuals (widget, window, etc.)
//         // Sets panel_fill, window_fill, text colors, button styles
//
//       // Install fonts into egui FontDefinitions
//       fn install_fonts(ctx: &egui::Context)
//         // Inter (variable): body/labels — embedded via include_bytes!
//         // JetBrains Mono: values/code — embedded via include_bytes!
//         // Sets proportional=Inter, monospace=JetBrains Mono
//   }
//
// USES (imports from):
//   egui  → Context, Style, Color32, Vec2, FontDefinitions, FontData
//
// USED BY:
//   crates/gargantua-ui/src/hud/mod.rs
//     → Theme::dark().apply(ctx) called in Hud::new()
//   crates/gargantua-ui/src/lib.rs
//     → pub use theme::Theme
//
// NOTE FOR AI:
//   All Color32 values above are in sRGB (0–255 per channel).
//   egui renders in sRGB space — NO linear conversion needed for UI colors.
//   Font files embedded via include_bytes!("../../assets/fonts/Inter.ttf")
//   Paths are relative to this file's location in the source tree.
//   to_egui_style() must set visuals.dark_mode = true.
//   theme.rs is the ONLY place where egui Style is configured —
//   no other file should call ctx.set_style() or ctx.set_visuals().
// ============================================================