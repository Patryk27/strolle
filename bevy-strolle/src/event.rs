use bevy::prelude::*;

#[derive(Debug, Event)]
pub enum StrolleEvent {
    MarkImageAsDynamic { id: AssetId<Image> },
}
