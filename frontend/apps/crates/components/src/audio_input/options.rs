use shared::{domain::audio::AudioId, media::MediaLibrary};
pub struct AudioInputOptions {
    pub on_change: Option<Box<dyn Fn(Option<AudioId>)>>,
    pub audio_id: Option<AudioId>, //initial audio id
}
