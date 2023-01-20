use bevy::prelude::Component;
use strolle as st;

/// Extends Bevy's camera with extra features supported by Strolle.
///
/// This is a component that can be attached into Bevy's `Camera`; when not
/// attached, the default configuration is used.
#[derive(Clone, Debug, Default, Component)]
pub struct StrolleCamera {
    pub config: st::ViewportConfiguration,
}
