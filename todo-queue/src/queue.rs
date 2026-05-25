use borsh_derive::{BorshDeserialize, BorshSerialize};
use std::collections::vec_deque::Iter;
use std::collections::VecDeque;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Queue<T> {
    items: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, item: T) {
        self.items.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.items.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let queue: Queue<u64> = Queue::new();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.peek(), None);
    }

    #[test]
    fn enqueue_adds_items_to_back() {
        let mut queue = Queue::new();

        queue.enqueue("first");
        queue.enqueue("second");

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.peek(), Some(&"first"));
    }

    #[test]
    fn dequeue_removes_items_from_front() {
        let mut queue = Queue::new();

        queue.enqueue("first");
        queue.enqueue("second");

        assert_eq!(queue.dequeue(), Some("first"));
        assert_eq!(queue.dequeue(), Some("second"));
        assert_eq!(queue.dequeue(), None);
        assert!(queue.is_empty());
    }

    #[test]
    fn iter_returns_items_in_fifo_order() {
        let mut queue = Queue::new();

        queue.enqueue(10);
        queue.enqueue(20);
        queue.enqueue(30);

        let items: Vec<_> = queue.iter().copied().collect();

        assert_eq!(items, vec![10, 20, 30]);
    }
}
