use std::{
    io::Read,
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
};

use ptyprocess::stream::Stream;

pub struct TryReader {
    _keep_alive: Arc<()>,
    rx: Receiver<String>,
}

impl TryReader {
    pub fn new(mut reader: Stream) -> Self {
        let (tx, rx) = mpsc::channel::<String>();
        let keep_alive = Arc::new(());
        let keep_alive_clone = keep_alive.clone();

        thread::spawn(move || {
            let keep_alive = Arc::downgrade(&keep_alive_clone);
            let mut buf = vec![0; 65536];

            while keep_alive.strong_count() > 0 {
                let amount = reader.read(buf.as_mut_slice()).unwrap_or(0);

                if amount > 0 {
                    tx.send(String::from_utf8_lossy(&buf[..amount]).into_owned())
                        .expect("could not send string from try_read thread to main thread");
                    buf.clear();
                }
            }
        });

        Self {
            _keep_alive: keep_alive,
            rx,
        }
    }

    pub fn try_read(&self) -> Option<String> {
        self.rx.try_recv().ok()
    }
}
