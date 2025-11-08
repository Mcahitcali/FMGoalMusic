## Context
Replace egui with zed/gpui and gpui-component to improve performance, theming, and developer ergonomics.

## Goals / Non-Goals
- Goals: GPUI runtime, ActiveTheme theming, component migration, parity
- Non-Goals: Broad UX redesign

## Decisions
- Initialize gpui-component at bootstrap
- Prefer stateless RenderOnce components
- Centralize theme tokens via ActiveTheme
- Use gpui-component library before custom widgets

## Alternatives
- Keep egui and extend (rejected)
- Other Rust GUI frameworks (rejected for maturity/integration)

## Risks / Trade-offs
- GPUI pre-1.0, potential breaking changes → pin versions, CI checks
- Wide migration surface → staged rollout, shims

## Migration Plan
1) Introduce GPUI behind a feature flag
2) Port base layout + theme
3) Port remaining views, remove egui
4) Enable GPUI by default; remove flag

## Open Questions
- Final theme palette & dark/light variants?
- Any bespoke widgets needed?
