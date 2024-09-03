use bevy::prelude::*;
use bevy::window::WindowResized;

pub struct WindowResizePlugin;

impl Plugin for WindowResizePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, window_resize_system);
    }
}

fn window_resize_system(mut resize_events: EventReader<WindowResized>, mut windows: Query<&mut Window>) {
    for event in resize_events.read() {
        let aspect_ratio = 16.0 / 9.0; // You can adjust this to your desired aspect ratio
        let new_height = event.width / aspect_ratio;
        
        if let Ok(mut window) = windows.get_single_mut() {
            window.resolution.set(event.width, new_height);
        }
    }
}