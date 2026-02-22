use rodio::{DeviceSinkError, MixerDeviceSink, Player, source::SineWave};

pub enum BeepingState {
    Stopped,
    Beeping,
}

pub struct Audio {
    player: Player,
    _handle: MixerDeviceSink,
    pub beeping_state: BeepingState,
}

impl Audio {
    pub fn build() -> Result<Self, DeviceSinkError> {
        let handle = rodio::DeviceSinkBuilder::open_default_sink()?;
        let player = rodio::Player::connect_new(handle.mixer());

        let source = SineWave::new(440.0);
        player.append(source);
        player.pause();

        Ok(Audio {
            player,
            _handle: handle,
            beeping_state: BeepingState::Stopped,
        })
    }

    pub fn start_beep(&self) {
        self.player.play();
    }

    pub fn stop_beep(&self) {
        self.player.pause();
    }
}
