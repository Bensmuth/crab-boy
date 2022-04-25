use glib::{clone, Continue, MainContext, PRIORITY_DEFAULT};
use gtk::glib;
// Should perhaps switch to an easier to understand gui library like https://relm4.org/
//
//
// Check out
// https://gtk-rs.org/gtk-rs-core/git/docs/gdk_pixbuf/struct.Pixbuf.html for
// drawing the screen.

//use gtk::{Application, Button, Label, ListBox, Orientation};
//use gtk::prelude::*;

//use adw::prelude::*;
//use adw::{ActionRow, ApplicationWindow, HeaderBar}

use adw::prelude::*;

use adw::{ApplicationWindow, HeaderBar};
use gtk::{Application, Box, ListBox, Orientation, Button, Label};

mod cpu;
mod memory;
use std::io::Write;
use std::thread;
use std::{fs::File, io::Read, sync::{Arc, Mutex}};

fn main() {
    // Create a new application
    let app = Application::builder()
        .application_id("org.bezmuth.crab-boy")
        .build();

    app.connect_startup(|_| {
        adw::init();
    });

    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

fn build_ui(app: &Application) {
    /*
    Create cpu. Just following the gtk docs here, i dont really know
    how reference counting and cells work in rust
    */
    let cpu = Arc::new(Mutex::new(create_cpu()));

    // Create ui elements
    let list = ListBox::builder()
        .margin_top(0)
        .margin_end(0)
        .margin_bottom(0)
        .margin_start(0)
         // the content class makes the list look nicer
        .css_classes(vec![String::from("content")])
        .build();
    let tick_button = Button::builder()
        .label("Step")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let dump_button = Button::builder()
        .label("Dump mem")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let go_button = Button::builder()
        .label("Go")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let label = Label::builder()
        .label("PC: unint")
        .build();

    list.append(&label);
    list.append(&tick_button);
    list.append(&dump_button);
    list.append(&go_button);

    // Combine the content in a box
    let content = Box::new(Orientation::Vertical, 0);
    // Adwaitas' ApplicationWindow does not include a HeaderBar
    content.append(
        &HeaderBar::builder()
            .title_widget(&adw::WindowTitle::new("Crab Boy", ""))
            .build(),
    );
    content.append(&list);


    let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
    sender.send(cpu.lock().unwrap().get_register_debug_string()).ok();
    
    /*
    Connect callback, look this is jank as shit, but it works so
    meh. Basically im mixing oop and functional programming (kinda)
    which isnt exactly the best way to do things, tbh its what i get
    for not planning around the gui. For now this will remain single
    threaded, see
    https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html
    for when you wanna fix that
     */
    tick_button.connect_clicked(
        clone!(@strong cpu, @strong sender => move |_| { // TODO read the move and clone! docs
            let mut cpu = cpu.lock().unwrap();
            cpu.tick();
            let register_dump = cpu.get_register_debug_string();
        
            sender.send(register_dump).ok();
        }
    ));

    go_button.connect_clicked(
        clone!(@strong cpu, @strong sender => move |_| {
            thread::spawn( // okay this is threaded now, might be a better way to do this
                clone!(@strong cpu, @strong sender => move || {
                    for _ in 0..100000 {
                        let mut cpu = cpu.lock().unwrap();
                        cpu.tick();
                        let register_dump = cpu.get_register_debug_string();

                        sender.send(register_dump).ok();
                    }
                }
            ));
        }
    ));


    receiver.attach(
        None,
        clone!(@weak label => @default-return Continue(false),
               move |receiver_data| { // OKAY SO APPARENTLY THIS IS WHERE THE DATA FROM THE SEND THING GETS PASSED, I HAD NO IDEA
                   label.set_text(&receiver_data); 
                   Continue(true)
               }
        ),
    );

    dump_button.connect_clicked(move |_| {
        let memory_dump = cpu.lock().unwrap().get_memory_debug();
        let mut file = File::create("dump").unwrap();
        file.write_all(&memory_dump.memory).unwrap();
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(350)
         // add content to window
        .content(&content)
        .build();
    window.show();
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
