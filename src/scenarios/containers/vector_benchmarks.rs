use container::vector::Vector;

const VEC_COUNT: usize = 10_000;

#[repr(C)]
#[derive(Default)]
struct AllocationData {
    pub data_block_1: [usize; 10],
    pub data_block_2: [usize; 10],
    pub data_block_3: [usize; 10],
    pub data_block_4: [usize; 10],
}

pub fn vec_1000_without_cap() {
    let mut vec: Vector<AllocationData> = Vector::new();

    for _idx in 0 .. VEC_COUNT {
        vec.push(AllocationData::default());
    }
}

pub fn vec_1000_with_cap() {
    let mut vec: Vector<AllocationData> = Vector::with_capacity(VEC_COUNT);

    for _idx in 0 .. VEC_COUNT {
        vec.push(AllocationData::default());
    }
}

pub fn vec_1000_iteration() {
    let mut vec: Vector<AllocationData> = Vector::with_capacity(VEC_COUNT);

    for _idx in 0 .. VEC_COUNT {
        vec.push(AllocationData::default());
    }

    for idx in 0 .. VEC_COUNT {
        vec[idx].data_block_1[0] = 10;
    }
}

pub fn vec_1000_erase_range() {
    let mut vec: Vector<AllocationData> = Vector::with_capacity(VEC_COUNT);

    for _idx in 0 .. VEC_COUNT {
        vec.push(AllocationData::default());
    }

    vec.erase_range(0, VEC_COUNT / 2);
}