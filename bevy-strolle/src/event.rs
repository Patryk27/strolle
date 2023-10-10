use bevy::prelude::*;

#[derive(Debug, Event)]
pub enum StrolleEvent {
    MarkImageAsDynamic { handle: Handle<Image> },
}
