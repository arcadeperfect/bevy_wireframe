# '''
# this script assigns a unique index to each vertex in the active object and stores it as a custom attribute.
# this is we can be sure bevy and blender are using the same indices for the vertices.
# '''


# import bpy
# import bmesh

# def add_vertex_index_attribute():
#     # Get the active object
#     obj = bpy.context.active_object

#     # Check if the object exists and is a mesh
#     if obj is None or obj.type != 'MESH':
#         print("Please select a mesh object")
#         return

#     # Create a BMesh from the object data
#     bm = bmesh.new()
#     bm.from_mesh(obj.data)

#     # Ensure lookup table is up-to-date
#     bm.verts.ensure_lookup_table()

#     # Create a new custom attribute layer
#     index_layer = bm.verts.layers.int.new('_INDEX')

#     # Iterate through all vertices and assign unique IDs
#     for idx, v in enumerate(bm.verts):
#         v[index_layer] = idx

#     # Update the mesh with BMesh data
#     bm.to_mesh(obj.data)
#     obj.data.update()

#     # Free the BMesh
#     bm.free()

#     print(f"Added '_INDEX' attribute to {len(obj.data.vertices)} vertices")

# # Run the function
# add_vertex_index_attribute()
# 


############################################################################################################


# import bpy
# import bmesh

# def add_vertex_index_attribute(obj):
#     # Check if the object is a mesh
#     if obj.type != 'MESH':
#         print(f"Skipping {obj.name}: Not a mesh object")
#         return

#     # Create a BMesh from the object data
#     bm = bmesh.new()
#     bm.from_mesh(obj.data)

#     # Ensure lookup table is up-to-date
#     bm.verts.ensure_lookup_table()

#     # Create a new custom attribute layer
#     index_layer = bm.verts.layers.int.new('_INDEX')

#     # Iterate through all vertices and assign unique IDs
#     for idx, v in enumerate(bm.verts):
#         v[index_layer] = idx

#     # Update the mesh with BMesh data
#     bm.to_mesh(obj.data)
#     obj.data.update()

#     # Free the BMesh
#     bm.free()

#     print(f"Added '_INDEX' attribute to {len(obj.data.vertices)} vertices in {obj.name}")

# def process_selected_objects():
#     # Get all selected objects
#     selected_objects = bpy.context.selected_objects

#     # Check if any objects are selected
#     if not selected_objects:
#         print("No objects selected. Please select at least one mesh object.")
#         return

#     # Process each selected object
#     for obj in selected_objects:
#         add_vertex_index_attribute(obj)

# # Run the function
# process_selected_objects()

############################################################################################################

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