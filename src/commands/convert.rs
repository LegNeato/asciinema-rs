use alacritty;
use alacritty::cli;
use alacritty::config::Config;
use alacritty::display::{Display, DisplayCommand, InitialSize};
use alacritty::event;
use alacritty::event::Notify;

use alacritty::ansi::Handler;
use alacritty::event_loop::{self, EventLoop, Msg, WindowNotifier};
use alacritty::sync::FairMutex;
use alacritty::term::{SizeInfo, Term};
use alacritty::tty::{self, process_should_exit, Pty};
use asciicast::{Entry, Header};
use color_quant::NeuQuant;
use commands::concatenate::get_file;
use failure::Error;
use gif::{Encoder, Frame};
use gl;
use gl::types::*;
use glutin;
use glutin::{Api, GlContext, GlProfile, GlRequest, MouseButton, MouseScrollDelta, VirtualKeyCode,
             WindowEvent};
use image::{imageops, DynamicImage, ImageBuffer, ImageFormat, RgbaImage};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json;
use session::clock::get_elapsed_seconds;
use settings::ConvertSettings;
use settings::PlaySettings;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, StdoutLock, Write};
use std::os::raw::c_void;
use std::path::PathBuf;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{thread, time};
use tempfile::{self, NamedTempFile};

fn neuquant_palettize(width: u16, height: u16, pixels: &mut [u8], sample_rate: u16) -> NeuQuant {
    let image_len: u64 =
        (width as u64 * height as u64 * 4 / sample_rate as u64 / sample_rate as u64) as u64;
    let width = width as usize;
    let sample_rate = sample_rate as usize;
    let transparent_black = [0u8; 4];

    let mut temp: Vec<_> = Vec::with_capacity(image_len as usize);
    let mut n = 0;
    for px in pixels.chunks_mut(4) {
        n = n + 1;
        if sample_rate > 1 {
            if n % sample_rate != 0 || (n / width) % sample_rate != 0 {
                continue;
            }
        }
        if px[3] == 0 {
            temp.extend_from_slice(&transparent_black);
        } else {
            temp.extend_from_slice(&px[..3]);
            temp.push(255);
        }
    }

    let time_quant = Instant::now();
    let quant = NeuQuant::new(10, 256, &temp);
    quant
}

fn frame_from_rgba(width: u16, height: u16, pixels: &mut [u8], delay: u16) -> Frame<'static> {
    assert_eq!(width as usize * height as usize * 4, pixels.len());
    let mut frame = Frame::default();
    let mut transparent = None;
    for pix in pixels.chunks_mut(4) {
        if pix[3] != 0 {
            pix[3] = 0xFF;
        } else {
            transparent = Some([pix[0], pix[1], pix[2], pix[3]])
        }
    }
    frame.width = width;
    frame.height = height;
    frame.delay = delay;
    // A higher number here means we sample fewer pixels, which is faster but produces less
    // accurate colors.
    let nq = neuquant_palettize(width, height, pixels, 4);
    frame.buffer = Cow::Owned(pixels.chunks(4).map(|pix| nq.index_of(pix) as u8).collect());
    frame.palette = Some(nq.color_map_rgb());
    frame.transparent = if let Some(t) = transparent {
        Some(nq.index_of(&t) as u8)
    } else {
        None
    };
    frame
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
    let header = res?;

    // TODO: Do not hardcode.
    let hardcoded_width = 564;
    let hardcoded_height = 340;

    let window = glutin::HeadlessRendererBuilder::new(hardcoded_width, hardcoded_height)
        .build()
        .expect("create headless renderer");

    unsafe { window.make_current().expect("Couldn't make window current") };

    //gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    alacritty::gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let mut framebuffer = Framebuffer::new(hardcoded_width, hardcoded_height);
    framebuffer.bind();

    let config = Config::default();
    let mut options = cli::Options::default();
    options.print_events = true;

    let mut display =
        Display::new(&config, InitialSize::Cells(config.dimensions()), 1.0).expect("Display::new");

    let size = display.size().clone();

    let terminal = Term::new(&config, display.size().to_owned());
    let terminal = Arc::new(FairMutex::new(terminal));

    let pty = tty::new(&config, &options, &display.size(), None);

    let mut parser = alacritty::ansi::Processor::new();

    let p = PathBuf::from("/tmp/output.gif");

    let mut file = File::create(&p)?;
    let mut encoder = Encoder::new(
        file,
        size.width.clone() as u16,
        size.height.clone() as u16,
        &[],
    )?;

    let lines: Vec<String> = reader
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect();
    let entry_count = lines.len();
    let pb = ProgressBar::new(lines.len() as u64);
    let mut previous_time = 0.0;

    for (n, line) in lines.iter().enumerate() {
        let entry: Entry = serde_json::from_str(line.as_str())?;

        let data = entry.event_data.clone();
        for byte in data.as_bytes() {
            let mut locked_terminal = terminal.lock();
            parser.advance(&mut *locked_terminal, *byte, &mut pty.reader());
        }

        // Clamp to max ~100fps (~.01s).
        let seconds_delta = (entry.time.clone() - previous_time.clone());
        if n > 0 && // We always render the first frame.
           n < entry_count-1 && // We always render the last frame.
           seconds_delta < 0.1
        {
            pb.inc(1);
            continue;
        }

        let time_in_ms = (entry.time.clone() - previous_time.clone()) * 1000_f64; // enrty time is in seconds.
        let gif_frame_delay: u16 = if time_in_ms > 9.0 {
            (time_in_ms / 10 as f64) as u16
        } else {
            0
        }; // Delay is in 10s of ms.

        let mut locked_terminal = terminal.lock();
        locked_terminal.dirty = true;
        if locked_terminal.needs_draw() {
            display.draw(locked_terminal, &config, None, true);
        }
        let mut img: RgbaImage =
            ImageBuffer::new(size.width.clone() as u32, size.height.clone() as u32);

        unsafe {
            alacritty::gl::PixelStorei(alacritty::gl::PACK_ALIGNMENT, 1);
            alacritty::gl::ReadPixels(
                0,
                0,
                size.width.clone() as i32,
                size.height.clone() as i32,
                alacritty::gl::RGBA,
                alacritty::gl::UNSIGNED_BYTE,
                img.as_mut_ptr() as *mut GLvoid,
            );
        }

        let img = imageops::flip_vertical(&img);

        let frame = frame_from_rgba(
            size.width.clone() as u16,
            size.height.clone() as u16,
            &mut img.into_vec(),
            gif_frame_delay,
        );

        encoder.write_frame(&frame)?;

        previous_time = entry.time.clone();
        pb.inc(1);
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
            /*
            alacritty::gl::ClearColor(0.0, 1.0, 0.0, 1.0);
            alacritty::gl::Clear(gl::COLOR_BUFFER_BIT);
            */
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
