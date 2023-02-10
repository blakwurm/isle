use std::{collections::HashMap, error::Error, sync::Arc};

use bytemuck::{Pod, Zeroable};
use rand::{thread_rng, Rng};
use vulkano::{
  buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
  command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage,
    PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents,
  },
  device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo},
  image::{view::ImageView, ImageUsage, SwapchainImage},
  instance::{
    debug::{DebugUtilsMessenger, DebugUtilsMessengerCreateInfo},
    Instance, InstanceCreateInfo,
  },
  pipeline::{
    graphics::{
      input_assembly::InputAssemblyState,
      vertex_input::BuffersDefinition,
      viewport::{Viewport, ViewportState},
    },
    GraphicsPipeline,
  },
  render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
  shader::ShaderModule,
  swapchain::{
    self, AcquireError, PresentFuture, PresentInfo, Surface, Swapchain, SwapchainAcquireFuture,
    SwapchainCreateInfo, SwapchainCreationError,
  },
  sync::{self, FenceSignalFuture, FlushError, GpuFuture, JoinFuture},
  VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  platform::run_return::EventLoopExtRunReturn,
  window::{Window, WindowBuilder},
};

mod vs {
  vulkano_shaders::shader! {
    ty: "vertex",
    src: "
    #version 450
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec4 color;

    layout(location = 0) out vec4 out_color;

    // layout(set = 0, binding = 0) uniform MVP {
    //   mat4 model;
    //   mat4 view;
    //   mat4 proj;
    // } mvp;

    void main() {
      // gl_Position = mvp.proj * mvp.view * mvp.model * vec4(position, 1.0);
      gl_Position = vec4(position, 1.0);
      out_color = color;
    }
    "
  }
}

mod fs {
  vulkano_shaders::shader! {
    ty: "fragment",
    src: "
    #version 450
    layout(location = 0) in vec4 in_color;

    layout(location = 0) out vec4 f_color;

    void main() {
      f_color = in_color;
    }
    "
  }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
pub struct Vertex {
  pub position: [f32; 3],
  pub color: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, color);

struct Actor {
  name: String,
  buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
  tri_count: u32,
  translation: [f32; 3],
  scale: [f32; 3],
  rotation: [f32; 4],
}

pub struct VulkanBackend {
  _debug: Option<DebugUtilsMessenger>,
  surface: Arc<Surface<Window>>,
  device: Arc<Device>,
  queue: Arc<Queue>,
  event_loop: EventLoop<()>,
  render_pass: Arc<RenderPass>,
  framebuffers: Vec<Arc<Framebuffer>>,
  swapchain: Arc<Swapchain<Window>>,
  swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
  vertex_shader: Arc<ShaderModule>,
  fragment_shader: Arc<ShaderModule>,
  viewport: Viewport,
  actors: HashMap<String, Actor>,
  pipeline: Arc<GraphicsPipeline>,
  window_resized: bool,
  recreate_swapchain: bool,
  fences: Vec<
    Option<
      Arc<
        FenceSignalFuture<
          PresentFuture<
            CommandBufferExecFuture<
              JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture<Window>>,
              Arc<PrimaryAutoCommandBuffer>,
            >,
            Window,
          >,
        >,
      >,
    >,
  >,
  previous_fence_i: usize,
}

impl VulkanBackend {
  pub fn new() -> Result<Self, Box<dyn Error>> {
    let library = VulkanLibrary::new()?;
    let required_extensions = vulkano_win::required_extensions(&library);
    let instance = Instance::new(
      library,
      InstanceCreateInfo {
        enabled_extensions: required_extensions,
        enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_string()],
        ..Default::default()
      },
    )?;

    let _debug = unsafe {
      DebugUtilsMessenger::new(
        instance.clone(),
        DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
          println!("Vulkan: {:?}", msg.description);
        })),
      )
      .ok()
    };

    let physical = match instance.enumerate_physical_devices()?.next() {
      Some(physical) => physical,
      None => return Err("no device available".into()),
    };

    let queue_family_index = match physical
      .queue_family_properties()
      .iter()
      .enumerate()
      .position(|(_, q)| q.queue_flags.graphics)
    {
      Some(index) => index,
      None => return Err("couldn't find a graphical queue family".into()),
    } as u32;

    let device_extensions = DeviceExtensions {
      khr_swapchain: true,
      ..DeviceExtensions::empty()
    };

    let (device, mut queues) = Device::new(
      physical.clone(),
      DeviceCreateInfo {
        queue_create_infos: vec![QueueCreateInfo {
          queue_family_index,
          ..Default::default()
        }],
        enabled_extensions: device_extensions,
        ..Default::default()
      },
    )?;

    let queue = match queues.next() {
      Some(queue) => queue,
      None => return Err("no queue available".into()),
    };

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone())?;

    let capabilities = physical.surface_capabilities(&surface, Default::default())?;

    let dimensions = surface.window().inner_size();
    let composite_alpha = capabilities
      .supported_composite_alpha
      .iter()
      .next()
      .unwrap();
    let image_format = Some(physical.surface_formats(&surface, Default::default())?[0].0);

    let (swapchain, swapchain_images) = Swapchain::new(
      device.clone(),
      surface.clone(),
      SwapchainCreateInfo {
        min_image_count: capabilities.min_image_count + 1,
        image_format,
        image_extent: dimensions.into(),
        image_usage: ImageUsage {
          color_attachment: true,
          ..Default::default()
        },
        composite_alpha,
        ..Default::default()
      },
    )?;

    let render_pass = get_render_pass(device.clone(), &swapchain);
    let framebuffers = get_framebuffers(&swapchain_images, &render_pass);

    let vertex_shader = vs::load(device.clone())?;
    let fragment_shader = fs::load(device.clone())?;

    let viewport = Viewport {
      origin: [0.0, 0.0],
      dimensions: surface.window().inner_size().into(),
      depth_range: 0.0..1.0,
    };

    let pipeline = get_pipeline(
      device.clone(),
      vertex_shader.clone(),
      fragment_shader.clone(),
      render_pass.clone(),
      viewport.clone(),
    );

    let frames_in_flight = swapchain_images.len();

    Ok(Self {
      _debug,
      surface,
      device,
      queue,
      event_loop,
      render_pass,
      framebuffers,
      swapchain,
      swapchain_images,
      vertex_shader,
      fragment_shader,
      viewport,
      actors: HashMap::new(),
      pipeline,
      window_resized: false,
      recreate_swapchain: false,
      fences: vec![None; frames_in_flight],
      previous_fence_i: 0,
    })
  }

  pub fn create_actor(&mut self, name: Option<String>) {
    let name = match name {
      Some(name) => name,
      None => gen_id(None),
    };

    self.actors.insert(
      name.clone(),
      Actor {
        name,
        buffer: None,
        tri_count: 0,
        translation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
      },
    );
  }

  pub fn upload_model(&mut self, actor: String, model: Vec<Vertex>) {
    let mut actor = self.actors.get_mut(&actor).unwrap();

    actor.tri_count = (model.len() / 3) as u32;

    let buffer = CpuAccessibleBuffer::from_iter(
      self.device.clone(),
      BufferUsage {
        vertex_buffer: true,
        ..Default::default()
      },
      false,
      model.into_iter(),
    )
    .unwrap();

    actor.buffer = Some(buffer);
  }

  pub fn render(&mut self) -> bool {
    let mut close_requested = false;
    self
      .event_loop
      .run_return(|event, _, control_flow| match event {
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => {
          close_requested = true;
          *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
          event: WindowEvent::Resized(_),
          ..
        } => self.window_resized = true,
        Event::RedrawEventsCleared => {
          if self.window_resized || self.recreate_swapchain {
            self.recreate_swapchain = false;

            let dimensions = self.surface.window().inner_size();
            let (swapchain, swapchain_images) = match self.swapchain.recreate(SwapchainCreateInfo {
              image_extent: dimensions.into(),
              ..self.swapchain.create_info()
            }) {
              Ok(r) => r,
              Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
              Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

            self.swapchain = swapchain;
            self.framebuffers = get_framebuffers(&swapchain_images, &self.render_pass);
            self.swapchain_images = swapchain_images;

            if self.window_resized {
              self.window_resized = false;

              self.viewport.dimensions = dimensions.into();
              self.pipeline = get_pipeline(
                self.device.clone(),
                self.vertex_shader.clone(),
                self.fragment_shader.clone(),
                self.render_pass.clone(),
                self.viewport.clone(),
              );
            }
          }

          let (image_i, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
              Ok(r) => r,
              Err(AcquireError::OutOfDate) => {
                self.recreate_swapchain = true;
                return;
              }
              Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

          if suboptimal {
            self.recreate_swapchain = true;
          }

          if let Some(image_fence) = &self.fences[image_i] {
            image_fence.wait(None).unwrap();
          }

          let previous_future = match self.fences[self.previous_fence_i].clone() {
            None => {
              let mut now = sync::now(self.device.clone());
              now.cleanup_finished();

              now.boxed()
            }
            Some(fence) => fence.boxed(),
          };

          let command_buffer = get_command_buffers(
            self.device.clone(),
            self.queue.clone(),
            self.pipeline.clone(),
            self.framebuffers[image_i].clone(),
            &self.actors.values().collect::<Vec<_>>(),
          );

          let future = previous_future
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
              self.queue.clone(),
              PresentInfo {
                index: image_i,
                ..PresentInfo::swapchain(self.swapchain.clone())
              },
            )
            .then_signal_fence_and_flush();

          self.fences[image_i] = match future {
            Ok(value) => Some(Arc::new(value)),
            Err(FlushError::OutOfDate) => {
              self.recreate_swapchain = true;
              None
            }
            Err(e) => {
              println!("Failed to flush future: {:?}", e);
              None
            }
          };

          self.previous_fence_i = image_i;

          *control_flow = ControlFlow::Exit;
        }
        _ => (),
      });

    return close_requested;
  }
}

fn get_render_pass(device: Arc<Device>, swapchain: &Arc<Swapchain<Window>>) -> Arc<RenderPass> {
  vulkano::single_pass_renderpass!(
    device,
    attachments: {
      color: {
        load: Clear,
        store: Store,
        format: swapchain.image_format(),
        samples: 1,
      }
    },
    pass: {
      color: [color],
      depth_stencil: {}
    }
  )
  .unwrap()
}

fn get_framebuffers(
  images: &[Arc<SwapchainImage<Window>>],
  render_pass: &Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
  images
    .iter()
    .map(|image| {
      let view = ImageView::new_default(image.clone()).unwrap();
      Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
          attachments: vec![view],
          ..Default::default()
        },
      )
      .unwrap()
    })
    .collect::<Vec<_>>()
}

fn get_pipeline(
  device: Arc<Device>,
  vs: Arc<ShaderModule>,
  fs: Arc<ShaderModule>,
  render_pass: Arc<RenderPass>,
  viewport: Viewport,
) -> Arc<GraphicsPipeline> {
  GraphicsPipeline::start()
    .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
    .vertex_shader(vs.entry_point("main").unwrap(), ())
    .input_assembly_state(InputAssemblyState::new())
    .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
    .fragment_shader(fs.entry_point("main").unwrap(), ())
    .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
    .build(device.clone())
    .unwrap()
}

fn get_command_buffers(
  device: Arc<Device>,
  queue: Arc<Queue>,
  pipeline: Arc<GraphicsPipeline>,
  framebuffer: Arc<Framebuffer>,
  actors: &Vec<&Actor>,
) -> Arc<PrimaryAutoCommandBuffer> {
  let mut builder = AutoCommandBufferBuilder::primary(
    device,
    queue.queue_family_index(),
    CommandBufferUsage::MultipleSubmit,
  )
  .unwrap();

  builder
    .begin_render_pass(
      RenderPassBeginInfo {
        clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
        ..RenderPassBeginInfo::framebuffer(framebuffer)
      },
      SubpassContents::Inline,
    )
    .unwrap()
    .bind_pipeline_graphics(pipeline);
  // .bind_vertex_buffers(0, vertex_buffer.clone())
  // .draw(vertex_buffer.len() as u32, 1, 0, 0)

  add_actor_buffers(&mut builder, actors);

  builder.end_render_pass().unwrap();

  Arc::new(builder.build().unwrap())
}

fn add_actor_buffers(
  builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
  actors: &Vec<&Actor>,
) {
  for actor in actors {
    if let Some(buffer) = &actor.buffer {
      builder
        .bind_vertex_buffers(0, buffer.clone())
        .draw(buffer.len() as u32, actor.tri_count, 0, 0)
        .unwrap();
    }
  }
}

fn gen_id(length: Option<usize>) -> String {
  let length = length.unwrap_or(10);
  const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
  let mut rng = thread_rng();
  (0..length)
    .map(|_| {
      let idx = rng.gen_range(0..CHARSET.len());
      CHARSET[idx] as char
    })
    .collect()
}
