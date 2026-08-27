use crate::mechanical_port::source::{
    animation::{keyed_callback_reporter::KeyedCallbackReporter, keyframe::KeyFrame},
    generated::animation::{
        keyed_object_base::KeyedObjectBase, keyed_property_base::KeyedPropertyBase,
    },
    importers::{import_stack::ImportStack, keyed_object_importer::KeyedObjectImporter},
    status_code::StatusCode,
};
pub trait KeyFrameBehavior {
    fn seconds(&self) -> f32;
    fn on_added_dirty(&mut self, context: *mut ()) -> StatusCode;
    fn on_added_clean(&mut self, context: *mut ()) -> StatusCode;
    fn interpolation_type(&self) -> u32;
    fn apply(&self, object: *mut (), key: i32, mix: f32, context: *const ());
    fn interpolate(
        &self,
        object: *mut (),
        key: i32,
        time: f32,
        next: &dyn KeyFrameBehavior,
        mix: f32,
        context: *const (),
    );
}
#[derive(Default)]
pub struct KeyedProperty {
    pub base: KeyedPropertyBase,
    keyframes: Vec<Box<dyn KeyFrameBehavior>>,
}
impl KeyedProperty {
    pub fn add_key_frame<T: KeyFrameBehavior + 'static>(&mut self, value: Box<T>) {
        self.keyframes.push(value)
    }
    fn closest_frame_index(&self, seconds: f32, offset: i32) -> i32 {
        let mut start = 0;
        let mut end = self.keyframes.len() as i32 - 1;
        if seconds > self.keyframes[end as usize].seconds() {
            return end + 1;
        }
        while start <= end {
            let mid = (start + end) >> 1;
            let value = self.keyframes[mid as usize].seconds();
            if value < seconds {
                start = mid + 1
            } else if value > seconds {
                end = mid - 1
            } else {
                return mid + offset;
            }
        }
        start
    }
    pub fn report_keyed_callbacks(
        &self,
        reporter: &mut dyn KeyedCallbackReporter,
        object: u32,
        from: f32,
        to: f32,
        at_start: bool,
    ) {
        if from == to {
            return;
        }
        let forward = from <= to;
        let from_offset = if (forward && !at_start) || (!forward && at_start) {
            1
        } else {
            0
        };
        let mut index = self.closest_frame_index(from, from_offset);
        let mut end = self.closest_frame_index(to, if forward { 1 } else { 0 });
        if end < index {
            std::mem::swap(&mut index, &mut end)
        }
        while end > index {
            let frame = &self.keyframes[index as usize];
            reporter.report_keyed_callback(object, self.base.property_key(), to - frame.seconds());
            index += 1;
        }
    }
    pub fn apply(
        &self,
        object: *mut (),
        seconds: f32,
        mix: f32,
        context: *const (),
        override_mix: bool,
    ) {
        assert!(!self.keyframes.is_empty());
        let mix = if override_mix { 1.0 } else { mix };
        let index = self.closest_frame_index(seconds, 0);
        if index == 0 {
            self.keyframes[0].apply(object, self.base.property_key() as i32, mix, context)
        } else if index < self.keyframes.len() as i32 {
            let from = &self.keyframes[index as usize - 1];
            let to = &self.keyframes[index as usize];
            if seconds == to.seconds() {
                to.apply(object, self.base.property_key() as i32, mix, context)
            } else if from.interpolation_type() == 0 {
                from.apply(object, self.base.property_key() as i32, mix, context)
            } else {
                from.interpolate(
                    object,
                    self.base.property_key() as i32,
                    seconds,
                    to.as_ref(),
                    mix,
                    context,
                )
            }
        } else {
            self.keyframes[index as usize - 1].apply(
                object,
                self.base.property_key() as i32,
                mix,
                context,
            )
        }
    }
    pub fn on_added_dirty(&mut self, c: *mut ()) -> StatusCode {
        for f in &mut self.keyframes {
            let s = f.on_added_dirty(c);
            if s != StatusCode::Ok {
                return s;
            }
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, c: *mut ()) -> StatusCode {
        for f in &mut self.keyframes {
            let s = f.on_added_clean(c);
            if s != StatusCode::Ok {
                return s;
            }
        }
        StatusCode::Ok
    }
    pub fn import(self: Box<Self>, stack: &mut ImportStack) -> StatusCode {
        let raw = Box::into_raw(self);
        let Some(i) = stack.latest::<KeyedObjectImporter>(KeyedObjectBase::TYPE_KEY) else {
            unsafe { drop(Box::from_raw(raw)) };
            return StatusCode::MissingObject;
        };
        i.add_keyed_property(unsafe { Box::from_raw(raw) });
        unsafe { (*raw).base.base.import(stack) }
    }
    pub fn first(&self) -> Option<&dyn KeyFrameBehavior> {
        self.keyframes.first().map(Box::as_ref)
    }
    pub fn num_key_frames(&self) -> usize {
        self.keyframes.len()
    }
    pub fn get_key_frame(&self, i: usize) -> Option<&dyn KeyFrameBehavior> {
        self.keyframes.get(i).map(Box::as_ref)
    }
    pub fn is_callback(&self) -> bool {
        self.base.property_key() == 395
    }
    pub fn apply_to_object(&mut self, _id: u32, t: f32, m: f32, c: *const ()) {
        self.apply(std::ptr::null_mut(), t, m, c, false)
    }
}
