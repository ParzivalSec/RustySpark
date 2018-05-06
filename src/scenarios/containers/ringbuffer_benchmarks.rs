use container::ringbuffer::RingBuffer;

const RINGBUFFER_COUNT : usize = 1000;
const WRAPPING_OFFSET : usize  = 500;
 
#[repr(C)]
#[derive(Default, Copy, Clone)]
struct AllocationData {
    pub data_block_1: [usize; 10],
    pub data_block_2: [usize; 10],
    pub data_block_3: [usize; 10],
    pub data_block_4: [usize; 10],
}

pub fn ringbuffe_1000_write()
{
	let mut ringbuffer = RingBuffer::new(RINGBUFFER_COUNT);

	for _idx in 0 .. RINGBUFFER_COUNT
	{
		ringbuffer.write(AllocationData::default());
	}
}

pub fn ringbuffer_1000_read()
{
	let mut ringbuffer = RingBuffer::new(RINGBUFFER_COUNT);

	for _idx in 0 .. RINGBUFFER_COUNT
	{
		ringbuffer.write(AllocationData::default());
	}

	for _idx in 0 .. RINGBUFFER_COUNT
	{
		let _data = ringbuffer.read().unwrap();
	}
}

pub fn ringbuffer_500_write_wrapping()
{
	let mut ringbuffer = RingBuffer::new(RINGBUFFER_COUNT);

	for _idx in 0 .. RINGBUFFER_COUNT + WRAPPING_OFFSET
	{
		ringbuffer.write(AllocationData::default());
	}
}