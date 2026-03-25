use std::ops::Range;

pub const trait Span {
    fn span(&self) -> usize;
}

impl const Span for Range<usize> {
    fn span(&self) -> usize {
        self.end - self.start
    }
}