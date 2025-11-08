# Change: Replace egui with zed/gpui and adopt gpui-component

## Why
egui limits our desired component patterns and GPU-accelerated hybrid architecture. zed/gpui + gpui-component better match our UX and theming needs.

## What Changes
- Replace egui with zed/gpui runtime
- Integrate gpui-component (stateless RenderOnce components)
- Initialize gpui-component via gpui_component::init(cx) at app bootstrap
- Implement theming via ActiveTheme and project token mapping
- Migrate views, messages, event handling to GPUI model
- Remove egui dependencies and code paths
- Update build/CI for GPUI
- Document migration and rollback
- **BREAKING**: GUI APIs, event model, and theme tokens change

## Impact
- Affected specs: gui
- Affected code: src/gui/**, Cargo.toml, build.rs/build scripts, GUI tests
