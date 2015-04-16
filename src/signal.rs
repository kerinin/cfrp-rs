use std::thread;
use std::sync::mpsc::*;

// use super::coordinator2::{Coordinator};

pub struct Signal<T> {
    // pub coordinator: &'a Coordinator,
    sender: Sender<Sender<Option<T>>>,
}

enum Either<L,R> {
    Data(L),
    Channel(R),
}

impl<'a, T: 'static + Clone + Send> Signal<T> {
    pub fn new(port: Receiver<Option<T>>) -> Signal<T> {
        let (chanchan, chanport) = channel();
        let (xchan, xport) = channel();

        let xchan1 = xchan.clone();
        thread::spawn(move || {
            loop {
                match port.recv() {
                    Ok(x) => {
                        match xchan1.send(Either::Data(x)) {
                            Ok(_) => {},
                            Err(_) => { break; },
                        }
                    }
                    Err(_) => { break; }
                }
            }
        });

        let xchan2 = xchan.clone();
        thread::spawn(move || {
            loop {
                match chanport.recv() {
                    Ok(rx) => {
                        match xchan2.send(Either::Channel(rx)) {
                            Ok(_) => {},
                            Err(_) => { break; },
                        }
                    }
                    Err(_) => { break; }
                }
            }
        });

        thread::spawn(move || {
            let mut chans: Vec<Sender<Option<T>>> = Vec::new();
            let mut fuse = false;
            loop {
                match xport.recv() {
                    Ok(Either::Data(x)) => {
                        fuse = true;
                        for c in chans.iter() {
                            match c.send(x.clone()) {
                                Ok(_) => {},
                                Err(_) => { break; },
                            }
                        }
                    }
                    Ok(Either::Channel(rx)) => {
                        if fuse {
                            panic!("Cannot subscribe to a signal that has begun data processing");
                        }
                        chans.push(rx);
                    }
                    Err(_) => { break; }
                }
            }
        });

        Signal { sender: chanchan }
    }

    pub fn send_to(&self, chan: Sender<Option<T>>) -> Result<(), SendError<Sender<Option<T>>>> {
        self.sender.send(chan)
    }
}

