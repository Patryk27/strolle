use crate::{Engine, Event, Params};

pub trait EventHandler<P>
where
    P: Params,
{
    fn handle(&mut self, ctxt: EventHandlerContext<P>);
}

#[derive(Copy, Clone)]
pub struct EventHandlerContext<'a, P>
where
    P: Params,
{
    pub(crate) engine: &'a Engine<P>,
    pub(crate) device: &'a wgpu::Device,
    pub(crate) event: &'a Event<P>,
}
