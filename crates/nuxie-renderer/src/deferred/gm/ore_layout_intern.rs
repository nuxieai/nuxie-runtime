//! tests/unit_tests/renderer/ore_layout_intern_test.cpp at 1cdecb8e.
//! Execute the upstream identity scenario on this host's live Metal backend.
use super::ore_gm_helper::*;
use nuxie_ore_metal::{
    bind_group_layout::{makeBindGroupLayoutFromShader, populateBindGroupLayoutEntriesFromShader},
    binding_map::BindingMap,
};

#[test]
fn ore_layouts_intern_by_baked_id() {
    let host = GmHost::new(0xff000000);
    let mut ctx = host.ore.borrow_mut();
    let shader = super::ore_gm_helper::shader(&mut *ctx, 6);
    let module = shader.shaderModuleBase().expect("binding witness module");
    assert_ne!(
        module.m_bindingMap.layoutIdForGroup(0),
        BindingMap::kNoLayoutId
    );
    let first = makeBindGroupLayoutFromShader(&mut *ctx, Some(module), 0, &[]).unwrap();
    assert_eq!(
        makeBindGroupLayoutFromShader(&mut *ctx, Some(module), 0, &[])
            .unwrap()
            .allocation_identity(),
        first.allocation_identity(),
    );

    let other = super::ore_gm_helper::shader(&mut *ctx, 6);
    assert_ne!(other.allocation_identity(), shader.allocation_identity());
    assert_eq!(
        makeBindGroupLayoutFromShader(&mut *ctx, other.shaderModuleBase(), 0, &[])
            .unwrap()
            .allocation_identity(),
        first.allocation_identity(),
    );

    let multi = super::ore_gm_helper::shader(&mut *ctx, 8);
    let multi_module = multi.shaderModuleBase().unwrap();
    assert_ne!(
        multi_module.m_bindingMap.layoutIdForGroup(0),
        multi_module.m_bindingMap.layoutIdForGroup(1),
    );
    let g0 = makeBindGroupLayoutFromShader(&mut *ctx, Some(multi_module), 0, &[]).unwrap();
    let g1 = makeBindGroupLayoutFromShader(&mut *ctx, Some(multi_module), 1, &[]).unwrap();
    assert_ne!(g0.allocation_identity(), g1.allocation_identity());

    let mut room = [BindGroupLayoutEntry::default(); 16];
    let needed = populateBindGroupLayoutEntriesFromShader(&mut room, Some(module), 0, &[]);
    assert!(needed > 0);
    assert_eq!(
        populateBindGroupLayoutEntriesFromShader(&mut [], Some(module), 0, &[]),
        needed,
    );

    let dynamic = makeBindGroupLayoutFromShader(&mut *ctx, Some(module), 0, &[0]).unwrap();
    assert_ne!(dynamic.allocation_identity(), first.allocation_identity());
    assert_eq!(
        makeBindGroupLayoutFromShader(&mut *ctx, Some(module), 0, &[])
            .unwrap()
            .allocation_identity(),
        first.allocation_identity(),
    );
}

#[test]
fn wide_reflected_layout_spills_without_losing_bindings() {
    let host = GmHost::new(0xff000000);
    let mut module = nuxie_ore_metal::new_shader_module_backend_base();
    let mut blob = vec![3, 2, 14, 0, 17, 0, 0, 0, 9, 0, 0, 0];
    for binding in 0..17 {
        blob.extend_from_slice(&[
            0, binding, 0, 3, 0, binding, 0, binding, 0, 255, 255, 0, 0, 0,
        ]);
    }
    assert!(BindingMap::fromBlob(
        Some(&blob),
        blob.len(),
        Some(&mut module.m_bindingMap)
    ));
    let layout =
        makeBindGroupLayoutFromShader(&mut *host.ore.borrow_mut(), Some(&module), 0, &[]).unwrap();
    let entries = layout.bindGroupLayoutBase().unwrap().entries();
    assert_eq!(entries.len(), 17);
    assert_eq!(entries[16].binding, 16);
}
