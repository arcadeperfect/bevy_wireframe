import bpy
import bmesh

def remove_vertex_attribute(attribute_name):
    selected_objects = bpy.context.selected_objects

    if not selected_objects:
        print("No objects selected. Please select at least one object.")
        return

    for obj in selected_objects:
        if obj.type != 'MESH':
            print(f"Skipping {obj.name}: Not a mesh object")
            continue

        # Create a BMesh from the object data
        bm = bmesh.new()
        bm.from_mesh(obj.data)

        # Ensure lookup table is up-to-date
        bm.verts.ensure_lookup_table()

        # Check if the attribute exists
        if attribute_name in bm.verts.layers.int:
            bm.verts.layers.int.remove(bm.verts.layers.int[attribute_name])
            print(f"Removed '{attribute_name}' attribute from {obj.name}")
        elif attribute_name in bm.verts.layers.float:
            bm.verts.layers.float.remove(bm.verts.layers.float[attribute_name])
            print(f"Removed '{attribute_name}' attribute from {obj.name}")
        elif attribute_name in bm.verts.layers.string:
            bm.verts.layers.string.remove(bm.verts.layers.string[attribute_name])
            print(f"Removed '{attribute_name}' attribute from {obj.name}")
        else:
            print(f"Attribute '{attribute_name}' not found in {obj.name}")

        # Update the mesh with BMesh data
        bm.to_mesh(obj.data)
        obj.data.update()

        # Free the BMesh
        bm.free()

# Example usage:
# Change '_VERT_INDEX' to the name of the attribute you want to remove
attribute_to_remove = '_INDEX'
remove_vertex_attribute(attribute_to_remove)