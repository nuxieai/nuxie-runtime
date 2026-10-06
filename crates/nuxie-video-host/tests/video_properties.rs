use nuxie_runtime::{
    source::{
        core::CoreArena,
        data_bind::data_bind::{DataBind, TO_SOURCE},
        generated::core_registry::CoreRegistry,
        viewmodel::viewmodel_instance_number::ViewModelInstanceNumber,
    },
    video::{
        Video,
        playback::{DecoderAction, Playback},
        resources::Allocation,
    },
};
use nuxie_video_host::scene::{Decoder, Frame, FramePixels, ScenePlayer};

struct FailingDecoder {
    fail_tick: bool,
}
impl Decoder for FailingDecoder {
    type Error = &'static str;
    fn apply(&mut self, _: DecoderAction) -> Result<(), Self::Error> {
        Ok(())
    }
    fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, Self::Error> {
        if self.fail_tick {
            return Err("decode");
        }
        Ok(Some(Frame {
            generation: playback.generation(),
            pts: 0.0,
            width: 1,
            height: 1,
            pixels: FramePixels::Rgba(vec![0; 4]),
        }))
    }
}

#[test]
fn every_native_failure_path_publishes_failed_state_to_bindings() {
    for failure in ["open", "decode", "upload"] {
        let arena = CoreArena::default();
        let video = arena.insert(Video::default());
        let number = arena.insert(ViewModelInstanceNumber::default());
        let binding = arena.insert(DataBind::new(TO_SOURCE, 60020, 0));
        binding.with_mut(|b| {
            let b = b.as_data_bind_mut().unwrap();
            b.set_target(Some(video.clone()));
            b.set_source(number.clone());
        });
        DataBind::bind_handle(&binding);
        DataBind::update_data_bind_handle(&binding, true);
        assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(0.0));
        let mut player = ScenePlayer::new(video, 1.0, false).unwrap();
        let result = player.tick(
            Allocation::Hardware,
            0.0,
            |_, _| {
                if failure == "open" {
                    Err("open")
                } else {
                    Ok(FailingDecoder {
                        fail_tick: failure == "decode",
                    })
                }
            },
            |_| Err("upload"),
        );
        assert!(result.is_err());
        DataBind::update_data_bind_handle(&binding, true);
        assert_eq!(
            CoreRegistry::get_double_handle(&number, 575),
            Some(7.0),
            "{failure}"
        );
    }
}
