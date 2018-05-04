use container::handlemap::{ HandleMap, Handle };

#[repr(C)]
#[derive(Default)]
struct AllocationData {
    pub data_block_1: [usize; 10],
    pub data_block_2: [usize; 10],
    pub data_block_3: [usize; 10],
    pub data_block_4: [usize; 10],
}

const HANDLEMAP_COUNT: u32 = 1000;

pub fn handlemap_1000_insertion() {
    let mut handlemap: HandleMap<AllocationData> = HandleMap::new(HANDLEMAP_COUNT);

    for idx in 0 .. HANDLEMAP_COUNT {
        handlemap.insert(AllocationData::default());
    }
}

pub fn handlemap_1000_iteration() {
    let mut handlemap: HandleMap<AllocationData> = HandleMap::new(HANDLEMAP_COUNT);
    let mut vec: Vec<Handle> = Vec::with_capacity(HANDLEMAP_COUNT as usize);

    for idx in 0 .. HANDLEMAP_COUNT {
        let handle = handlemap.insert(AllocationData::default()).unwrap();
        vec.push(handle);
    }

    for idx in 0 .. HANDLEMAP_COUNT {
        handlemap[vec[idx as usize]].data_block_1[0] = 10;
    }
}

pub fn handlemap_1000_remove() {
    let mut handlemap: HandleMap<AllocationData> = HandleMap::new(HANDLEMAP_COUNT);
    let mut vec: Vec<Handle> = Vec::with_capacity(HANDLEMAP_COUNT as usize);

    for idx in 0 .. HANDLEMAP_COUNT {
        let handle = handlemap.insert(AllocationData::default()).unwrap();
        vec.push(handle);
    }

    for idx in 0 .. HANDLEMAP_COUNT/ 2 {
        handlemap.remove(vec[idx as usize]);
    }
}
