//! Owned Metal TestingWindow flush-command lifetime for the Rust GM host.
use std::{any::Any, cell::RefCell, rc::Rc};

use nuxie_render_api::RenderCanvasHandle;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{MTLCommandBuffer, MTLCommandQueue};

use super::{
    command_submission::make_command_buffer_on_queue,
    mechanical_render_context::{MechanicalCompletionToken, MechanicalRenderContext},
    render_canvas::NativeMetalRenderCanvas,
};
use crate::{exact_source_adapter::ExactSourceRenderCanvas, RendererError};

pub(crate) struct NativeMetalTestingWindow {
    mechanical: Rc<RefCell<MechanicalRenderContext>>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    // beginFrame deliberately leaves this alone. Canvas replay can flush
    // before the first screen beginFrame, or between screen brackets.
    command: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
}

impl NativeMetalTestingWindow {
    pub(super) fn new(
        mechanical: Rc<RefCell<MechanicalRenderContext>>,
        queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    ) -> Self {
        Self {
            mechanical,
            queue,
            command: None,
        }
    }

    fn take_command(
        &mut self,
    ) -> Result<Retained<ProtocolObject<dyn MTLCommandBuffer>>, RendererError> {
        match self.command.take() {
            Some(command) => Ok(command),
            None => make_command_buffer_on_queue(&self.queue),
        }
    }

    pub(crate) fn flush_canvas(
        &mut self,
        canvas: &RenderCanvasHandle,
    ) -> Result<(), RendererError> {
        // Retain the exact intrusive target owner across the flush, without
        // extending the public generic canvas finish contract.
        let source = {
            let canvas = canvas.borrow();
            let any: &dyn Any = canvas.as_ref();
            if let Some(canvas) = any.downcast_ref::<NativeMetalRenderCanvas>() {
                canvas.window_source()
            } else {
                any.downcast_ref::<ExactSourceRenderCanvas>()
                    .map(ExactSourceRenderCanvas::ref_source)
            }
        }
        .ok_or_else(|| {
            RendererError::NativeMetal("GM window requires a backed source canvas".into())
        })?;
        let command = self.take_command()?;
        self.mechanical
            .borrow_mut()
            .flush_window(Some(unsafe { &mut *source.get() }), command)?;
        // flushPLSContext commits now to release the ring asynchronously,
        // then prepares a fresh command for the next flush/endFrame.
        self.command = Some(make_command_buffer_on_queue(&self.queue)?);
        Ok(())
    }

    pub(super) fn flush_screen(&mut self) -> Result<MechanicalCompletionToken, RendererError> {
        let command = self.take_command()?;
        let completion = self.mechanical.borrow_mut().flush_window(None, command)?;
        self.command = Some(make_command_buffer_on_queue(&self.queue)?);
        Ok(completion)
    }

    pub(super) fn end_frame(&mut self) {
        // The readback adapter waits on the preceding screen flush. The
        // window still commits/releases its successor at terminal endFrame.
        if let Some(command) = self.command.take() {
            command.commit();
        }
    }

    pub(crate) fn pending_command_identity(&self) -> Option<usize> {
        self.command
            .as_ref()
            .map(|command| Retained::as_ptr(command) as usize)
    }
}
