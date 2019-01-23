use epoch::with_handle;

use deque_impl;

pub use deque_impl::Steal;

/// A worker queue.
///
/// This is a FIFO or LIFO queue that is owned by a single thread, but other threads may steal
/// tasks from it. Task schedulers typically create a single worker queue per thread.
///
/// # Examples
///
/// A FIFO worker:
///
/// ```
/// use crossbeam_deque::{Steal, Worker};
///
/// let w = Worker::new_fifo();
/// let s = w.stealer();
///
/// w.push(1);
/// w.push(2);
/// w.push(3);
///
/// assert_eq!(s.steal(), Steal::Success(1));
/// assert_eq!(w.pop(), Some(2));
/// assert_eq!(w.pop(), Some(3));
/// ```
///
/// A LIFO worker:
///
/// ```
/// use crossbeam_deque::{Steal, Worker};
///
/// let w = Worker::new_lifo();
/// let s = w.stealer();
///
/// w.push(1);
/// w.push(2);
/// w.push(3);
///
/// assert_eq!(s.steal(), Steal::Success(1));
/// assert_eq!(w.pop(), Some(3));
/// assert_eq!(w.pop(), Some(2));
/// ```
#[derive(Debug)]
pub struct Worker<T>(deque_impl::Worker<T>);

impl<T> Worker<T> {
    /// Creates a FIFO worker queue.
    ///
    /// Tasks are pushed and popped from opposite ends.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::Worker;
    ///
    /// let w = Worker::<i32>::new_fifo();
    /// ```
    pub fn new_fifo() -> Worker<T> {
        Worker(deque_impl::Worker::new_fifo())
    }

    /// Creates a LIFO worker queue.
    ///
    /// Tasks are pushed and popped from the same end.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::Worker;
    ///
    /// let w = Worker::<i32>::new_lifo();
    /// ```
    pub fn new_lifo() -> Worker<T> {
        Worker(deque_impl::Worker::new_lifo())
    }

    /// Creates a stealer for this queue.
    ///
    /// The returned stealer can be shared among threads and cloned.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::Worker;
    ///
    /// let w = Worker::<i32>::new_lifo();
    /// let s = w.stealer();
    /// ```
    pub fn stealer(&self) -> Stealer<T> {
        Stealer(self.0.stealer())
    }

    /// Returns `true` if the queue is empty.
    ///
    /// ```
    /// use crossbeam_deque::Worker;
    ///
    /// let w = Worker::new_lifo();
    ///
    /// assert!(w.is_empty());
    /// w.push(1);
    /// assert!(!w.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Pushes a task into the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::Worker;
    ///
    /// let w = Worker::new_lifo();
    /// w.push(1);
    /// w.push(2);
    /// ```
    pub fn push(&self, task: T) {
        with_handle(|h| self.0.push(task, h))
    }

    /// Pops a task from the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::Worker;
    ///
    /// let w = Worker::new_fifo();
    /// w.push(1);
    /// w.push(2);
    ///
    /// assert_eq!(w.pop(), Some(1));
    /// assert_eq!(w.pop(), Some(2));
    /// assert_eq!(w.pop(), None);
    /// ```
    pub fn pop(&self) -> Option<T> {
        with_handle(|h| self.0.pop(h))
    }
}

/// A stealer handle of a worker queue.
///
/// Stealers can be shared among threads.
///
/// Task schedulers typically have a single worker queue per worker thread.
///
/// # Examples
///
/// ```
/// use crossbeam_deque::{Steal, Worker};
///
/// let w = Worker::new_lifo();
/// w.push(1);
/// w.push(2);
///
/// let s = w.stealer();
/// assert_eq!(s.steal(), Steal::Success(1));
/// assert_eq!(s.steal(), Steal::Success(2));
/// assert_eq!(s.steal(), Steal::Empty);
/// ```
#[derive(Debug, Clone)]
pub struct Stealer<T>(deque_impl::Stealer<T>);

impl<T> Stealer<T> {
    /// Returns `true` if the queue is empty.
    ///
    /// ```
    /// use crossbeam_deque::Worker;
    ///
    /// let w = Worker::new_lifo();
    /// let s = w.stealer();
    ///
    /// assert!(s.is_empty());
    /// w.push(1);
    /// assert!(!s.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Steals a task from the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::{Steal, Worker};
    ///
    /// let w = Worker::new_lifo();
    /// w.push(1);
    /// w.push(2);
    ///
    /// let s = w.stealer();
    /// assert_eq!(s.steal(), Steal::Success(1));
    /// assert_eq!(s.steal(), Steal::Success(2));
    /// ```
    pub fn steal(&self) -> Steal<T> {
        with_handle(|h| self.0.steal(h))
    }

    /// Steals a batch of tasks and pushes them into another worker.
    ///
    /// How many tasks exactly will be stolen is not specified. That said, this method will try to
    /// steal around half of the tasks in the queue, but also not more than some constant limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::Worker;
    ///
    /// let w1 = Worker::new_fifo();
    /// w1.push(1);
    /// w1.push(2);
    /// w1.push(3);
    /// w1.push(4);
    ///
    /// let s = w1.stealer();
    /// let w2 = Worker::new_fifo();
    ///
    /// s.steal_batch(&w2);
    /// assert_eq!(w2.pop(), Some(1));
    /// assert_eq!(w2.pop(), Some(2));
    /// ```
    pub fn steal_batch(&self, dest: &Worker<T>) -> Steal<()> {
        with_handle(|h| self.0.steal_batch(&dest.0, h))
    }

    /// Steals a batch of tasks, pushes them into another worker, and pops a task from that worker.
    ///
    /// How many tasks exactly will be stolen is not specified. That said, this method will try to
    /// steal around half of the tasks in the queue, but also not more than some constant limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::{Steal, Worker};
    ///
    /// let w1 = Worker::new_fifo();
    /// w1.push(1);
    /// w1.push(2);
    /// w1.push(3);
    /// w1.push(4);
    ///
    /// let s = w1.stealer();
    /// let w2 = Worker::new_fifo();
    ///
    /// assert_eq!(s.steal_batch_and_pop(&w2), Steal::Success(1));
    /// assert_eq!(w2.pop(), Some(2));
    /// ```
    pub fn steal_batch_and_pop(&self, dest: &Worker<T>) -> Steal<T> {
        with_handle(|h| self.0.steal_batch_and_pop(&dest.0, h))
    }
}

/// An injector queue.
///
/// This is a FIFO queue that can be shared among multiple threads. Task schedulers typically have
/// a single injector queue, which is the entry point for new tasks.
///
/// # Examples
///
/// ```
/// use crossbeam_deque::{Injector, Steal};
///
/// let q = Injector::new();
/// q.push(1);
/// q.push(2);
///
/// assert_eq!(q.steal(), Steal::Success(1));
/// assert_eq!(q.steal(), Steal::Success(2));
/// assert_eq!(q.steal(), Steal::Empty);
/// ```
#[derive(Debug)]
pub struct Injector<T>(deque_impl::Injector<T>);

impl<T> Injector<T> {
    /// Creates a new injector queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::Injector;
    ///
    /// let q = Injector::<i32>::new();
    /// ```
    pub fn new() -> Injector<T> {
        Injector(deque_impl::Injector::new())
    }

    /// Pushes a task into the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::Injector;
    ///
    /// let w = Injector::new();
    /// w.push(1);
    /// w.push(2);
    /// ```
    pub fn push(&self, task: T) {
        self.0.push(task)
    }

    /// Steals a task from the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::{Injector, Steal};
    ///
    /// let q = Injector::new();
    /// q.push(1);
    /// q.push(2);
    ///
    /// assert_eq!(q.steal(), Steal::Success(1));
    /// assert_eq!(q.steal(), Steal::Success(2));
    /// assert_eq!(q.steal(), Steal::Empty);
    /// ```
    pub fn steal(&self) -> Steal<T> {
        self.0.steal()
    }

    /// Steals a batch of tasks and pushes them into a worker.
    ///
    /// How many tasks exactly will be stolen is not specified. That said, this method will try to
    /// steal around half of the tasks in the queue, but also not more than some constant limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::{Injector, Worker};
    ///
    /// let q = Injector::new();
    /// q.push(1);
    /// q.push(2);
    /// q.push(3);
    /// q.push(4);
    ///
    /// let w = Worker::new_fifo();
    /// q.steal_batch(&w);
    /// assert_eq!(w.pop(), Some(1));
    /// assert_eq!(w.pop(), Some(2));
    /// ```
    pub fn steal_batch(&self, dest: &Worker<T>) -> Steal<()> {
        with_handle(|h| self.0.steal_batch(&dest.0, h))
    }

    /// Steals a batch of tasks, pushes them into a worker, and pops a task from that worker.
    ///
    /// How many tasks exactly will be stolen is not specified. That said, this method will try to
    /// steal around half of the tasks in the queue, but also not more than some constant limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::{Injector, Steal, Worker};
    ///
    /// let q = Injector::new();
    /// q.push(1);
    /// q.push(2);
    /// q.push(3);
    /// q.push(4);
    ///
    /// let w = Worker::new_fifo();
    /// assert_eq!(q.steal_batch_and_pop(&w), Steal::Success(1));
    /// assert_eq!(w.pop(), Some(2));
    /// ```
    pub fn steal_batch_and_pop(&self, dest: &Worker<T>) -> Steal<T> {
        with_handle(|h| self.0.steal_batch_and_pop(&dest.0, h))
    }

    /// Returns `true` if the queue is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_deque::Injector;
    ///
    /// let q = Injector::new();
    ///
    /// assert!(q.is_empty());
    /// q.push(1);
    /// assert!(!q.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
