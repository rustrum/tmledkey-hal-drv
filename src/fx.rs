use super::*;

use alloc::vec::Vec;

const CIRCLE: [u8; 6] = [SEG_1, SEG_2, SEG_3, SEG_4, SEG_5, SEG_6];

///
/// Simple iterator-like animation functionality.
///
pub trait Animate<R> {
    /// Return next animation payload (value), that you should send to MCU somehow.
    /// Delays between calls and other stuff should be handled in your code.
    ///
    /// If animation completed returns `None` for infinite animations always returns something.
    fn next(&mut self) -> Option<R>;
}

/// Stands for spinning segments round and round.
/// Spinning path is the same as for ZERO digit.
#[derive(Debug)]
pub struct Spinner {
    offset: u8,
    cw: bool,
    mask: [bool; 6],
}

impl Spinner {
    /// Creates new spinner
    ///
    /// Arguments:
    ///  - `initial_mask` - consider it as ZERO without one segment or only one segment.
    ///  - `clockwise` - rotation direction true for clockwise
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

impl Animate<u8> for Spinner {
    fn next(&mut self) -> Option<u8> {
        let mut res = 0_u8;
        let max_i = self.mask.len();
        for i in 0..max_i {
            let ii = if self.cw {
                (max_i + i - self.offset as usize) % max_i
            } else {
                (i + self.offset as usize) % max_i
            };
            let v = self.mask[ii];
            if v {
                res |= CIRCLE[i];
            }
        }
        self.offset = (self.offset + 1) % max_i as u8;
        return Some(res);
    }
}

/// Defines how to slide your "text"
#[derive(Debug)]
pub enum SlideType {
    // Slides from behind last display to first, stops then fist character reaches first display
    StopAtFirstChar,
    // Slides from behind last display to first, stops then last character moves behind first display
    StopAfterLastChar,
    // Slides from last display to first with spacer equivalent to displays count
    Cycle,
}

/// Sliding animation from last display to first
#[derive(Debug)]
pub struct Slider {
    tp: SlideType,
    count: u8,
    result_len: u8,
    word: Vec<u8>,
}

impl Slider {
    /// Configure slider animation.
    ///
    /// Arguments:
    ///  - `slide_type` - animation behaviour
    ///  - `displays_count` - number of displays connected to MCU
    ///  - `bytes_to_slide` - input bytes that should slide along displays
    pub fn new(slide_type: SlideType, displays_count: u8, bytes_to_slide: &[u8]) -> Slider {
        let mut word = Vec::<u8>::new();

        for i in 0..bytes_to_slide.len() {
            word.push(bytes_to_slide[i]);
        }

        Slider {
            tp: slide_type,
            count: 0,
            result_len: displays_count,
            word: word,
        }
    }
}

impl Animate<Vec<u8>> for Slider {
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

        if off_in >= self.word.len() {
            // Stop if last char already out of screen
            if let SlideType::StopAfterLastChar = self.tp {
                return None;
            }
        }

        if (off_in as usize) < self.word.len() {
            for i in 0..self.word.len() {
                if out.len() >= self.result_len as usize {
                    break;
                }
                if i < off_in {
                    // Skip first chars
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

#[cfg(test)]
mod tests {
    use super::*;

    fn animate_next<R>(ani: &mut dyn Animate<R>, steps: usize) -> Option<R> {
        // if steps <= 0 {
        //     panic!("Wrong steps argument value = 0");
        // }
        let mut last = None;
        for _ in 0..steps {
            last = ani.next();
        }
        last
    }

    fn animate_none<R>(ani: &mut dyn Animate<R>, steps: usize) {
        for s in 0..steps {
            if let Some(_) = ani.next() {
                assert!(false, "Resultis not None at step {}", s);
            }
        }
    }

    #[test]
    fn spinner_test() {
        let init = CHAR_0 & !SEG_1;

        let mut scw = Spinner::new(init.clone(), true);
        assert_eq!(animate_next(&mut scw, 2).unwrap(), CHAR_0 & !SEG_2);
        assert_eq!(animate_next(&mut scw, 4).unwrap(), CHAR_0 & !SEG_6);
        assert_eq!(animate_next(&mut scw, 6).unwrap(), CHAR_0 & !SEG_6);
        assert_eq!(animate_next(&mut scw, 3).unwrap(), CHAR_0 & !SEG_3);

        let mut sccw = Spinner::new(init.clone(), false);
        assert_eq!(animate_next(&mut sccw, 2).unwrap(), CHAR_0 & !SEG_6);
        assert_eq!(animate_next(&mut sccw, 5).unwrap(), CHAR_0 & !SEG_1);
        assert_eq!(animate_next(&mut sccw, 6).unwrap(), CHAR_0 & !SEG_1);
        assert_eq!(animate_next(&mut sccw, 3).unwrap(), CHAR_0 & !SEG_4);
    }

    #[test]
    fn slide_test() {
        let init = [
            CHAR_0, CHAR_1, CHAR_2, CHAR_3, CHAR_4, CHAR_5, CHAR_6, CHAR_7, CHAR_8, CHAR_9,
        ];

        let mut sstop = Slider::new(SlideType::StopAtFirstChar, 5, &init);
        assert_eq!(animate_next(&mut sstop, 1).unwrap(), [0; 5]);
        assert_eq!(animate_next(&mut sstop, 1).unwrap(), [0, 0, 0, 0, CHAR_0]);
        assert_eq!(
            animate_next(&mut sstop, 4).unwrap(),
            [CHAR_0, CHAR_1, CHAR_2, CHAR_3, CHAR_4]
        );
        animate_none(&mut sstop, 15);

        let mut sstopaft = Slider::new(SlideType::StopAfterLastChar, 5, &init);
        assert_eq!(animate_next(&mut sstopaft, 1).unwrap(), [0; 5]);
        assert_eq!(
            animate_next(&mut sstopaft, 1).unwrap(),
            [0, 0, 0, 0, CHAR_0]
        );
        assert_eq!(
            animate_next(&mut sstopaft, 4).unwrap(),
            [CHAR_0, CHAR_1, CHAR_2, CHAR_3, CHAR_4]
        );
        assert_eq!(
            animate_next(&mut sstopaft, 3).unwrap(),
            [CHAR_3, CHAR_4, CHAR_5, CHAR_6, CHAR_7]
        );
        assert_eq!(
            animate_next(&mut sstopaft, 2).unwrap(),
            [CHAR_5, CHAR_6, CHAR_7, CHAR_8, CHAR_9]
        );
        assert_eq!(
            animate_next(&mut sstopaft, 4).unwrap(),
            [CHAR_9, 0, 0, 0, 0]
        );
        animate_none(&mut sstop, 15);

        let mut cycle = Slider::new(SlideType::Cycle, 5, &init);
        assert_eq!(animate_next(&mut cycle, 1).unwrap(), [0; 5]);
        assert_eq!(
            animate_next(&mut cycle, 3 * (init.len() + 5)).unwrap(),
            [0; 5]
        );
        assert_eq!(
            animate_next(&mut cycle, 5).unwrap(),
            [CHAR_0, CHAR_1, CHAR_2, CHAR_3, CHAR_4]
        );
        assert_eq!(
            animate_next(&mut cycle, 5).unwrap(),
            [CHAR_5, CHAR_6, CHAR_7, CHAR_8, CHAR_9]
        );
    }
}
