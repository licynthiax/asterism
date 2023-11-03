#![allow(unused)]
pub use macroquad::{
    color::{colors::*, Color},
    math::{Rect, Vec2},
    text::TextParams,
    texture::DrawTextureParams,
};

use macroquad::{shapes::*, text, texture};

use futures::executor::block_on;
use std::collections::BTreeMap;

/// provided struct for organizing drawables
pub struct Draw<LogicsList> {
    pub positions: Vec<DrawType<LogicsList>>,
    pub drawables: Vec<Drawable>,
    textures: Textures,
    pub background_color: Color,
}

impl<LogicsList> Draw<LogicsList> {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            drawables: Vec::new(),
            textures: Textures::new(),
            background_color: BLANK,
        }
    }

    pub fn add_drawable(&mut self, at: usize, pos: DrawType<LogicsList>, drawable: Drawable) {
        self.positions.insert(at, pos);
        self.drawables.insert(at, drawable);
    }
    pub fn remove_drawable(&mut self, at: usize) {
        self.positions.remove(at);
        self.drawables.remove(at);
    }

    pub fn clear_drawables(&mut self) {
        self.positions.clear();
        self.drawables.clear();
    }

    pub fn draw(&mut self, positions: Vec<Vec2>) {
        macroquad::window::clear_background(self.background_color);
        for (drawable, pos) in self.drawables.iter_mut().zip(positions.iter()) {
            drawable.draw(*pos, &self.textures);
        }
    }
}

pub struct DrawRects<LogicsList> {
    positions: Vec<DrawType<LogicsList>>,
    drawables: Vec<Drawable>,
}

impl<LogicsList> DrawRects<LogicsList> {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            drawables: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.drawables.clear();
    }

    pub fn get_draw_types(&self) -> &[DrawType<LogicsList>] {
        &self.positions
    }

    pub fn draw(&self, positions: Vec<Vec2>) {
        for (drawable, pos) in self.drawables.iter().zip(positions.iter()) {
            drawable.draw_rect(*pos);
        }
    }
}

#[derive(Debug)]
pub enum Drawable {
    Rectangle(Vec2, Color),
    Sprite(Vec2, TexID),
    Animation(Vec2, Box<Animation>),
    Text(String, u16, Color),
}

impl Drawable {
    /// draw if you're not using textures
    pub fn draw_rect(&self, pos: Vec2) {
        if let Drawable::Rectangle(size, color) = self {
            draw_rectangle(pos.x, pos.y, size.x, size.y, *color);
        }
    }

    // making self mutable is probably not sensible? honestly unsure
    pub fn draw(&mut self, pos: Vec2, textures: &Textures) {
        match self {
            Drawable::Rectangle(size, color) => {
                draw_rectangle(pos.x, pos.y, size.x, size.y, *color)
            }
            Drawable::Sprite(size, sprite) => {
                let params = DrawTextureParams {
                    dest_size: Some(*size),
                    ..Default::default()
                };
                let tex = textures.get_texture(*sprite);
                texture::draw_texture_ex(tex, pos.x, pos.y, BLACK, params);
            }
            Drawable::Animation(size, anim) => {
                let params = DrawTextureParams {
                    dest_size: Some(*size),
                    ..Default::default()
                };
                let frame = anim.current_frame;
                let current_anim = &anim.animations[frame.0];
                let id = current_anim.frames[frame.1];
                let tex = textures.get_texture(id);

                // update animation frame
                if current_anim.loops {
                    anim.current_frame.1 = (anim.current_frame.1 + 1) % current_anim.frames.len();
                } else if anim.current_frame.1 < current_anim.frames.len() - 1 {
                    anim.current_frame.1 += 1;
                }

                texture::draw_texture_ex(tex, pos.x, pos.y, BLACK, params);
            }
            Drawable::Text(string, size, color) => {
                let params = TextParams {
                    font_size: *size,
                    color: *color,
                    ..Default::default()
                };
                let mut pos = pos;
                pos.y += *size as f32;
                text::draw_text_ex(string, pos.x, pos.y, params);
            }
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct TexID(u32);

pub enum DrawType<WhichLogic> {
    FromLogic(WhichLogic),
    Offset(WhichLogic, Vec2),
    FixedPoint(Vec2),
}

/// struct to store all textures in a game
pub struct Textures {
    tex: BTreeMap<TexID, texture::Texture2D>,
    count: u32,
}

impl Textures {
    pub fn new() -> Self {
        Self {
            tex: BTreeMap::new(),
            count: 0,
        }
    }

    // this doesn't quite feel right but i want to be able to remove textures
    pub fn add_texture(&mut self, path: &str) -> TexID {
        let tex = block_on(texture::load_texture(path)).unwrap();
        let id = TexID(self.count);
        self.tex.insert(id, tex);
        id
    }

    pub fn get_texture(&self, id: TexID) -> &texture::Texture2D {
        self.tex
            .get(&id)
            .unwrap_or_else(|| panic!("texture id {} doesn't match any loaded texture", id.0))
    }

    pub fn remove_texture(&mut self, id: TexID) {
        // i think... this should never panic actually?
        let tex = self
            .tex
            .remove(&id)
            .unwrap_or_else(|| panic!("texture id {} doesn't match any loaded texture", id.0));
    }
}

#[derive(Debug)]
pub struct Animation {
    rect: Rect,
    animations: Vec<AnimData>,
    current_frame: (usize, usize),
}

#[derive(Debug)]
struct AnimData {
    frames: Vec<TexID>,
    loops: bool,
}
