## Context
GPUI + gpui-component are already in use. We will map the provided Tailwind-esque palette into `gpui_component::theme` tokens and implement a dashboard layout consistent with the shared screenshot.

## Goals / Non-Goals
- Goals: Modernize dashboard, consistent dark palette, intuitive navigation, preserve performance
- Non-Goals: Rewrite detection/audio systems; change app architecture

## Decisions
- Theme palette:
  - primary: `#EA2831`
  - surfaces: sidebar `#121212`, cards `#1E1E1E`, background `#211111`
  - success (status chip): `#39FF14`
  - border: subtle `#ffffff1a` equivalent using theme border token
- Layout: Sidebar (left, fixed width) + main content; cards with rounded corners and borders
- Icons: Use existing emoji/text icons to avoid new asset dependency

## Risks / Trade-offs
- Color accuracy vs. system theme tokens: we will approximate within GPUI theme limits

## Migration Plan
- Update tokens → build new dashboard sections → wire actions → polish → tests

## Open Questions
- Do we prefer exact hex matching vs. slight adjustments for contrast/AA?
