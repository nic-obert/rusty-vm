use std::collections::VecDeque;
use std::cell::RefCell;

use slint::{Model, ModelNotify, ModelTracker};



/// A ['Model`] backed by a `VecDequeue<T>`, using interior mutability.
#[derive(Default)]
pub struct QueueModel<T> {
    queue: RefCell<VecDeque<T>>,
    notify: ModelNotify,
}

impl<T: 'static> QueueModel<T> {

    /// Add a row at the end of the model
    pub fn push_back(&self, value: T) {
        self.queue.borrow_mut().push_back(value);
        self.notify.row_added(self.queue.borrow().len() - 1, 1)
    }

    /// Remove a row from the front ofthe model
    pub fn pop_front(&self) -> Option<T> {
        let v = self.queue.borrow_mut().pop_front();
        self.notify.row_removed(0, 1);
        v
    }

    /// Returns the number of elements in the model
    pub fn len(&self) -> usize {
        self.queue.borrow().len()
    }

}

impl<T> From<Vec<T>> for QueueModel<T> {
    fn from(v: Vec<T>) -> Self {
        QueueModel { queue: RefCell::new(VecDeque::from(v)), notify: Default::default() }
    }
}

impl<T> FromIterator<T> for QueueModel<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        QueueModel::from(Vec::from_iter(iter))
    }
}

impl<T: Clone + 'static> Model for QueueModel<T> {
    type Data = T;

    fn row_count(&self) -> usize {
        self.queue.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.queue.borrow().get(row).cloned()
    }

    fn set_row_data(&self, row: usize, data: Self::Data) {
        if row < self.row_count() {
            self.queue.borrow_mut()[row] = data;
            self.notify.row_changed(row);
        }
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}
