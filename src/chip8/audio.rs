use rodio::{DeviceSinkError, MixerDeviceSink, Player};

pub enum BeepingState {
    Stopped,
    Beeping,
}

pub struct Audio {
    pub handle: MixerDeviceSink,
    pub player: Player,
    pub beeping_state: BeepingState,
}

impl Audio {
    pub fn build() -> Result<Self, DeviceSinkError> {
        let handle = rodio::DeviceSinkBuilder::open_default_sink()?;
        let player = rodio::Player::connect_new(handle.mixer());

        Ok(Audio {
            handle,
            player,
            beeping_state: BeepingState::Stopped,
        })
    }
}
