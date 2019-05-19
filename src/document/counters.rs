//! Logic for recursive counters in a document for titles and other
//! counted items.

use std::fmt;

/// The struct that manages the counters for the document.
#[derive(Clone, Default)]
pub struct Counters {
    /// The counters.
    pub counters: Vec<usize>,
}

impl Counters {
    /// Creates a new empty counters.
    pub fn new() -> Counters {
        Counters { counters: vec![0] }
    }

    /// Increases the corresponding counter and returns it if it is correct.
    ///
    /// The counters of the subsections will be reinitialized.
    ///
    /// # Example
    ///
    /// ```
    /// # use spandex::document::counters::Counters;
    /// let mut counters = Counters::new();
    /// counters.increment(0);
    /// assert_eq!(counters.counter(0), 1);
    /// assert_eq!(counters.counter(1), 0);
    /// assert_eq!(counters.counter(2), 0);
    /// counters.increment(1);
    /// assert_eq!(counters.counter(1), 1);
    /// counters.increment(1);
    /// assert_eq!(counters.counter(1), 2);
    /// counters.increment(0);
    /// assert_eq!(counters.counter(0), 2);
    /// assert_eq!(counters.counter(1), 0);
    /// println!("{}", counters);
    /// ```
    pub fn increment(&mut self, counter_id: usize) -> usize {
        self.counters.resize(counter_id + 1, 0);
        self.counters[counter_id] += 1;
        self.counters[counter_id]
    }

    /// Returns a specific value of a counter.
    pub fn counter(&self, counter_id: usize) -> usize {
        match self.counters.get(counter_id) {
            Some(i) => *i,
            None => 0,
        }
    }
}

impl fmt::Display for Counters {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}",
            self.counters
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}
