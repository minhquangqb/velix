// Shared Vue components will move here once a second package consumes them.
//
// Phase 3's shell components (TitleBar, PlatformRail, AccountList, GlyphBadge,
// ToggleSwitch) live in apps/desktop/src/components for now: this package is
// built with plain `tsc` and has no SFC pipeline, and the desktop app is still
// their only consumer. Adding a build here to serve one caller would be cost
// without a payer.
export {}
