pub struct RingBuffer<T: Copy> {
    empty:      bool,
    write_idx:  usize,
    read_idx:   usize,
    capacity:   usize,
    items:      Vec<T>,
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let mut items = Vec::with_capacity(capacity);
        items.resize(capacity, T::default());

        RingBuffer {
            empty:      true,
            write_idx:  0,
            read_idx:   0,
            capacity,
            items,
        }
    }

    pub fn write(&mut self, item: T) 
    {
        if self.write_idx == self.read_idx && !self.empty{
            self.read_idx += 1;
        }

        self.empty = false;

        self.items[self.write_idx] = item;
        self.write_idx = (self.write_idx + 1) % self.capacity;
    }

    pub fn write_clone(&mut self, item: &T) 
        where T: Clone
    {
        if self.write_idx == self.read_idx && !self.empty{
            self.read_idx += 1;
        }

        self.empty = false;

        self.items[self.write_idx] = item.clone();
        self.write_idx = (self.write_idx + 1) % self.capacity;

        if self.write_idx == self.read_idx {
            self.read_idx += 1;
        }
    }

    pub fn read(&mut self) -> Option<T> {
        if !self.is_empty() {
            let read_item = self.items[self.read_idx];
            self.read_idx = (self.read_idx + 1) % self.capacity;
            self.empty = self.read_idx == self.write_idx;
            return Some(read_item)
        }

        None
    }

    pub fn peek(&mut self) -> Option<&T> {
        if !self.is_empty() {
            return Some(&self.items[self.read_idx])
        }

        None
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if !self.is_empty() {
            return Some(&mut self.items[self.read_idx])
        }

        None
    }

    pub fn reset(&mut self) {
        self.write_idx = 0;
        self.read_idx = 0;
        self.empty = true;
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn size(&self) -> usize {
        if self.is_empty() {
            return 0
        }

        if self.read_idx < self.write_idx {
            self.write_idx - self.read_idx
        }
        else {
            let read_offset_to_end = self.capacity - self.read_idx;
            read_offset_to_end - self.write_idx
        }
     }

    pub fn is_empty(&self) -> bool { self.empty }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Default)]
    struct Task {
        pub data:   usize,
        pub id:     usize,
    }

    #[test]
    fn construction() {
        let rbf: RingBuffer<Task> = RingBuffer::new(10);

        assert_eq!(rbf.capacity(), 10, "RingBuffer was not created with enough capacity");
        assert_eq!(rbf.is_empty(), true, "RingBuffer was not empty at the beginning");
        assert_eq!(rbf.size(), 0, "RingBuffer was not created with a size of 0 although it should be empty");
    }

    #[test]
    fn none_on_read_empty() {
        let mut rbf: RingBuffer<Task> = RingBuffer::new(10);
        assert!(rbf.read().is_none(), "Reading from an empty buffer did not return NONE");
    }

    #[test]
    fn none_on_peek_empty() {
        let mut rbf: RingBuffer<Task> = RingBuffer::new(10);
        assert!(rbf.peek().is_none(), "Peeking from an empty buffer did not return NONE");
    }

    #[test]
    fn write() {
        let mut rbf: RingBuffer<Task> = RingBuffer::new(10);

        for idx in 0..9 {
            rbf.write(Task {
                data: idx * 10,
                id: idx,
            });
        }

        assert_eq!(rbf.size(), 9, "RingBuffer does not contain 10 tasks after wrtie loop");

        for idx in 0..9 {
            let task = rbf.read().unwrap();
            assert_eq!(task.id, idx, "Task id did not match");
        }

        assert!(rbf.is_empty(), "RingBuffer was not empty after reading all values");
    }

    #[test]
    fn peek() {
        let mut rbf: RingBuffer<Task> = RingBuffer::new(5);

        for idx in 0..2 {
            rbf.write(Task {
                data: idx * 10,
                id: idx,
            });
        }

        let peek_id_1;
        let peek_id_2;
        {
            let peek_1 = rbf.peek().unwrap();
            peek_id_1 = peek_1.id;
        }
        {
            let peek_2 = rbf.peek().unwrap();
            peek_id_2 = peek_2.id;
        }
        assert_eq!(peek_id_1, peek_id_2, "Peek() did not return the same value twice");

        assert!(!rbf.is_empty(), "Peek() did consume a value");
        assert_eq!(rbf.size(), 2, "Peek() corrupted buffer size");
    }

    #[test]
    fn reset() {
        let mut rbf: RingBuffer<Task> = RingBuffer::new(10);

        for idx in 0..9 {
            rbf.write(Task {
                data: idx * 10,
                id: idx,
            });
        }

        assert_eq!(rbf.size(), 9, "RingBuffer does not contain 10 tasks after wrtie loop");

        rbf.reset();

        assert!(rbf.is_empty(), "RingBuffer was not empty after calling reset");
    }

}