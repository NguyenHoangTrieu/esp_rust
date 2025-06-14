pub struct Buffer {
    data: Vec<u8>,
    cap: usize,
    first: usize,
    last: usize,
    size: usize,
}

impl Buffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0; capacity],
            cap: capacity,
            first: 0,
            last: 0, // dùng last = 0 thay vì -1
            size: 0,
        }
    }

    /// Thêm một byte vào cuối buffer
    pub fn enqueue(&mut self, byte: u8) -> bool {
        if self.size < self.cap {
            self.data[self.last] = byte;
            self.last = (self.last + 1) % self.cap;
            self.size += 1;
            true
        } else {
            false
        }
    }

    /// Lấy một byte từ đầu buffer
    pub fn dequeue(&mut self) -> Option<u8> {
        if self.size > 0 {
            let byte = self.data[self.first];
            self.first = (self.first + 1) % self.cap;
            self.size -= 1;
            Some(byte)
        } else {
            None
        }
    }

    /// Lấy toàn bộ dữ liệu và xóa buffer
    pub fn deallqueue(&mut self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.size);
        while let Some(b) = self.dequeue() {
            result.push(b);
        }
        result
    }

    /// Số phần tử hiện có
    pub fn available(&self) -> usize {
        self.size
    }
}
