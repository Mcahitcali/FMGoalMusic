## ADDED Requirements
### Requirement: GPUI Runtime Initialization
The system SHALL initialize the GPUI runtime and gpui-component before rendering UI.

#### Scenario: App bootstrap
- **WHEN** the application starts
- **THEN** it SHALL execute gpui_component::init(cx) inside the app.run closure and mount the root view

### Requirement: GPUI Component Rendering
The system SHALL render UI using GPUI’s component model and gpui-component stateless RenderOnce elements.

#### Scenario: Render a button
- **WHEN** a view renders a primary button
- **THEN** it SHALL use gpui_component::button::Button and handle its click via GPUI event handlers

### Requirement: Theming via ActiveTheme
The system MUST expose theme tokens via ActiveTheme and components MUST read from cx.theme().

#### Scenario: Access theme tokens
- **WHEN** a component needs a color
- **THEN** it SHALL read from cx.theme().primary (and related tokens)

## MODIFIED Requirements
### Requirement: GUI Event Handling and State
The system SHALL use GPUI’s event model instead of egui’s.

#### Scenario: Button click dispatch
- **WHEN** a user clicks a button
- **THEN** the system SHALL dispatch a GPUI event handler and update view state

## REMOVED Requirements
### Requirement: egui Runtime and Widgets
**Reason**: Replaced by GPUI runtime and gpui-component.
**Migration**: Widgets re-implemented with GPUI components; events/themes migrated to GPUI equivalents.
