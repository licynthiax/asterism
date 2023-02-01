#![allow(unused)]
pub use macroquad::{color::*, math::*, shapes::*, texture::*};

use futures::executor::block_on;
use std::collections::BTreeMap;

pub enum Draw {
    /// position + size, then color
    Rectangle(Rect, Color),
    /// position + size, then texture
    Sprite(Rect, TexID),
    /// position + size, then animation information
    Animation(Rect, Box<Animation>),
}

impl Draw {
    /// draw if you're not using textures
    pub fn draw_rect(&self) {
        if let Draw::Rectangle(rect, color) = self {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, *color);
        }
    }

    // making self mutable is probably not sensible? honestly unsure
    pub fn draw(&mut self, textures: &Textures) {
        match self {
            Draw::Rectangle(rect, color) => draw_rectangle(rect.x, rect.y, rect.w, rect.h, *color),
            Draw::Sprite(rect, sprite) => {
                let params = DrawTextureParams {
                    dest_size: Some(Vec2::new(rect.w, rect.h)),
                    ..Default::default()
                };
                let tex = textures.get_texture(*sprite);
                draw_texture_ex(*tex, rect.x, rect.y, BLACK, params);
            }
            Draw::Animation(rect, anim) => {
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

                draw_texture_ex(*tex, rect.x, rect.y, BLACK, params);
            }
        }
    }
}

/// struct to store all textures in a game
pub struct Textures {
    tex: BTreeMap<TexID, Texture2D>,
    count: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct TexID(u32);

impl Textures {
    pub fn new() -> Self {
        Self {
            tex: BTreeMap::new(),
            count: 0,
        }
    }

    // this doesn't quite feel right but i want to be able to remove textures
    pub fn add_texture(&mut self, path: &str) -> TexID {
        let tex = block_on(load_texture(path)).unwrap();
        let id = TexID(self.count);
        self.tex.insert(id, tex);
        id
    }

    pub fn get_texture(&self, id: TexID) -> &Texture2D {
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
        tex.delete()
    }
}

pub struct Animation {
    rect: Rect,
    animations: Vec<AnimData>,
    current_frame: (usize, usize),
}

// the difference between animation
struct AnimData {
    frames: Vec<TexID>,
    loops: bool,
}
