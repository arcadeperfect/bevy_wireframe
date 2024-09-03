
import bpy
import bmesh

def add_indices():
    selected_objects = bpy.context.selected_objects

    if not selected_objects:
        print("No objects selected. Please select at least one object.")
        return

    for primitive_idx, obj in enumerate(selected_objects):
        if obj.type != 'MESH':
            print(f"Skipping {obj.name}: Not a mesh object")
            continue

        # Assign primitive index to the object
        obj["gltf_primitive_index"] = primitive_idx

        # Create a BMesh from the object data
        bm = bmesh.new()
        bm.from_mesh(obj.data)

        # Ensure lookup table is up-to-date
        bm.verts.ensure_lookup_table()

        # Create a new custom attribute layer for vertex indices
        vert_index_layer = bm.verts.layers.int.new('_VERT_INDEX')

        # Iterate through all vertices and assign unique IDs
        for idx, v in enumerate(bm.verts):
            v[vert_index_layer] = idx

        # Update the mesh with BMesh data
        bm.to_mesh(obj.data)
        obj.data.update()

        # Free the BMesh
        bm.free()

        print(f"Object: {obj.name}")
        print(f"  Assigned primitive index: {primitive_idx}")
        print(f"  Added '_VERT_INDEX' attribute to {len(obj.data.vertices)} vertices")

# Run the function
add_indices()