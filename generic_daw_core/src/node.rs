use crate::track::Track;
use crate::channel::Channel;
use audio_graph::{NodeId, NodeImpl, Injector};
use std::convert::Infallible;

#[derive(Debug)]
pub enum Node {
    Track(Track),
    Channel(Channel),
}

impl From<Channel> for Node {
    fn from(c: Channel) -> Self { Self::Channel(c) }
}

impl NodeImpl for Node {
    type Event = crate::Event;
    type State = crate::audio_thread::State;
    type Inject<'a> = Infallible;

    fn id(&self) -> NodeId {
        match self {
            Self::Track(t) => t.id(),
            Self::Channel(c) => c.id(),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Track(t) => t.reset(),
            Self::Channel(c) => c.reset(),
        }
    }

    fn process(&mut self, state: &Self::State, output: &mut [[f32; 2]], events: &mut Vec<Self::Event>, injector: &Injector<Self::Inject<'_>>) -> usize {
        match self {
            Self::Track(t) => t.process(state, output, events, injector),
            Self::Channel(c) => c.process(state, output, events, injector),
        }
    }
}

impl Node {
    pub fn restart_all_plugins(&mut self) {
        match self {
            Self::Track(t) => t.restart_all_plugins(),
            Self::Channel(c) => c.restart_all_plugins(),
        }
    }
    pub fn clear_updates(&mut self) {
        match self {
            Self::Track(t) => t.clear_updates(),
            Self::Channel(c) => c.clear_updates(),
        }
    }
    pub fn collect_updates(&mut self, updates: &mut Vec<crate::Event>) {
        match self {
            Self::Track(t) => t.collect_updates(updates),
            Self::Channel(c) => c.collect_updates(updates),
        }
    }
    pub fn apply(&mut self, action: crate::midi_pattern::MidiPatternAction, state: &crate::audio_thread::State) {
        match self {
            Self::Track(t) => t.apply(action, state),
            Self::Channel(c) => c.apply(action, state),
        }
    }
}
