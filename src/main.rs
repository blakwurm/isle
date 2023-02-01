use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::device::{Device, DeviceCreateInfo, Features, QueueCreateInfo};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

fn main() {
  let library = VulkanLibrary::new().expect("no local Vulkan library installed");
  let instance = Instance::new(library, InstanceCreateInfo::default()).expect("failed to create instance");

  let physical = instance
    .enumerate_physical_devices()
    .expect("could not enumerate devices")
    .next()
    .expect("no devices available");

  let queue_family_index = physical
    .queue_family_properties()
    .iter()
    .enumerate()
    .position(|(_, q)| q.queue_flags.graphics)
    .expect("couldn't find a graphical queue family") as u32;

  let (device, mut queues) = Device::new(
    physical,
    DeviceCreateInfo { 
      queue_create_infos: vec![QueueCreateInfo {
        queue_family_index,
        ..Default::default()
      }],
      ..Default::default()
    }
  )
  .expect("failed to create device");

  let queue = queues.next().unwrap();

  let source_content: Vec<i32> = (0..64).collect();
  let source = CpuAccessibleBuffer::from_iter(
    device.clone(),
    BufferUsage {
      transfer_src: true,
      ..Default::default()
    },
    false,
    source_content,
  )
  .expect("failed to create source buffer");
  
  let destination_content: Vec<i32> = (0..64).map(|_| 0).collect();
  let destination = CpuAccessibleBuffer::from_iter(
    device.clone(),
    BufferUsage {
      transfer_dst: true,
      ..Default::default()
    },
    false,
    destination_content,
  )
  .expect("failed to create destination buffer");
}
