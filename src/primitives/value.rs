use super::input::Input;

pub struct Value<T>(pub T);

impl<T> Input<T> for Value<T> where
    T: 'static + Send + Clone
{
    fn pull(&mut self) -> Option<T> {
        Some(self.0.clone())
    }
}
