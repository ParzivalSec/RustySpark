pub fn round_to_next_multiple(num: usize, multiple: usize) -> usize {
    let remainer = num % multiple;
    num - remainer + multiple * !!(remainer)
}

pub fn round_to_previous_multiple(num: usize, multiple: usize) -> usize {
    let remainer = num % multiple;
    num - remainer
}
