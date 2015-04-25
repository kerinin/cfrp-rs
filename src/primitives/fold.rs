use std::marker::*;

use super::*;

pub struct FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    pub parent: Box<InternalSignal<A>>,
    pub f: F,
    pub state: B,
}

impl<F, A, B> InternalSignal<B> for FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let FoldSignal {parent, f, state} = inner;

        match target {
            Some(t) => {
                println!("Fold::push_to Some");

                parent.push_to(
                    Some(
                        Box::new(
                            FoldPusher {
                                child: Some(t),
                                f: f,
                                state: state,
                                cache: None,
                            }
                        )
                    )
                );
            },
            None => {
                println!("Fold::push_to None");

                parent.push_to(
                    Some(
                        Box::new(
                            FoldPusher {
                                child: None,
                                f: f,
                                state: state,
                                cache: None,
                            }
                        )
                    )
                );
            }
        }
    }
}

struct FoldPusher<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    child: Option<Box<Push<B>>>,
    f: F,
    state: B,
    cache: Option<A>,
}

impl<F, A, B> Push<A> for FoldPusher<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    fn push(&mut self, event: Event<A>) {
        let out = match event {
            Event::Changed(a) => { 
                println!("FoldPusher handling Event::Changed");
                self.cache = Some(a.clone());
                (self.f)(&mut self.state, a);
                Event::Changed(self.state.clone())
            },
            Event::Unchanged => {
                println!("FoldPusher handling Event::Unchanged");

                match self.cache {
                    Some(ref a) => {
                        (self.f)(&mut self.state, a.clone());
                        Event::Changed(self.state.clone())
                    },
                    None => panic!("FoldPusher handling Event::Unchanged (no cached data)"),
                }
            },
            Event::NoOp => {
                println!("FoldPusher handling Event::NoOp");
                Event::NoOp
            },
            Event::Exit => {
                println!("FoldPusher handling Event::NoOp");
                Event::Exit
            },
        };

        match self.child {
            Some(ref mut c) => c.push(out),
            None => {},
        }
    }
}
