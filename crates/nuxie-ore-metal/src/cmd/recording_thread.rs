//! renderer/cmd/recording_thread.hpp at e949498e.
#[derive(Default)]
pub struct RecordingThread {
    #[cfg(debug_assertions)]
    id: Option<std::thread::ThreadId>,
}
impl RecordingThread {
    pub fn bind(&mut self) {
        #[cfg(debug_assertions)]
        {
            self.id = Some(std::thread::current().id());
        }
    }
    pub fn check(&self) {
        #[cfg(debug_assertions)]
        assert!(
            self.id.is_none_or(|id| id == std::thread::current().id()),
            "deferred recording is single threaded: this stream, its id allocator, and its keep alive table are all unlocked"
        );
    }
}
