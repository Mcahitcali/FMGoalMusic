pub mod effects;
pub mod manager;
pub mod player;
/// Audio system module
///
/// Provides a comprehensive audio system supporting:
/// - Multiple simultaneous audio sources (music, ambiance, crowd sounds)
/// - Effect chains (fade in/out, volume control, time limiting)
/// - Event-driven playback
///
/// ## Architecture
///
/// ```text
/// AudioSystemManager
///   ├── AudioPlayer (GoalMusic)     ─┐
///   ├── AudioPlayer (GoalAmbiance)  ─┤ Simultaneous
///   ├── AudioPlayer (MatchStart)    ─┤ Playback
///   └── AudioPlayer (MatchEnd)      ─┘
///
/// Each AudioPlayer has:
///   └── EffectChain
///       ├── FadeEffect (in/out)
///       ├── VolumeEffect
///       └── LimiterEffect (time limit)
/// ```
///
/// ## Usage
///
/// ```rust,ignore
/// use audio_system::{AudioSystemManager, AudioSourceType, EffectChain};
///
/// let manager = AudioSystemManager::new();
///
/// // Load audio with effects
/// let effects = EffectChain::default()
///     .with_fade_in(200)
///     .with_fade_out(2000)
///     .with_volume(0.8)
///     .with_limit(20_000);
///
/// manager.load_audio(
///     AudioSourceType::GoalMusic,
///     Path::new("goal.mp3"),
///     effects
/// )?;
///
/// // Play audio
/// manager.play(AudioSourceType::GoalMusic)?;
///
/// // Multiple sources can play simultaneously
/// manager.play(AudioSourceType::GoalAmbiance)?;
/// ```
pub mod source;

// Re-export commonly used types
pub use effects::{EffectChain, FadeEffect, LimiterEffect, VolumeEffect};
pub use manager::AudioSystemManager;
pub use player::AudioPlayer;
pub use source::AudioSourceType;
