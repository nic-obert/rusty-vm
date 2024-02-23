

/// A doubly-linked list that allows accessing its raw pointers for manual unsafe operations.
///
/// The methods marked `unsafe` have undefined behavior if used improperly. Use at your own risk.
pub struct OpenLinkedList<T> {

    head: *mut OpenNode<T>,
    tail: *mut OpenNode<T>,
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


    /// Return the number of elements in the list. If the list was mutated unsafely, this number may be off.
    /// Because of this, this number should not be used in critical applications, but only as an estimate of the list length.
    pub fn length(&self) -> usize {
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
            prev: std::ptr::null_mut(),
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

