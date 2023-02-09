mod registry;
mod renderer;

use renderer::vulkan::{Vertex, VulkanBackend};
fn main() {
    let mut renderer = VulkanBackend::new().expect("Failed to create Vulkan backend");

    let vertices = vec![
        Vertex {
            position: [0.0, -0.575, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        Vertex {
            position: [-0.6, 0.575, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.6, 0.575, 0.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
    ];

    renderer.create_actor(Some(String::from("test_actor")));
    renderer.upload_model(String::from("test_actor"), vertices);

    loop {
        if renderer.render() {
            return;
        }
    }
}
