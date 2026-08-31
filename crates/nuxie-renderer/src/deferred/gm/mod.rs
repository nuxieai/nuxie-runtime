//! The eight deferred GM source owners added by upstream e949498e.
//! These use the live Metal backend, not a command-only stand-in. The paired
//! GMs use gmmain.cpp's non-atomic comparison (zero channel difference).
mod ore_deferred_context;
mod ore_deferred_multipass;
mod ore_deferred_replay;
mod ore_deferred_resource;
mod ore_gm_helper;
mod ore_render_deferred_canvas;
mod render_canvas_dag;
mod render_deferred_2d;
mod runtime_deferred_import;
mod serialized_replay_2d;
