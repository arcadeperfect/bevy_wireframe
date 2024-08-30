let line_mesh_handle = mesh_assets.add(m);

let line_material_handle = line_materials.add(LineMaterial {
    displacement: 1.5,
    ..default()
});

let bundle = MaterialMeshBundle {
    mesh: line_mesh_handle,
    material: line_materials.add(LineMaterial {
        displacement: 1.5,
        ..default()
    }),
    transform: Transform::IDENTITY,
    ..Default::default()
};



// let entity_commands = commands.entity(entity).with_children(|parent| {
//     parent.spawn((
//         bundle,
//         LineMesh,
//     ));
// });

let skinned_mesh = skinned_meshes.get(entity).cloned();

if let Ok(skinned_mesh) = skinned_mesh {
    println!("a");
    let entity_commands = commands.entity(entity).with_children(|parent| {
        parent.spawn((
            bundle,
            LineMesh,
        ));
    }).insert(skinned_mesh);
}
else {
    println!("b");
    let entity_commands = commands.entity(entity).with_children(|parent| {
        parent.spawn((
            bundle,
            LineMesh,
        ));
    });
}
///////////////////////

for entity in children.iter_descendants(event.parent) {
    if let (Ok((entity, mesh_handle)), Ok(wireframe_settings)) =
        (meshes.get(entity), processable_scenes.get(event.parent))
    {
        if let Some(flat_mesh) = mesh_assets.get_mut(mesh_handle) {
            commands.entity(entity).remove::<Handle<StandardMaterial>>();
            flat_mesh.randomize_vertex_colors();

            let smoothed_normals = get_smoothed_normals(flat_mesh).unwrap();
            flat_mesh.insert_attribute(ATTRIBUTE_SMOOTHED_NORMAL, smoothed_normals);
            
            
            let mut smooth_mesh = flat_mesh.clone();
            
            // smooth_mesh.compute_smooth_normals();
            // smooth_mesh.smooth_normals_non_indexed();
            flat_mesh.duplicate_vertices();
            flat_mesh.compute_flat_normals();
            // flat_mesh.compute_normals();

            // Add FillMaterial component
            let fill_material_handle = fill_materials.add(FillMaterial {
                color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                displacement: 0.0,
                shininess: 200.0,
                specular_strength: 1.0,
            });
            commands.entity(entity).insert(fill_material_handle.clone());

            // Add OutlineMaterial component
            let outline_material_handle = outline_materials.add(OutlineMaterial {
                outline_width: shader_settings.outline_width,
                ..default()
            });
            commands
                .entity(entity)
                .insert(outline_material_handle.clone());

            // let custom_line_list = None;
            match mesh_to_wireframe(&mut smooth_mesh, &wireframe_settings, &parsed_extra) {
                Ok(_) => {}
                Err(e) => {
                    panic!("fuckkkkkkkk");
                    // warn!("Error: {:?}", e);
                }
            }
            // mesh_to_wireframe(&mut smooth_mesh, &wireframe_settings);

            let new_mesh_handle = mesh_assets.add(smooth_mesh);
            let skinned_mesh = skinned_meshes.get(entity).cloned();

            let bundle = MaterialMeshBundle {
                mesh: new_mesh_handle,
                material: line_materials.add(LineMaterial {
                    displacement: 1.5,
                    ..default()
                }),
                ..Default::default()
            };

            // Spawn the new entity
            let mut entity_commands = commands.spawn(bundle);

            // If the original entity had a SkinnedMesh component, add it to the new entity
            if let Ok(skinned_mesh) = skinned_mesh {
                entity_commands.insert(skinned_mesh);
            }
        }
    }
}