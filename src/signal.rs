use std::thread;
use std::sync::mpsc::*;

pub struct Signal<T> {
    sender: Sender<Sender<T>>,
}

enum Either<L,R> {
    Left(L),
    Right(R),
}

impl<T: 'static + Clone + Send> Signal<T> {
    pub fn new(port: Receiver<T>) -> Signal<T> {
        let (chanchan, chanport) = channel();
        let (xchan, xport) = channel();

        let xchan1 = xchan.clone();
        thread::spawn(move || {
            loop {
                match port.recv() {
                    Ok(x) => {
                        match xchan1.send(Either::Left(x)) {
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
                    Ok(x) => {
                        match xchan2.send(Either::Right(x)) {
                            Ok(_) => {},
                            Err(_) => { break; },
                        }
                    }
                    Err(_) => { break; }
                }
            }
        });

        thread::spawn(move || {
            let mut chans: Vec<Sender<T>> = Vec::new();
            loop {
                match xport.recv() {
                    Ok(Either::Left(x)) => {
                        for c in chans.iter() {
                            match c.send(x.clone()) {
                                Ok(_) => {},
                                Err(_) => { break; },
                            }
                        }
                    }
                    Ok(Either::Right(x)) => { chans.push(x); }
                    Err(_) => { break; }
                }
            }
        });

        Signal { sender: chanchan }
    }

    pub fn send_to(&self, chan: Sender<T>) -> Result<(), SendError<Sender<T>>> {
        self.sender.send(chan)
    }
}

