
use js_sys::{ArrayBuffer, Math, Uint8Array};
use std::collections::HashMap;
use wasm_bindgen::JsCast;

use web_sys::{
    AudioBuffer, AudioContext,
};

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
}

#[derive(Clone, Debug)]
pub struct AudioClip {
    buffer: AudioBuffer,
    volume: f64,
}

#[derive(Clone, Debug)]
pub struct AudioSystem {
    context: AudioContext,
    audio_clips: HashMap<ClipId, AudioClip>,
}

impl AudioSystem {
    pub async fn register_audio_clip(&mut self, clip_id: ClipId, data: &[u8], volume: f64) {
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

            // console::log_1(&format!("{:?}", audio_clip).into());

            self.audio_clips.insert(clip_id, audio_clip);
        }
    }

    pub fn play_clip(&self, clip_id: ClipId) {
        // console::log_1(&format!("play_clip {:?}", clip_id).into());
        if let Some(audio_clip) = self.audio_clips.get(&clip_id) {
            // console::log_1(&format!("play_clip audio_clip {:?}", audio_clip).into());
            
            // let audio_element = audio_clip.audio_element;
            // audio_clip.audio_element.set_current_time(0.0);
            // let _ = audio_clip.audio_element.play();

            // if let Some(buffer_source) = &audio_clip.buffer_source {
            // } else {
            //     // audio_clip.audio_promise.
            // }

            let buffer_source = self.context.create_buffer_source().unwrap();
            buffer_source.set_buffer(Some(&audio_clip.buffer));
            // buffer_source
            //     .connect_with_audio_node(&self.context.destination().into())
            //     .unwrap();

            // // let _ = buffer_source.start();

            let gain_node = self.context.create_gain().unwrap();
            gain_node.gain().set_value(audio_clip.volume as f32);

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
            self.register_audio_clip(
                ClipId::MageMove,
                include_bytes!("../../static/wav/UI_Battle_MageMoveToSquare.wav"),
                1.0,
            )
            .await;
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
            self.register_audio_clip(
                ClipId::StarSparkle,
                include_bytes!("../../static/wav/UI_LevelCompletedStar_LOOP.wav"),
                1.0,
            )
            .await;
        }
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        

        Self {
            context: AudioContext::new().unwrap(),
            audio_clips: Default::default(),
        }
    }
}

fn u8_slice_to_array_buffer(u8_slice: &[u8]) -> ArrayBuffer {
    let uint8_array = Uint8Array::new_with_length(u8_slice.len() as u32);
    uint8_array.set(&Uint8Array::from(u8_slice), 0);
    uint8_array.buffer()
}
