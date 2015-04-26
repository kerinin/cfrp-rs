use super::input::Input;

pub struct Value<T>(pub T);

impl<T> Input<T> for Value<T> where
    T: 'static + Send + Clone
{
    fn pull(&mut self) -> Option<T> {
        // NOTE: Could also always emit Event::Unchanged and rely on initial value...
        Some(self.0.clone())
    }
}
