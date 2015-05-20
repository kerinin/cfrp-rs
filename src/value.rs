use std::ops::{Deref, DerefMut};
use std::fmt;
use std::cmp;

/// Value<T> encodes the difference between changed & unchanged data, in
/// cases where multiple inputs have been combined.
///
pub enum Value<T> {
    Changed(T),
    Unchanged(T),
}

impl<T> Value<T> {
    /// Returns the contained value
    ///
    pub fn into_inner(self) -> T {
        match self {
            Value::Changed(v) => v,
            Value::Unchanged(v) => v,
        }
    }
}

impl<T> Deref for Value<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        match self {
            &Value::Changed(ref v) => v,
            &Value::Unchanged(ref v) => v,
        }
    }
}

impl<T> DerefMut for Value<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        match self {
            &mut Value::Changed(ref mut v) => v,
            &mut Value::Unchanged(ref mut v) => v,
        }
    }
}

impl<T> Clone for Value<T> where T: Clone {
    fn clone(&self) -> Value<T> {
        match self {
            &Value::Changed(ref v) => Value::Changed(v.clone()),
            &Value::Unchanged(ref v) => Value::Unchanged(v.clone()),
        }
    }
}

impl<T> PartialEq for Value<T> where T: PartialEq {
    fn eq(&self, other: &Value<T>) -> bool {
        match (self, other) {
            (&Value::Changed(ref l), &Value::Changed(ref r)) => l == r,
            (&Value::Unchanged(ref l), &Value::Changed(ref r)) => l == r,
            (&Value::Changed(ref l), &Value::Unchanged(ref r)) => l == r,
            (&Value::Unchanged(ref l), &Value::Unchanged(ref r)) => l == r,
        }
    }
}

impl<T> fmt::Display for Value<T> where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (**self).fmt(f)
    }
}

impl<T> fmt::Debug for Value<T> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (**self).fmt(f)
    }
}

impl<T> Default for Value<T> where T: Default {
    fn default() -> Value<T> {
        Value::Unchanged(Default::default())
    }
}

impl<T> PartialOrd for Value<T> where T: PartialOrd {
    fn partial_cmp(&self, other: &Value<T>) -> Option<cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<T> Ord for Value<T> where T: Ord {
    fn cmp(&self, other: &Value<T>) -> cmp::Ordering {
        (**self).cmp(&**other)
    }
}

impl<T> Eq for Value<T> where T: Eq {
}

impl<T> Iterator for Value<T> where T: Iterator {
    type Item = T::Item;
    fn next(&mut self) -> Option<T::Item> {
        (**self).next()
    }
}

impl<T> DoubleEndedIterator for Value<T> where T: DoubleEndedIterator {
    fn next_back(&mut self) -> Option<T::Item> {
        (**self).next_back()
    }
}

impl<T> ExactSizeIterator for Value<T> where T: ExactSizeIterator {
    fn len(&self) -> usize {
        (**self).len()
    }
}

// FnOnce, From, Read, Write?
