use crate::gui::Gui;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod gui;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const BOX_SIZE: i16 = 64;

mod cpu;
mod memory;
use std::{fs::File, io::Read, sync::{Arc, Mutex}};
use std::cell::{RefCell, RefMut};

struct Display {
    width: i16,
    height: i16,
    box_x: i16,
    box_y: i16,
}

fn main() -> Result<(), Error> {

    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Crab Boy")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_max_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut scale_factor = window.scale_factor();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut display = Display::new(WIDTH, HEIGHT);

    let mut cpu = create_cpu();
    let mut gui = Gui::new(&window, &pixels);
    let mut run = true;

    event_loop.run(move |event, _, control_flow| {
        if run{
            cpu.tick();
        }

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            // Draw the display
            display.draw(pixels.get_frame());

            // Prepare Dear ImGui
            gui.prepare(&window).expect("gui.prepare() failed");

            // Render everything together
            let render_result = pixels.render_with(|encoder, render_target, context| {
                // Render the world texture
                context.scaling_renderer.render(encoder, render_target);

                // Render Dear ImGui
                gui.render(&window, encoder, render_target, context, &mut cpu, &mut run)?;

                Ok(())
            });


            // Basic error handling
            if render_result
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        gui.handle_event(&window, &event);
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Update the scale factor
            if let Some(factor) = input.scale_factor() {
                scale_factor = factor;
            }

            // Update internal state and request a redraw
            display.update();
            window.request_redraw();
        }
    });



}


fn create_cpu() -> cpu::Cpu {
    let registers = cpu::Registers::new(); // * sets starting registers and opcode
    let mut main_memory = memory::Memory::new();


    let mut file=File::open("resources/game.gb").unwrap(); // ! dirty rom load, replace this when cartridge controller implemented
    let mut buf=[0u8;256_000];
    file.read(&mut buf).unwrap();
    for x in 0..0x8000 { // ! dirty rom into memeory merge, bad method only supports bios at the moment
        main_memory.memory[x] = buf[x];
    }

    let main_cpu = cpu::Cpu::new(registers,  main_memory);

    return main_cpu;
}


impl Display {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width: width as i16,
            height: height as i16,
            box_x: 0,
            box_y: 0,
        }
    }

    /// Update the `Display` internal state; bounce the box around the screen.
    fn update(&mut self) {
    }

    /// Draw the `Display` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % self.width as usize) as i16;
            let y = (i / self.width as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
