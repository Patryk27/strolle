pub fn pad_size(size: usize) -> usize {
    (size + 31) & !31
}
