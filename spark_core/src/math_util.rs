pub fn round_to_next_multiple(num: usize, multiple: usize) -> usize {
    let remainer = num % multiple;
    let multiplicator = if remainer == 0 {
        0
    } else {
        1
    };

    num - remainer + multiple * multiplicator
}

pub fn round_to_previous_multiple(num: usize, multiple: usize) -> usize {
    let remainer = num % multiple;
    num - remainer
}
