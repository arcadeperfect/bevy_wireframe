import bpy
import bmesh

def add_vertex_index_attribute():
    # Get the active object
    obj = bpy.context.active_object

    # Check if the object exists and is a mesh
    if obj is None or obj.type != 'MESH':
        print("Please select a mesh object")
        return

    # Create a BMesh from the object data
    bm = bmesh.new()
    bm.from_mesh(obj.data)

    # Ensure lookup table is up-to-date
    bm.verts.ensure_lookup_table()

    # Create a new custom attribute layer
    index_layer = bm.verts.layers.int.new('_INDEX')

    # Iterate through all vertices and assign unique IDs
    for idx, v in enumerate(bm.verts):
        v[index_layer] = idx

    # Update the mesh with BMesh data
    bm.to_mesh(obj.data)
    obj.data.update()

    # Free the BMesh
    bm.free()

    print(f"Added '_INDEX' attribute to {len(obj.data.vertices)} vertices")

# Run the function
add_vertex_index_attribute()