use crate::office::zip::Zip;
use crate::replace::data::Data;
use crossbeam_channel::unbounded;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

struct MsgData {
    office: Arc<Mutex<Zip>>,
    id: u64,
}

type ChannelMsg = Option<Arc<Mutex<MsgData>>>;

pub struct Thread {
    data: Arc<Data>,
    tx: crossbeam_channel::Sender<ChannelMsg>,
    rx: crossbeam_channel::Receiver<ChannelMsg>,
    threads: Vec<JoinHandle<()>>,
    index: u64,
    results: Arc<Mutex<HashMap<u64, Box<[u8]>>>>,
    concurrency: u8,
}

impl Thread {
    pub fn new(concurrency: u8, data: Arc<Data>) -> Thread {
        let (tx, rx) = unbounded();
        let threads = Vec::new();
        let mut t = Thread {
            data,
            tx,
            rx,
            threads,
            concurrency,
            index: 0,
            results: Arc::new(Mutex::new(HashMap::new())),
        };
        t.init();
        t
    }

    fn init(&mut self) {
        if self.threads.len() > 0 {
            return;
        }
        self.results = Arc::new(Mutex::new(HashMap::new()));
        self.index = 0;
        self.start_concurrency();
    }

    fn start_concurrency(&mut self) {
        use std::thread;
        for _ in 0..self.concurrency {
            let data = Arc::clone(&self.data);
            let results = Arc::clone(&self.results);
            let rx = self.rx.clone();
            self.threads.push(thread::spawn(move || {
                while let Ok(option) = rx.try_recv() {
                    match option {
                        Some(msg) => {
                            // let id = msg.lock().unwrap().id.clone();
                            // let office_box = Arc::clone(&msg.lock().unwrap().office);
                            // let res = replace(office_box, &data);
                            // results.lock().unwrap().insert(id, res);
                        }
                        None => {
                            return;
                        }
                    }
                }
            }));
        }
    }

    pub fn add(&mut self, office: Arc<Mutex<Zip>>) -> u64 {
        let id = self.index;
        self.index += 1;
        let msg = MsgData { office, id };
        self.tx.send(Some(Arc::new(Mutex::new(msg)))).unwrap();
        id
    }

    pub fn over(&mut self, ids: Vec<u64>) -> Vec<Box<[u8]>> {
        for _ in 0..self.threads.len() {
            self.tx.send(None).unwrap()
        }

        while let Some(thread) = self.threads.pop() {
            thread.join().unwrap();
        }

        let mut res = Vec::new();
        let results = self.results.lock().unwrap();
        for id in ids {
            match results.get(&id) {
                Some(result) => {
                    res.push(result.clone());
                }
                None => {
                    res.push(Box::from([]));
                }
            }
        }
        res
    }
}
