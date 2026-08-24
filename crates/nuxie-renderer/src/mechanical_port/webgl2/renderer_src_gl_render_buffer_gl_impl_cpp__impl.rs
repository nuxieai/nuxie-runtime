//! Complete mechanical implementation translation of
//! `renderer/src/gl/render_buffer_gl_impl.cpp` for `RIVE_WEBGL`.

#![allow(non_snake_case)]

use super::gles3_decl::*;
use super::render_buffer_gl_impl_decl::{GLStateOwner, RenderBufferGLImpl};
use crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_gl_render_buffer_gl_impl.cpp");

pub(crate) fn init(buffer: &mut RenderBufferGLImpl, state: GLStateOwner) {
    let execution = state
        .borrow()
        .executionStamp()
        .expect("RenderBufferGLImpl requires GLState execution authority");
    execution.withCurrent(|| {
        buffer.base.base.install_owner_thread_final_release_route(
            execution.domain().ownerThreadFinalReleaseRoute(),
        );
        let bufferID = generateGLObject(GLObjectKind::Buffer);
        buffer.installNativeOwner(state, execution.clone(), bufferID);
        let mut state = buffer.state().borrow_mut();
        state.bindVAO(0);
        state.bindBuffer(buffer.m_target, buffer.bufferID());
        recordGLCommand(GLCommand::BufferData {
            target: buffer.m_target,
            size: buffer.base.sizeInBytes(),
            data: None,
            usage: if buffer.base.flags() as u8
                & RenderBufferFlags::mappedOnceAtInitialization as u8
                != 0
            {
                GL_STATIC_DRAW
            } else {
                GL_DYNAMIC_DRAW
            },
        });
    });
}

pub(crate) fn onMap(buffer: &mut RenderBufferGLImpl) -> *mut core::ffi::c_void {
    let execution = buffer.executionStamp().clone();
    execution.withCurrent(|| {
        if buffer.m_fallbackMappedMemory.is_none() {
            *buffer.m_fallbackMappedMemory = Some(vec![0; buffer.base.sizeInBytes()]);
        }
        buffer
            .m_fallbackMappedMemory
            .as_mut()
            .expect("WebGL fallback mapping was allocated")
            .as_mut_ptr()
            .cast()
    })
}

pub(crate) fn onUnmap(buffer: &mut RenderBufferGLImpl) {
    let execution = buffer.executionStamp().clone();
    execution.withCurrent(|| {
        let bytes = buffer
            .m_fallbackMappedMemory
            .as_ref()
            .expect("RenderBufferGLImpl must be mapped before unmap")
            .clone();
        {
            let mut state = buffer.state().borrow_mut();
            state.bindVAO(0);
            state.bindBuffer(buffer.m_target, buffer.bufferID());
        }
        recordGLCommand(GLCommand::BufferSubData {
            target: buffer.m_target,
            offset: 0,
            data: bytes,
        });
        if buffer.base.flags() as u8 & RenderBufferFlags::mappedOnceAtInitialization as u8 != 0 {
            *buffer.m_fallbackMappedMemory = None;
        }
    });
}

impl Drop for RenderBufferGLImpl {
    fn drop(&mut self) {
        if let Some(execution) = self.executionStampOption().cloned() {
            let _ = execution.withDeleteCurrent(|| {
                if self.bufferID() != 0 {
                    self.state().borrow_mut().deleteBuffer(self.bufferID());
                }
            });
        } else {
            assert_eq!(self.bufferID(), 0);
        }
        unsafe { self.dropOwnedSourceFields() };
    }
}

#[cfg(test)]
mod tests {
    use super::super::gl_state_decl::GLState;
    use super::*;
    use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
    use crate::mechanical_port::source::include::rive::renderer_hpp::{
        RenderBuffer, RenderBufferFlags, RenderBufferType,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    struct ProviderLog {
        commands: Vec<GLCommand>,
        generated: Vec<(GLObjectKind, GLuint)>,
    }

    struct TestProvider {
        log: Rc<RefCell<ProviderLog>>,
        nextName: GLuint,
        lifecycleIngress: Option<GLContextLifecycleIngress>,
        finalReleaseIngress: Option<GLFinalReleaseIngress>,
    }

    impl TestProvider {
        fn allocate(&mut self) -> GLuint {
            let name = self.nextName;
            self.nextName += 17;
            name
        }
    }

    impl GLExecutionProvider for TestProvider {
        fn installContextLifecycleIngress(&mut self, ingress: GLContextLifecycleIngress) {
            assert!(self.lifecycleIngress.replace(ingress).is_none());
        }

        fn installFinalReleaseIngress(
            &mut self,
            ingress: GLFinalReleaseIngress,
        ) -> std::sync::Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake> {
            assert!(self.finalReleaseIngress.replace(ingress).is_none());
            std::sync::Arc::new(TestFinalReleaseWake::default())
        }

        fn submit(&mut self, command: GLCommand) {
            self.log.borrow_mut().commands.push(command);
        }

        fn generateObject(&mut self, kind: GLObjectKind) -> GLuint {
            let name = self.allocate();
            self.log.borrow_mut().generated.push((kind, name));
            name
        }

        fn createProgram(&mut self) -> GLuint {
            self.allocate()
        }

        fn createShader(&mut self, _shaderType: GLenum) -> GLuint {
            self.allocate()
        }

        fn getInteger(&mut self, _parameter: GLenum) -> GLint {
            0
        }

        fn getString(&mut self, _parameter: GLenum) -> Option<Vec<u8>> {
            None
        }

        fn getExtension(&mut self, _index: GLuint) -> Option<Vec<u8>> {
            None
        }

        fn enableWebGLExtension(&mut self, _name: &str) -> bool {
            false
        }

        fn isObject(&mut self, _kind: GLObjectKind, _name: GLuint) -> bool {
            false
        }

        fn checkFramebufferStatus(&mut self, _target: GLenum) -> GLenum {
            GL_FRAMEBUFFER_COMPLETE
        }

        fn shaderParameter(&mut self, _shader: GLuint, _parameter: GLenum) -> GLint {
            0
        }

        fn shaderInfoLog(&mut self, _shader: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn programParameter(&mut self, _program: GLuint, _parameter: GLenum) -> GLint {
            0
        }

        fn programInfoLog(&mut self, _program: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn uniformBlockIndex(&mut self, _program: GLuint, _name: &[u8]) -> GLuint {
            0
        }

        fn uniformLocation(&mut self, _program: GLuint, _name: &[u8]) -> GLint {
            -1
        }

        fn readPixelsRGBA8(&mut self, _x: i32, _y: i32, _width: u32, _height: u32) -> Vec<u8> {
            Vec::new()
        }

        fn contextLost(&mut self, _nextGeneration: u64) {}

    }

    fn domain(startName: GLuint) -> (GLExecutionDomain, Rc<RefCell<ProviderLog>>) {
        let log = Rc::new(RefCell::new(ProviderLog::default()));
        let domain = GLExecutionDomain::new(Box::new(TestProvider {
            log: log.clone(),
            nextName: startName,
            lifecycleIngress: None,
            finalReleaseIngress: None,
        }));
        (domain, log)
    }

    fn state(domain: &GLExecutionDomain) -> GLStateOwner {
        Rc::new(RefCell::new(GLState::newInDomain(
            GLCapabilities::default(),
            domain.clone(),
        )))
    }

    #[test]
    fn complete_webgl_implementation_denominator_is_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 120);
    }

    #[test]
    fn initialization_and_direct_one_time_unmap_self_scope_exactly() {
        let (domain, log) = domain(101);
        let state = state(&domain);
        log.borrow_mut().commands.clear();
        let mut buffer = RenderBufferGLImpl::new(
            RenderBufferType::vertex,
            RenderBufferFlags::mappedOnceAtInitialization,
            4,
            state,
        );
        assert_eq!(log.borrow().generated, [(GLObjectKind::Buffer, 101)]);
        assert_eq!(
            log.borrow().commands,
            [
                GLCommand::BindVertexArray(0),
                GLCommand::BindBuffer(GL_ARRAY_BUFFER, 101),
                GLCommand::BufferData {
                    target: GL_ARRAY_BUFFER,
                    size: 4,
                    data: None,
                    usage: GL_STATIC_DRAW,
                },
            ]
        );
        log.borrow_mut().commands.clear();
        let mapped = onMap(&mut buffer).cast::<u8>();
        unsafe { std::ptr::copy_nonoverlapping([1_u8, 2, 3, 4].as_ptr(), mapped, 4) };
        // No ambient GL scope is installed here. `onUnmap` must install the
        // buffer's own creation scope around the complete source operation.
        onUnmap(&mut buffer);
        assert!(buffer.m_fallbackMappedMemory.is_none());
        assert_eq!(
            log.borrow().commands,
            [GLCommand::BufferSubData {
                target: GL_ARRAY_BUFFER,
                offset: 0,
                data: vec![1, 2, 3, 4],
            }]
        );
    }

    #[test]
    fn foreign_ambient_domain_cannot_capture_buffer_uploads() {
        let (ownerDomain, ownerLog) = domain(201);
        let (foreignDomain, foreignLog) = domain(901);
        let mut buffer = RenderBufferGLImpl::new(
            RenderBufferType::index,
            RenderBufferFlags::none,
            2,
            state(&ownerDomain),
        );
        onMap(&mut buffer);
        ownerLog.borrow_mut().commands.clear();

        foreignDomain.withCurrent(|| {
            onUnmap(&mut buffer);
            recordGLCommand(GLCommand::Clear(GL_COLOR_BUFFER_BIT));
        });

        assert!(matches!(
            ownerLog.borrow().commands.as_slice(),
            [
                GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, 201),
                GLCommand::BufferSubData {
                    target: GL_ELEMENT_ARRAY_BUFFER,
                    ..
                }
            ]
        ));
        assert_eq!(
            foreignLog.borrow().commands,
            [GLCommand::Clear(GL_COLOR_BUFFER_BIT)]
        );
    }

    #[test]
    fn stale_generation_skips_buffer_delete_but_drops_rust_fields() {
        let (domain, log) = domain(301);
        let buffer = RenderBufferGLImpl::new(
            RenderBufferType::vertex,
            RenderBufferFlags::none,
            8,
            state(&domain),
        );
        log.borrow_mut().commands.clear();
        domain.markContextLost();
        drop(buffer);
        assert!(log.borrow().commands.is_empty());
        domain.shutdown();
    }

    #[test]
    fn retired_renderer_worker_last_release_waits_for_owner_execution_scope() {
        let (domain, log) = domain(401);
        let owner = Box::new(RenderBufferGLImpl::new(
                RenderBufferType::vertex,
                RenderBufferFlags::none,
                8,
                state(&domain),
            ));
        let erased: rcp<RenderBuffer> = unsafe {
            rcp::from_ptr(Box::into_raw(owner).cast::<RenderBuffer>())
        };
        log.borrow_mut().commands.clear();

        domain.retireRenderer();
        assert!(domain.isRendererRetired());
        assert!(
            domain.isLive(),
            "normal renderer retirement preserves the valid context generation"
        );

        std::thread::spawn(move || drop(erased))
            .join()
            .expect("worker releases erased RenderBuffer owner");
        assert!(
            log.borrow().commands.is_empty(),
            "worker zero transition must not run the GL concrete destructor"
        );

        domain.withCurrent(|| {});
        assert_eq!(log.borrow().commands, [GLCommand::DeleteBuffer(401)]);
        domain.shutdown();
    }

    #[test]
    fn borrowed_gl_state_defers_queued_buffer_finalizer_until_later_safe_boundary() {
        let (domain, log) = domain(501);
        let state = state(&domain);
        let owner = Box::new(RenderBufferGLImpl::new(
            RenderBufferType::vertex,
            RenderBufferFlags::none,
            8,
            state.clone(),
        ));
        let erased: rcp<RenderBuffer> =
            unsafe { rcp::from_ptr(Box::into_raw(owner).cast::<RenderBuffer>()) };
        log.borrow_mut().commands.clear();

        std::thread::spawn(move || drop(erased))
            .join()
            .expect("worker queues erased RenderBuffer final release");

        state.borrow_mut().bindProgram(77);
        assert_eq!(
            log.borrow().commands,
            [GLCommand::UseProgram(77)],
            "a live GLState RefMut is not a final-release safe point"
        );

        domain.withCurrent(|| {});
        assert_eq!(
            log.borrow().commands,
            [GLCommand::UseProgram(77), GLCommand::DeleteBuffer(501)]
        );
        domain.shutdown();
    }

    #[test]
    fn uninitialized_owner_has_no_gl_drop_path() {
        drop(RenderBufferGLImpl::newUninitialized(
            RenderBufferType::vertex,
            RenderBufferFlags::none,
            1,
        ));
    }
}
