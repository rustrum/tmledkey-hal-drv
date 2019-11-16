use super::*;

use alloc::vec::Vec;

const CIRCLE: [u8; 6] = [SEG_1, SEG_2, SEG_3, SEG_4, SEG_5, SEG_6];

pub trait Animation<R> {
    fn next(&mut self) -> Option<R>;
}

#[derive(Debug)]
pub struct Spinner {
    offset: u8,
    cw: bool,
    mask: [bool; 6],
}

impl Spinner {
    pub fn new(initial_mask: u8, clockwise: bool) -> Spinner {
        let mut init = [false; 6];

        for s in 0..CIRCLE.len() {
            if CIRCLE[s] & initial_mask != 0 {
                init[s] = true;
            }
        }

        Spinner {
            offset: 0,
            cw: clockwise,
            mask: init,
        }
    }
}

impl Animation<u8> for Spinner {
    fn next(&mut self) -> Option<u8> {
        let mut res = 0_u8;
        for i in 0..self.mask.len() {
            let v = self.mask[(i + self.offset as usize) % self.mask.len()];
            if v {
                res |= CIRCLE[v as usize];
            }
        }
        self.offset = (self.offset + 1) % self.mask.len() as u8;
        return Some(res);
    }
}

#[derive(Debug)]
pub enum SlideType {
    StopAtFirstChar,
    StopAfterLastChar,
    Cycle,
}

#[derive(Debug)]
pub struct Slide {
    tp: SlideType,
    count: u8,
    result_len: u8,
    word: Vec<u8>,
}

impl Slide {
    pub fn new(slide_type: SlideType, displays_count: u8, bytes_to_slide: &[u8]) -> Slide {
        let mut word = Vec::<u8>::new();

        for i in 0..bytes_to_slide.len() {
            word.push(bytes_to_slide[i]);
        }

        Slide {
            tp: slide_type,
            count: 0,
            result_len: displays_count,
            word: word,
        }
    }
}

impl Animation<Vec<u8>> for Slide {
    fn next(&mut self) -> Option<Vec<u8>> {
        if self.count == 255 {
            // Do not support long words
            return None;
        }

        let mut out = Vec::<u8>::new();
        let off_out = self.result_len as isize - self.count as isize;

        // Add blank offset before word
        if off_out > 0 {
            for _ in 0..off_out {
                out.push(0);
            }
        }

        let off_in = if off_out < 0 {
            (off_out * -1) as usize
        } else {
            0
        };

        if off_in > 0 {
            // Must stop if first char of word goes out of screen
            if let SlideType::StopAtFirstChar = self.tp {
                return None;
            }
        }

        let len_in = if off_in > self.result_len as usize {
            0
        } else {
            self.result_len as usize - off_in
        };

        if off_in >= self.word.len() {
            // Stop if last char already out of screen
            if let SlideType::StopAfterLastChar = self.tp {
                return None;
            }
        }

        if (off_in as usize) < self.word.len() {
            for i in 0..self.word.len() {
                if i < off_in || i >= off_in + len_in {
                    continue;
                }
                out.push(self.word[i]);
            }
        }

        // Add blank offset after word
        if out.len() < self.result_len as usize {
            for _ in 0..(self.result_len as usize - out.len()) {
                out.push(0);
            }
        }

        self.count += 1;
        if off_in + 1 >= self.word.len() {
            // Reset counter if last char out of screen and we do it in cycles
            if let SlideType::Cycle = self.tp {
                self.count = 0;
            }
        }

        return Some(out);
    }
}
