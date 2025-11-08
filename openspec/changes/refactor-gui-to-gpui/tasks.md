## 1. Bootstrap & Dependencies
- [ ] 1.1 Add gpui and gpui-component to Cargo.toml; remove egui
- [ ] 1.2 Create GPUI app entrypoint; call gpui_component::init(cx) in app.run
- [ ] 1.3 Update build scripts/CI for GPUI

## 2. Theming
- [ ] 2.1 Define ActiveTheme mapping (primary/background/foreground)
- [ ] 2.2 Replace hardcoded colors with cx.theme()
- [ ] 2.3 Document theme selection/switching

## 3. Component Migration
- [ ] 3.1 Map egui widgets → gpui-component equivalents
- [ ] 3.2 Port src/gui/views/** to GPUI components
- [ ] 3.3 Replace event handlers with GPUI model
- [ ] 3.4 Validate layout/render parity

## 4. Messaging & State
- [ ] 4.1 Integrate src/gui/messages.rs with GPUI handlers
- [ ] 4.2 Bridge app state with GPUI view state
- [ ] 4.3 Remove egui-specific adapters

## 5. Feature Parity & Testing
- [ ] 5.1 Update GUI tests under tests/**
- [ ] 5.2 Add smoke tests for startup/navigation

## 6. Cleanup & Docs
- [ ] 6.1 Remove egui code/assets
- [ ] 6.2 Update README/dev docs
- [ ] 6.3 Add rollback notes/limitations

## 7. Validation
- [ ] 7.1 Run openspec validate refactor-gui-to-gpui --strict
