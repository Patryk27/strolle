use std::mem;
use std::ops::Range;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Allocator {
    slots: Vec<Range<usize>>,
    dirty: bool,
}

impl Allocator {
    pub fn give(&mut self, slot: Range<usize>) {
        if let Some(last_slot) = self.slots.last() {
            self.dirty |= slot.start <= last_slot.end;
        }

        self.slots.push(slot);
    }

    pub fn take(&mut self, len: usize) -> Option<Range<usize>> {
        assert!(len > 0);

        self.compact();

        let slot_id = self.slots.iter().position(|slot| slot.len() >= len)?;
        let remaining_slot_size = self.slots[slot_id].len() - len;

        if remaining_slot_size > 0 {
            let slot = &mut self.slots[slot_id];

            slot.start += len;

            Some(Range {
                start: slot.start - len,
                end: slot.start,
            })
        } else {
            Some(self.slots.remove(slot_id))
        }
    }

    fn compact(&mut self) {
        if !mem::take(&mut self.dirty) || self.slots.is_empty() {
            return;
        }

        self.slots.sort_by_key(|slot| slot.start);

        let mut idx = 0;

        while idx < (self.slots.len() - 1) {
            if self.slots[idx].end == self.slots[idx + 1].start {
                self.slots[idx].end = self.slots.remove(idx + 1).end;
            } else {
                idx += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut target = Allocator::default();

        assert_eq!(None, target.take(16));

        // ---
        // Case 1: Just one range

        target.give(0..32);

        assert_eq!(Some(0..8), target.take(8));
        assert_eq!(Some(8..16), target.take(8));
        assert_eq!(Some(16..24), target.take(8));
        assert_eq!(Some(24..32), target.take(8));
        assert_eq!(None, target.take(8));

        // ---
        // Case 2: Many ranges

        target.give(0..8);
        target.give(10..15);

        assert_eq!(Some(0..4), target.take(4));
        assert_eq!(Some(4..8), target.take(4));
        assert_eq!(Some(10..14), target.take(4));
        assert_eq!(None, target.take(4));
        assert_eq!(Some(14..15), target.take(1));
        assert_eq!(None, target.take(1));

        // ---
        // Case 3a: Compaction

        target.give(0..8);
        target.give(8..16);
        target.give(16..24);
        target.give(24..32);
        target.give(32..40);
        target.give(64..256);

        assert_eq!(Some(64..128), target.take(64));
        assert_eq!(Some(0..20), target.take(20));
        assert_eq!(Some(20..40), target.take(20));
        assert_eq!(Some(128..148), target.take(20));

        // ---
        // Case 3b: Compaction + checking if the `dirty` flag gets set properly

        target = Default::default();

        target.give(64..256);
        target.give(32..40);
        target.give(24..32);
        target.give(16..24);
        target.give(8..16);
        target.give(0..8);

        assert_eq!(Some(64..128), target.take(64));
        assert_eq!(Some(0..20), target.take(20));
        assert_eq!(Some(20..40), target.take(20));
        assert_eq!(Some(128..148), target.take(20));
    }
}
