use std::ptr;


/// A doubly-linked list that allows accessing its raw pointers for manual unsafe operations.
///
/// The methods marked `unsafe` have undefined behavior if used improperly. Use at your own risk.
#[derive(Debug)]
pub struct OpenLinkedList<T> {

    head: *mut OpenNode<T>,
    tail: *mut OpenNode<T>,
    /// Estimate of the list's length.
    length: usize,

}

#[allow(dead_code)]
impl<T> OpenLinkedList<T> {

    /// Create a new empty linked list .
    pub fn new() -> Self {
        OpenLinkedList {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            length: 0
        }
    }


    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }


    /// Return the number of elements in the list. If the list was mutated unsafely, this number may be off.
    /// Because of this, this number should not be used in critical applications, but only as an estimate of the list length.
    pub fn estimated_length(&self) -> usize {
        self.length
    }


    pub unsafe fn head(&self) -> *mut OpenNode<T> {
        self.head
    }


    pub unsafe fn tail(&self) -> *mut OpenNode<T> {
        self.tail
    }


    /// Append the item to the tail of the linked list
    pub fn push_back(&mut self, data: T) {

        self.length += 1;

        let new_node = Box::into_raw(Box::new(OpenNode {
            data,
            next: std::ptr::null_mut(),
            prev: self.tail,
        }));

        if self.tail.is_null() {
            self.head = new_node;
        } else {
            unsafe {
                (*self.tail).next = new_node;
            }
        }

        self.tail = new_node;
    }


    /// Prepend the item to the head of the linked list
    pub fn push_front(&mut self, data: T) {

        self.length += 1;

        let new_node = Box::into_raw(Box::new(OpenNode {
            data,
            next: self.head,
            prev: ptr::null_mut(),
        }));

        if self.head.is_null() {
            self.tail = new_node;
        } else {
            unsafe {
                (*self.head).prev = new_node;
            }
        }

        self.head = new_node;
    }


    /// Remove the node from the list, assuming the node is indeed in the list.
    ///
    /// Passing a node that is not in the list is undefined behavior.
    ///
    /// The node will be deallocated and its data will be returned. Accessing the node after calling this method is undefined behavior.
    pub unsafe fn remove(&mut self, node: *mut OpenNode<T>) -> T {

        self.length -= 1;

        let node = Box::from_raw(node);

        if node.prev.is_null() {
            self.head = node.next;
        } else {
            (*node.prev).next = node.next;
        }

        if node.next.is_null() {
            self.tail = node.prev;
        } else {
            (*node.next).prev = node.prev;
        }

        node.data
    }


    /// Create an immutable iterator over the linked list. Mutating the list or any of its elements while iterating is undefined behavior.
    pub fn iter(&self) -> OpenLinkedListIterator<'_, T> {
        OpenLinkedListIterator::new(self.head)
    }


    /// Split the list at the specified node.
    /// Returns a tuple containing:
    /// - the slice before the node, excluding the `at` node
    /// - the slice after the node, including the `at` node
    /// 
    /// One of the returned slices may be empty if the `at` node is a list bound.
    /// 
    /// Assumes the `at` node is in the list.
    pub unsafe fn split_before(mut self, at: *mut OpenNode<T>) -> (Self, Self) {

        let at_node = unsafe { &mut *at };

        let prev = at_node.prev;
        
        // Disconnect the bound nodes
        at_node.prev = ptr::null_mut();
        if let Some(prev_node) = unsafe { prev.as_mut() } {
            prev_node.next = ptr::null_mut();
        } else {
            self.head = ptr::null_mut();
        }

        // Head and tail may coincide, but that's ok
        let first_half = Self {
            head: self.head,
            tail: prev,
            length: 0, // It's not important to keep the length updated as it's just an estimate.
        };

        let second_half = Self {
            head: at,
            tail: self.tail,
            length: 0,
        };

        // Invalidate the original list
        self.head = ptr::null_mut();
        self.tail = ptr::null_mut();

        (first_half, second_half)
    }


    pub unsafe fn split_after(mut self, at: *mut OpenNode<T>) -> (Self, Self) {
        
        let at_node = unsafe { &mut *at };

        let next: *mut OpenNode<T> = at_node.next;

        // Disconnect the bound nodes
        at_node.next = ptr::null_mut();
        if let Some(next_node) = unsafe { next.as_mut() } {
            next_node.prev = ptr::null_mut();
        } else {
            self.tail = ptr::null_mut();
        }

        let first_half = Self {
            head: self.head,
            tail: at,
            length: 0,
        };

        let second_half = Self {
            head: next,
            tail: self.tail,
            length: 0,
        };

        // Invalidate the original list
        self.head = ptr::null_mut();
        self.tail = ptr::null_mut();

        (first_half, second_half)
    }


}

impl<T> Drop for OpenLinkedList<T> {
    fn drop(&mut self) {
        let mut node_ptr = self.head;

        while !node_ptr.is_null() {
            let node = unsafe { Box::from_raw(node_ptr) };
            node_ptr = node.next;
        }
    }
}

impl<T> Default for OpenLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}


pub struct OpenLinkedListIterator<'a, T> {
    next: Option<&'a OpenNode<T>> 
}

impl<'a, T> OpenLinkedListIterator<'a, T> {

    fn new(start: *const OpenNode<T>) -> Self {
        Self {
            next: unsafe { start.as_ref() }
        }
    }

}

impl<'a, T> Iterator for OpenLinkedListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.next {
            self.next = unsafe { node.next.as_ref() };
            Some(&node.data)
        } else {
            None
        }
    }
}


pub struct OpenNode<T> {

    pub data: T,
    next: *mut OpenNode<T>,
    prev: *mut OpenNode<T>,

}

#[allow(dead_code)]
impl<T> OpenNode<T> {

    pub unsafe fn next(&self) -> *mut OpenNode<T> {
        self.next
    }

    pub unsafe fn prev(&self) -> *mut OpenNode<T> {
        self.prev
    }

}

