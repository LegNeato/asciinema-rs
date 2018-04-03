use alacritty;
use alacritty::cli;
use alacritty::config::Config;
use alacritty::display::{Display, DisplayCommand, InitialSize};
use alacritty::event::Notify;
use alacritty::event_loop::{self, EventLoop, WindowNotifier};
use alacritty::sync::FairMutex;
use alacritty::term::{SizeInfo, Term};
use alacritty::tty::{self, Pty};
use asciicast::{Entry, Header};
use commands::concatenate::get_file;
use gl;
use gl::types::*;
use glutin;
use glutin::{Api, GlContext, GlProfile, GlRequest, MouseButton, MouseScrollDelta, VirtualKeyCode,
             WindowEvent};
use image::{DynamicImage, ImageFormat};
use serde_json;
use session::clock::get_elapsed_seconds;
use settings::PlaySettings;
use std::borrow::Cow;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, StdoutLock, Write};
use std::os::raw::c_void;
use std::time::Instant;
use tempfile::NamedTempFile;

use failure::Error;
use settings::ConvertSettings;
use std::cell::RefCell;
use std::path::PathBuf;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

pub struct IsControlHeld(bool);

pub enum Event {
    Blank,
    CharInput(char, IsControlHeld),
    StringInput(String),
    StrInput(&'static str),
    WindowResized(u32, u32),
    HiDPIFactorChanged(f32),
    ChangeFontSize(i8),
    ResetFontSize,
}

struct Notifier;

impl WindowNotifier for Notifier {
    fn notify(&self) {
        // TODO: redraw?
    }
}

pub struct State {
    config: Config,
    display: Display,
    terminal: Arc<FairMutex<Term>>,
    pty: Pty,
    loop_notifier: event_loop::Notifier,
    pub event_queue: Vec<Event>,
}

pub fn go(settings: &ConvertSettings) -> Result<PathBuf, Error> {
    let location = settings.location.clone();
    let mut temp: NamedTempFile = NamedTempFile::new()?;
    let file = get_file(location, &mut temp)?;

    let mut reader = BufReader::new(file);
    let mut line = String::new();

    // Skip the first line, and maybe Header is needed later.
    let _len = reader.read_line(&mut line);
    let res: Result<Header, serde_json::Error> = serde_json::from_str(line.as_str());
    let header = res.unwrap();

    let gl_request = GlRequest::Specific(Api::OpenGl, (3, 3));
    let gl_profile = GlProfile::Core;
    let window = glutin::HeadlessRendererBuilder::new(header.width.clone(), header.height.clone())
        //.with_gl(gl_request)
        //.with_gl_profile(gl_profile)
        .build()
        .unwrap();

    unsafe { window.make_current().expect("Couldn't make window current") };

    //gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    alacritty::gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    /*
    unsafe {
            let mut framebuffer = 0;
            let mut texture = 0;
            gl.GenFramebuffers(1, &mut framebuffer);
            gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            gl.GenTextures(1, &mut texture);
            gl.BindTexture(gl::TEXTURE_2D, texture);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl.TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, header.width.clone(), header.height.clone(),
                         0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());
            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0);
            let status = gl.CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
              panic!("Error while creating the framebuffer");
            }
        }
        */
    {
        let mut framebuffer = Framebuffer::new(header.width.clone(), header.height.clone());
        framebuffer.bind();
    }

    /*
    unsafe {
        gl::Viewport(
            0,
            0,
            header.width.clone() as i32,
            header.height.clone() as i32,
        );
    }
*/

    let config = Config::default();
    let mut options = cli::Options::default();
    options.print_events = true;

    let mut display =
        Display::new(&config, InitialSize::Cells(config.dimensions()), 1.0).expect("Display::new");

    let size = display.size().clone();
    println!("SIZE: {:?}", size.clone());

    //framebuffer.unbind();
    //let framebuffer2 = Framebuffer::new(size.width.clone() as u32, size.height.clone() as u32);
    //framebuffer2.bind();

    let terminal = Term::new(&config, display.size().to_owned());
    let terminal = Arc::new(FairMutex::new(terminal));

    let pty = tty::new(&config, &options, &display.size(), None);

    let event_loop = EventLoop::new(
        Arc::clone(&terminal),
        Box::new(Notifier),
        pty.reader(),
        options.ref_test,
    );

    let mut loop_notifier = event_loop::Notifier(event_loop.channel());
    let _io_thread = event_loop.spawn(None);
    /*
    let mut event_queue = Vec::new();
    //let mut data;

    let base = Instant::now();
    for line in reader.lines() {
        let entry: Entry = serde_json::from_str(line.unwrap().as_str())?;
        loop {
            if entry.time <= get_elapsed_seconds(&base.elapsed()) {
                let data = entry.event_data.clone();
                event_queue.push(Box::new(data.clone().to_owned()));
                break;
            }
        }
    }

    for data in event_queue {
        loop_notifier.notify(data.as_bytes());
    }
    */
    let mut locked_terminal = terminal.lock();
    display.draw(locked_terminal, &config, None, true);

    let mut img = DynamicImage::new_rgba8(header.width.clone(), header.height.clone());
    unsafe {
        let pixels = img.as_mut_rgba8().unwrap();
        alacritty::gl::PixelStorei(alacritty::gl::PACK_ALIGNMENT, 1);
        alacritty::gl::ReadPixels(
            0,
            0,
            size.width.clone() as i32,
            size.height.clone() as i32,
            alacritty::gl::RGBA,
            alacritty::gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as *mut c_void,
        );
        // gl_check_error!();
    }

    let img = img.flipv();

    let p = PathBuf::from("/tmp/output.png");

    let mut file = File::create(&p).unwrap();
    if let Err(err) = img.save(&mut file, ImageFormat::PNG) {
        //error!("{}", err);
        unimplemented!();
    }
    Ok(p)
}

#[derive(Debug)]
pub struct Framebuffer {
    pub id: u32,
    pub texture_colorbuffer: u32,
    pub rbo: u32,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Framebuffer {
        let mut framebuffer = 0;
        let mut texture_colorbuffer = 0;
        let mut rbo = 0;

        unsafe {
            alacritty::gl::GenFramebuffers(1, &mut framebuffer);
            alacritty::gl::BindFramebuffer(alacritty::gl::FRAMEBUFFER, framebuffer);
            // create a color attachment texture
            alacritty::gl::GenTextures(1, &mut texture_colorbuffer);
            alacritty::gl::BindTexture(alacritty::gl::TEXTURE_2D, texture_colorbuffer);
            alacritty::gl::TexImage2D(
                alacritty::gl::TEXTURE_2D,
                0,
                alacritty::gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                alacritty::gl::RGBA,
                alacritty::gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            alacritty::gl::TexParameteri(
                alacritty::gl::TEXTURE_2D,
                alacritty::gl::TEXTURE_MIN_FILTER,
                alacritty::gl::LINEAR as i32,
            );
            alacritty::gl::TexParameteri(
                alacritty::gl::TEXTURE_2D,
                alacritty::gl::TEXTURE_MAG_FILTER,
                alacritty::gl::LINEAR as i32,
            );
            alacritty::gl::FramebufferTexture2D(
                alacritty::gl::FRAMEBUFFER,
                alacritty::gl::COLOR_ATTACHMENT0,
                alacritty::gl::TEXTURE_2D,
                texture_colorbuffer,
                0,
            );
            // create a renderbuffer object for depth and stencil attachment (we won't be sampling these)
            alacritty::gl::GenRenderbuffers(1, &mut rbo);
            alacritty::gl::BindRenderbuffer(alacritty::gl::RENDERBUFFER, rbo);
            alacritty::gl::RenderbufferStorage(
                alacritty::gl::RENDERBUFFER,
                alacritty::gl::DEPTH24_STENCIL8,
                width as i32,
                height as i32,
            ); // use a single renderbuffer object for both a depth AND stencil buffer.
            alacritty::gl::FramebufferRenderbuffer(
                alacritty::gl::FRAMEBUFFER,
                alacritty::gl::DEPTH_STENCIL_ATTACHMENT,
                alacritty::gl::RENDERBUFFER,
                rbo,
            ); // now actually attach it
               // now that we actually created the framebuffer and added all attachments we want to check if it is actually complete now
            if alacritty::gl::CheckFramebufferStatus(alacritty::gl::FRAMEBUFFER)
                != alacritty::gl::FRAMEBUFFER_COMPLETE
            {
                panic!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
            }
            alacritty::gl::BindFramebuffer(alacritty::gl::FRAMEBUFFER, 0);
        }

        Framebuffer {
            id: framebuffer,
            texture_colorbuffer,
            rbo,
        }
    }

    pub fn bind(&self) {
        unsafe { alacritty::gl::BindFramebuffer(alacritty::gl::FRAMEBUFFER, self.id) }
    }

    pub fn unbind(&mut self) {
        unsafe {
            alacritty::gl::DeleteTextures(1, &mut self.texture_colorbuffer);
            alacritty::gl::DeleteRenderbuffers(1, &mut self.rbo);
            alacritty::gl::DeleteFramebuffers(1, &mut self.id);

            //alacritty::gl::BindFramebuffer(alacritty::gl::FRAMEBUFFER, 0);
        }
    }
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
#[cfg_attr(feature = "clippy", allow(doc_markdown))]
#[cfg_attr(feature = "clippy", allow(unreadable_literal))]
#[allow(unused_mut)]
pub mod mygl {
    //pub use glutin::Gles2 as Gl;
    #![allow(non_upper_case_globals)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
