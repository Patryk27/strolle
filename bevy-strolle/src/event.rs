use bevy::prelude::*;

#[derive(Debug)]
pub enum StrolleEvent {
    MarkImageAsDynamic { handle: Handle<Image> },
}
