use audio_graph::NodeId;
use crate::audio_thread::State;
use crate::Event; // Import directo

#[derive(Debug, Default)]
pub struct Channel {
    pub id: NodeId,
}

impl Channel {
    pub fn id(&self) -> NodeId { self.id }
    pub fn reset(&mut self) {}
    pub fn apply(&mut self, _action: crate::midi_pattern::MidiPatternAction, _state: &State) {}
    pub fn collect_updates(&mut self, _updates: &mut Vec<Event>) {}
    pub fn clear_updates(&mut self) {}
    pub fn restart_all_plugins(&mut self) {}
    pub fn process(&mut self, _state: &State, _output: &mut [[f32; 2]], _events: &mut Vec<Event>, _injector: &audio_graph::Injector<std::convert::Infallible>) -> usize { 0 }
}
