#![allow(unused)]
pub use macroquad::{
    color::{colors::*, Color},
    math::{Rect, Vec2},
    texture::DrawTextureParams,
};

use macroquad::{shapes::*, texture};

use futures::executor::block_on;
use std::collections::BTreeMap;

/// provided struct for organizing drawables
pub struct Draw {
    pub drawables: Vec<Drawable>,
    pub textures: Textures,
    pub background_color: Color,
}

impl Draw {
    pub fn new() -> Self {
        Self {
            drawables: Vec::new(),
            textures: Textures::new(),
            background_color: BLANK,
        }
    }

    pub fn clear_drawables(&mut self) {
        self.drawables.clear();
    }

    pub fn update_rect(&mut self, i: usize, rect: Rect) {
        match &mut self.drawables[i] {
            Drawable::Rectangle(r, _) => *r = rect,
            Drawable::Sprite(r, _) => *r = rect,
            Drawable::Animation(r, _) => *r = rect,
        }
    }

    pub fn draw(&mut self) {
        macroquad::window::clear_background(self.background_color);
        for drawable in self.drawables.iter_mut() {
            drawable.draw(&self.textures);
        }
    }
}

pub struct DrawRects {
    pub drawables: Vec<Drawable>,
}

impl DrawRects {
    pub fn new() -> Self {
        Self {
            drawables: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.drawables.clear();
    }

    pub fn draw(&self) {
        for drawable in self.drawables.iter() {
            drawable.draw_rect();
        }
    }
}

#[derive(Debug)]
pub enum Drawable {
    /// position + size, then color
    Rectangle(Rect, Color),
    /// position + size, then texture
    Sprite(Rect, TexID),
    /// position + size, then animation information
    Animation(Rect, Box<Animation>),
}

impl Drawable {
    /// draw if you're not using textures
    pub fn draw_rect(&self) {
        if let Drawable::Rectangle(rect, color) = self {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, *color);
        }
    }

    // making self mutable is probably not sensible? honestly unsure
    pub fn draw(&mut self, textures: &Textures) {
        match self {
            Drawable::Rectangle(rect, color) => {
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, *color)
            }
            Drawable::Sprite(rect, sprite) => {
                let params = DrawTextureParams {
                    dest_size: Some(Vec2::new(rect.w, rect.h)),
                    ..Default::default()
                };
                let tex = textures.get_texture(*sprite);
                texture::draw_texture_ex(tex, rect.x, rect.y, BLACK, params);
            }
            Drawable::Animation(rect, anim) => {
                let params = DrawTextureParams {
                    dest_size: Some(Vec2::new(rect.w, rect.h)),
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

                texture::draw_texture_ex(tex, rect.x, rect.y, BLACK, params);
            }
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct TexID(u32);

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
