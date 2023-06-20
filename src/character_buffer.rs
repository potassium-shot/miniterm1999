use std::ops::Range;

use crate::{character::Character, globals};

pub struct CharacterBuffer {
    characters: Vec<Character>,
    pub bounds: (u32, u32),
    pub cursor_position: (u32, u32),

    pending_change: Option<Range<usize>>,
}

impl CharacterBuffer {
    pub fn new(bounds: (u32, u32)) -> Self {
        Self {
            characters: vec![
                Character::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0);
                globals::CHARACTER_BUFFER_SIZE
            ],
            bounds,
            cursor_position: (0, 0),

            pending_change: None,
        }
    }

    pub fn push_char(&mut self, c: Character) {
        let index = Self::vec_coord(self.cursor_position);
        self.characters[index] = c;
        self.cursor_position.0 += 1;

        if self.cursor_position.0 >= self.bounds.0 {
            self.cursor_position.1 += 1;
            self.cursor_position.0 = 0;
        }

        match &mut self.pending_change {
            Some(range) => {
                if index >= range.end {
                    range.end = index + 1;
                } else if index < range.start {
                    range.start = index;
                }
            }
            None => self.pending_change = Some(index..(index + 1)),
        }
    }

    pub fn write_changes(&self, queue: &wgpu::Queue, buffer: &wgpu::Buffer) {
        if let Some(range) = &self.pending_change {
            queue.write_buffer(
                buffer,
                range.start as u64,
                bytemuck::cast_slice(&self.characters[range.clone()]),
            );
        }
    }

    pub fn buffer(&self) -> &[Character] {
        &self.characters
    }

    fn vec_coord(pos: (u32, u32)) -> usize {
        (pos.1 * (globals::CHARACTER_BUFFER_WIDTH as u32) + pos.0) as usize
    }
}
