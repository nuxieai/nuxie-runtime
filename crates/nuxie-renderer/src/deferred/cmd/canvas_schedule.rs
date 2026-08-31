//! renderer/cmd/canvas_schedule.hpp at e949498e.
use super::deferred_session::{DeferredSegment, SegmentTarget};
use super::render_commands::{payload_size_of, RenderCmd};
use super::render_handle::{CANVAS_HANDLE_FLAG, CANVAS_HANDLE_MASK, INVALID_RENDER_HANDLE};

#[derive(Default, Debug)]
pub struct CanvasSchedule {
    pub order: Vec<u64>,
    pub had_cycle: bool,
    pub multi_write_fallback: bool,
}

pub fn drawn_image_handle(pod: &[u8]) -> u32 {
    u32::from_ne_bytes(pod[..4].try_into().expect("recorded image handle"))
}

pub fn schedule_canvases(commands: &[u8], segments: &[DeferredSegment]) -> CanvasSchedule {
    let mut result = CanvasSchedule::default();
    struct Node {
        canvas_id: u64,
        first_begin: u32,
        last_begin: u32,
    }
    let mut nodes: Vec<Node> = Vec::new();
    for segment in segments {
        if segment.target != SegmentTarget::Canvas {
            continue;
        }
        if let Some(node) = nodes.iter_mut().find(|n| n.canvas_id == segment.target_id) {
            node.last_begin = segment.begin;
        } else {
            nodes.push(Node {
                canvas_id: segment.target_id,
                first_begin: segment.begin,
                last_begin: segment.begin,
            });
        }
    }
    if nodes.is_empty() {
        return result;
    }
    let node_for = |id| nodes.iter().position(|n| n.canvas_id == id);
    let mut dependencies = vec![Vec::new(); nodes.len()];
    for segment in segments {
        if segment.target != SegmentTarget::Canvas {
            continue;
        }
        let reader = node_for(segment.target_id).expect("written canvas");
        let mut pos = segment.begin;
        while pos < segment.end && (pos as usize) < commands.len() {
            let Some(command) = RenderCmd::from_byte(commands[pos as usize]) else {
                break;
            };
            let payload = payload_size_of(command) as u32;
            if matches!(command, RenderCmd::DrawImage | RenderCmd::DrawImageMesh) {
                let handle = drawn_image_handle(&commands[pos as usize + 1..]);
                if handle != INVALID_RENDER_HANDLE && handle & CANVAS_HANDLE_FLAG != 0 {
                    if let Some(sampled) = node_for(u64::from(handle & CANVAS_HANDLE_MASK)) {
                        if sampled == reader {
                            result.had_cycle = true;
                        } else {
                            if nodes[sampled].first_begin < pos && nodes[sampled].last_begin > pos {
                                result.multi_write_fallback = true;
                            }
                            dependencies[reader].push(sampled);
                        }
                    }
                }
            }
            pos = pos.wrapping_add(1 + payload);
        }
    }
    if result.multi_write_fallback {
        result.order.extend(nodes.iter().map(|n| n.canvas_id));
        return result;
    }
    let mut done = vec![false; nodes.len()];
    while result.order.len() < nodes.len() {
        let pick = (0..nodes.len()).find(|&i| !done[i] && dependencies[i].iter().all(|&d| done[d]));
        let pick = pick.unwrap_or_else(|| {
            result.had_cycle = true;
            done.iter()
                .position(|&value| !value)
                .expect("remaining canvas")
        });
        done[pick] = true;
        result.order.push(nodes[pick].canvas_id);
    }
    result
}
