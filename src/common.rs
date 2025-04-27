use std::ops::Range;

/// span of `Range<usize>`  
/// because i want it `const` as of today i can't make it more generic or ergonomic w/out unstable (it seems)
pub const fn span(r: Range<usize>) -> usize {
    r.end - r.start
}