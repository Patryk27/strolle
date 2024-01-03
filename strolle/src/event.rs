use bevy::prelude::*;

#[derive(Debug, Event)]
pub enum Event {
    MarkImageAsDynamic { id: AssetId<Image> },
}
