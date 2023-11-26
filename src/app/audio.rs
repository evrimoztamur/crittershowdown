use std::collections::HashMap;

use js_sys::{ArrayBuffer, Math, Uint8Array};
use wasm_bindgen::JsCast;
use web_sys::{console, AudioBuffer, AudioContext, GainNode};

use super::SettingsMenuState;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ClipId {
    CrackleI,
    CrackleII,
    CrackleIII,
    ZapI,
    ZapII,
    ZapIII,
    Beam,
    Diagonal,
    Shield,
    LevelEnter,
    LevelSuccess,
    LevelFailure,
    MageSelect,
    MageDeselect,
    MageMove,
    ClickForward,
    ClickBack,
    ButtonHover,
    MapPlaceObject,
    MapSelectSquare,
    MapIncreaseSize,
    MapDecreaseSize,
    StarSparkle,
    MusicI,
}

#[derive(Clone, Debug)]
pub struct AudioClip {
    buffer: AudioBuffer,
    volume: f32,
}

#[derive(Clone, Debug)]
pub struct AudioSystem {
    context: AudioContext,
    audio_clips: HashMap<ClipId, AudioClip>,
    music_gain: Option<GainNode>,
    base_volume: f32,
    music_volume: i8,
    clip_volume: i8,
}

impl AudioSystem {
    pub async fn register_audio_clip(&mut self, clip_id: ClipId, data: &[u8], volume: f32) {
        let promise = self
            .context
            .decode_audio_data(&u8_slice_to_array_buffer(data))
            .ok();

        if let Some(promise) = promise {
            let buffer = wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .unwrap()
                .dyn_into::<AudioBuffer>()
                .unwrap();

            let audio_clip = AudioClip { buffer, volume };

            console::log_1(&format!("{:?}", audio_clip).into());

            self.audio_clips.insert(clip_id, audio_clip);
        }
    }

    pub fn set_music_volume(&mut self, volume: i8) {
        self.music_volume = volume;

        if let Some(gain_node) = &self.music_gain {
            gain_node.gain().set_value(self.music_volume());
        }
    }

    pub fn music_volume(&self) -> f32 {
        self.music_volume as f32 / 10.0
    }

    pub fn set_clip_volume(&mut self, volume: i8) {
        self.clip_volume = volume;
    }

    pub fn clip_volume(&self) -> f32 {
        self.clip_volume as f32 / 10.0
    }

    pub fn play_clip(&self, clip_id: ClipId) {
        if let Some(audio_clip) = self.audio_clips.get(&clip_id) {
            let real_volume = audio_clip.volume * self.base_volume * self.clip_volume();

            let buffer_source = self.context.create_buffer_source().unwrap();
            buffer_source.set_buffer(Some(&audio_clip.buffer));

            let gain_node = self.context.create_gain().unwrap();
            gain_node.gain().set_value(real_volume);

            buffer_source.connect_with_audio_node(&gain_node).unwrap();
            gain_node
                .connect_with_audio_node(&self.context.destination())
                .unwrap();

            buffer_source.start_with_when(0.0).unwrap();
        }
    }

    pub fn play_clip_option(&self, clip_id: Option<ClipId>) {
        if let Some(clip_id) = clip_id {
            self.play_clip(clip_id);
        }
    }

    pub fn play_music(&mut self, clip_id: ClipId) {
        if let Some(audio_clip) = self.audio_clips.get(&clip_id) {
            let real_volume = audio_clip.volume * self.base_volume * self.music_volume();

            let buffer_source = self.context.create_buffer_source().unwrap();
            buffer_source.set_buffer(Some(&audio_clip.buffer));

            let gain_node = self.context.create_gain().unwrap();
            gain_node.gain().set_value(real_volume);

            buffer_source.connect_with_audio_node(&gain_node).unwrap();
            gain_node
                .connect_with_audio_node(&self.context.destination())
                .unwrap();

            buffer_source.set_loop(true);

            buffer_source.start_with_when(0.0).unwrap();

            self.music_gain = Some(gain_node);
        }
    }

    pub fn play_random_zap(&self, hits: usize) {
        let rand = Math::random();

        if rand < 0.33 {
            self.play_clip(ClipId::ZapI);
        } else if rand < 0.66 {
            self.play_clip(ClipId::ZapII);
        } else {
            self.play_clip(ClipId::ZapIII);
        }

        match hits {
            0 => (),
            1 => self.play_clip(ClipId::CrackleI),
            2 => self.play_clip(ClipId::CrackleII),
            _ => self.play_clip(ClipId::CrackleIII),
        }
    }

    pub async fn populate_audio(&mut self) {
        {
            // COMBAT Crackle Implemented
            self.register_audio_clip(
                ClipId::CrackleI,
                include_bytes!("../../static/wav/COMBAT_Crackle_1.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::CrackleII,
                include_bytes!("../../static/wav/COMBAT_Crackle_2.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::CrackleIII,
                include_bytes!("../../static/wav/COMBAT_Crackle_3.wav"),
                1.0,
            )
            .await;
        }

        {
            // COMBAT Hit Implemented
            self.register_audio_clip(
                ClipId::ZapI,
                include_bytes!("../../static/wav/COMBAT_Hit_1.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::ZapII,
                include_bytes!("../../static/wav/COMBAT_Hit_2.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::ZapII,
                include_bytes!("../../static/wav/COMBAT_Hit_3.wav"),
                1.0,
            )
            .await;
        }

        {
            // POWERUP Implemented
            self.register_audio_clip(
                ClipId::Diagonal,
                include_bytes!("../../static/wav/POWERUP_Diagonal.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::Beam,
                include_bytes!("../../static/wav/POWERUP_BigLaser.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::Shield,
                include_bytes!("../../static/wav/POWERUP_Shield.wav"),
                1.0,
            )
            .await;
        }

        {
            // UI Battle Implemented
            self.register_audio_clip(
                ClipId::MageDeselect,
                include_bytes!("../../static/wav/UI_Battle_MageDeSelect.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::MageSelect,
                include_bytes!("../../static/wav/UI_Battle_MageSelect.wav"),
                1.0,
            )
            .await;
            // self.register_audio_clip(
            //     ClipId::MageMove,
            //     include_bytes!("../../static/wav/UI_Battle_MageMoveToSquare_2.wav"),
            //     1.0,
            // )
            // .await;
        }

        {
            // UI Click Implemented
            self.register_audio_clip(
                ClipId::ClickBack,
                include_bytes!("../../static/wav/UI_Click_Back.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::ClickForward,
                include_bytes!("../../static/wav/UI_Click_Forward.wav"),
                1.0,
            )
            .await;
        }

        {
            // UI Level
            self.register_audio_clip(
                ClipId::LevelEnter,
                include_bytes!("../../static/wav/UI_LevelChangeWhoosh.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::LevelSuccess,
                include_bytes!("../../static/wav/UI_LevelFinish_Success.wav"),
                1.0,
            )
            .await;
            self.register_audio_clip(
                ClipId::LevelFailure,
                include_bytes!("../../static/wav/UI_LevelFinish_Failure.wav"),
                1.0,
            )
            .await;
            // self.register_audio_clip(
            //     ClipId::StarSparkle,
            //     include_bytes!("../../static/wav/UI_LevelCompleteCrystals.wav"),
            //     1.0,
            // )
            // .await;
        }

        // {
        //     self.register_audio_clip(
        //         ClipId::MusicI,
        //         include_bytes!("../../static/wav/music_1.mp3"),
        //         1.0,
        //     )
        //     .await;
        // }
    }
}

fn u8_slice_to_array_buffer(u8_slice: &[u8]) -> ArrayBuffer {
    let uint8_array = Uint8Array::new_with_length(u8_slice.len() as u32);
    uint8_array.set(&Uint8Array::from(u8_slice), 0);
    ArrayBuffer::from(uint8_array.buffer())
}

impl Default for AudioSystem {
    fn default() -> Self {
        let (music_volume, clip_volume) = SettingsMenuState::load_volume();

        Self {
            context: AudioContext::new().unwrap(),
            audio_clips: Default::default(),
            base_volume: 1.0,
            music_gain: None,
            music_volume,
            clip_volume,
        }
    }
}
