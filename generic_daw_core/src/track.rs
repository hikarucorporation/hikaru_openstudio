use crate::channel::Channel;
use audio_graph::NodeId;

#[derive(Debug)]
pub struct Track {
    pub channel: Channel,
}

impl Track {
    pub fn id(&self) -> NodeId {
        self.channel.id()
    }

    pub fn reset(&mut self) {
        self.channel.reset();
    }

    pub fn apply(&mut self, action: crate::midi_pattern::MidiPatternAction, state: &crate::audio_thread::State) {
        self.channel.apply(action, state);
    }

    pub fn collect_updates(&mut self, updates: &mut Vec<crate::audio_thread::Event>) {
        self.channel.collect_updates(updates);
    }

    pub fn clear_updates(&mut self) {
        self.channel.clear_updates();
    }

    pub fn restart_all_plugins(&mut self) {
        self.channel.restart_all_plugins();
    }
}
