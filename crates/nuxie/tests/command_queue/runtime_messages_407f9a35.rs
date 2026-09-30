//! All four runtime-message cases from upstream 407f9a35, including both
//! discard sections. The callback pumps the actual server on the same thread
//! instead of waiting for the upstream C++ worker.
use super::*;
use nuxie::command_queue::{RuntimeMessageListener, RuntimeMessageListenerHandle};

thread_local! {
    static CALLBACK_SERVER: RefCell<Option<Box<CommandServer>>> = const { RefCell::new(None) };
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReceivedMessage {
    tag: u32,
    payload: Vec<u8>,
}

struct RuntimeMessageTestListener {
    received: Arc<Mutex<Vec<ReceivedMessage>>>,
    queue: Option<CommandQueue>,
    followup_tag: u32,
    followup_payload: Vec<u8>,
}
impl RuntimeMessageListener for RuntimeMessageTestListener {
    fn on_runtime_message(&mut self, tag: u32, payload: Vec<u8>) {
        let first = {
            let mut received = self.received.lock().unwrap();
            received.push(ReceivedMessage { tag, payload });
            received.len() == 1
        };
        if first && let Some(queue) = &mut self.queue {
            let tag = self.followup_tag;
            let payload = self.followup_payload.clone();
            queue.run_once(Box::new(move |server| {
                server.post_runtime_message(tag, payload)
            }));
            // Process the enqueued command before returning from this message
            // callback, reproducing the upstream worker's completion barrier.
            CALLBACK_SERVER.with(|slot| {
                assert!(slot.borrow_mut().as_mut().unwrap().process_commands());
            });
        }
    }
}
fn listener(
    queue: Option<CommandQueue>,
) -> (
    RuntimeMessageListenerHandle,
    Arc<Mutex<Vec<ReceivedMessage>>>,
) {
    let received = Arc::new(Mutex::new(Vec::new()));
    let listener = RuntimeMessageListenerHandle::new(Box::new(RuntimeMessageTestListener {
        received: received.clone(),
        queue,
        followup_tag: 2,
        followup_payload: vec![4, 5, 6],
    }));
    (listener, received)
}

#[test]
fn runtime_messages_are_delivered_in_order() {
    let mut queue = CommandQueue::new();
    let mut server = server(&queue);
    let (listener, received) = listener(None);
    queue.set_global_runtime_message_listener(Some(&listener));
    queue.run_once(Box::new(|server| {
        server.post_runtime_message(7, vec![1, 2, 3]);
        server.post_runtime_message(11, vec![]);
        server.post_runtime_message(13, vec![9]);
    }));
    server.process_commands();
    queue.process_messages();
    assert_eq!(
        *received.lock().unwrap(),
        vec![
            ReceivedMessage {
                tag: 7,
                payload: vec![1, 2, 3]
            },
            ReceivedMessage {
                tag: 11,
                payload: vec![]
            },
            ReceivedMessage {
                tag: 13,
                payload: vec![9]
            },
        ]
    );
    queue.disconnect();
}

#[test]
fn runtime_messages_without_a_listener_are_discarded() {
    for registered_before_delivery in [false, true] {
        let mut queue = CommandQueue::new();
        let mut server = server(&queue);
        let (listener, received) = listener(None);
        if registered_before_delivery {
            queue.set_global_runtime_message_listener(Some(&listener));
        }
        queue.run_once(Box::new(|server| {
            server.post_runtime_message(1, vec![1, 2, 3]);
            server.post_runtime_message(2, vec![]);
        }));
        server.process_commands();
        queue.set_global_runtime_message_listener(None);
        queue.process_messages();
        assert!(received.lock().unwrap().is_empty());
        queue.set_global_runtime_message_listener(Some(&listener));
        queue.run_once(Box::new(|server| {
            server.post_runtime_message(3, vec![4, 5])
        }));
        server.process_commands();
        queue.process_messages();
        assert_eq!(
            *received.lock().unwrap(),
            vec![ReceivedMessage {
                tag: 3,
                payload: vec![4, 5]
            }]
        );
        queue.set_global_runtime_message_listener(None);
        queue.disconnect();
    }
}

#[test]
fn runtime_messages_can_be_posted_from_draw_callbacks() {
    let mut queue = CommandQueue::new();
    let mut server = server(&queue);
    let (listener, received) = listener(None);
    queue.set_global_runtime_message_listener(Some(&listener));
    let key = queue.create_draw_key();
    queue.draw(
        key,
        Box::new(|_, server| server.post_runtime_message(7, vec![1, 2, 3])),
    );
    server.process_commands();
    assert!(received.lock().unwrap().is_empty());
    queue.process_messages();
    assert_eq!(
        *received.lock().unwrap(),
        vec![ReceivedMessage {
            tag: 7,
            payload: vec![1, 2, 3]
        }]
    );
    queue.set_global_runtime_message_listener(None);
    queue.disconnect();
}

#[test]
fn runtime_messages_posted_while_processing_wait_until_the_next_pass() {
    let mut queue = CommandQueue::new();
    CALLBACK_SERVER.with(|slot| {
        *slot.borrow_mut() = Some(server(&queue));
    });
    let (listener, received) = listener(Some(queue.clone()));
    queue.set_global_runtime_message_listener(Some(&listener));
    queue.run_once(Box::new(|server| {
        server.post_runtime_message(1, vec![1, 2, 3])
    }));
    CALLBACK_SERVER.with(|slot| {
        slot.borrow_mut().as_mut().unwrap().process_commands();
    });
    queue.process_messages();
    let initial_messages = received.lock().unwrap().clone();
    queue.process_messages();
    queue.disconnect();
    CALLBACK_SERVER.with(|slot| {
        slot.borrow_mut().take();
    });
    assert_eq!(
        initial_messages,
        vec![ReceivedMessage {
            tag: 1,
            payload: vec![1, 2, 3]
        }]
    );
    let received = received.lock().unwrap();
    assert_eq!(received.len(), 2);
    assert_eq!(
        received[1],
        ReceivedMessage {
            tag: 2,
            payload: vec![4, 5, 6]
        }
    );
}
