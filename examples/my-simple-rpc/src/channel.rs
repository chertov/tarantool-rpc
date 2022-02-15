use std::collections::VecDeque;
use std::io::{Read, Write};
use std::os::unix::prelude::{AsRawFd, IntoRawFd};
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::io::AsyncWriteExt;

pub struct TNTSender<T> {
    queue: Arc<RwLock<VecDeque<T>>>,
    tx: Arc<RwLock<os_pipe::PipeWriter>>,
}
impl<T> Clone for TNTSender<T> {
    fn clone(&self) -> Self {
        Self{ queue: self.queue.clone(), tx: self.tx.clone() }
    }
}

impl<T> TNTSender<T> {
    pub fn send(&mut self, val: T) -> Result<(), anyhow::Error> {
        // trace!("TNTSender tx.write().push_back(val)..");
        let mut lock = self.queue.write();
        lock.push_back(val);
        drop(lock);
        // trace!("TNTSender tx.write().push_back(val)..ok");
        use byteorder::WriteBytesExt;
        // trace!("TNTSender tx.write_u8(0)..");
        self.tx.write().write_u8(0)?;
        // trace!("TNTSender tx.write_u8(0).. Ok");
        Ok(())
    }
}

pub struct TNTReceiver<T: std::fmt::Debug> {
    queue: Arc<RwLock<VecDeque<T>>>,
    rx: tarantool::coio::CoIOStream,
}
impl<T: std::fmt::Debug> TNTReceiver<T> {
    pub fn recv(&mut self) -> Result<Option<T>, anyhow::Error> {
        // trace!("TNTReceiver recv.. start");
        if let Some(val) = {
            let mut rw_lock = self.queue.write();
            let val = rw_lock.pop_front();
            drop(rw_lock);
            val
        } {
            // trace!("TNTReceiver recv.. return val {:?}", val);
            // tarantool::fiber::fiber_yield();
            return Ok(Some(val));
        }
        loop {
            use byteorder::ReadBytesExt;
            // trace!("TNTReceiver self.rx.read_u8()..");
            self.rx.read_u8().map_err(|err| { log::error!("self.rx.read_u8() err: {}", err); err })?;
            // trace!("TNTReceiver self.rx.read_u8()..ok");
            if let Some(val) = {
                let mut rw_lock = self.queue.write();
                let val = rw_lock.pop_front();
                drop(rw_lock);
                val
            } {
                // trace!("TNTReceiver recv after rx.. return val {:?}", val);
                return Ok(Some(val));
            } else {
                // warn!("TNTReceiver self.queue.pop_front() is None");
            }
        }
        Ok(None)
    }
}

pub fn channel<T: std::fmt::Debug>() -> Result<(TNTSender<T>, TNTReceiver<T>), anyhow::Error> {
    let queue = Arc::new(RwLock::new(VecDeque::<T>::new()));
    let (rx, tx) = os_pipe::pipe().unwrap();

    let rx = tarantool::coio::CoIOStream::new(rx.into_raw_fd()).unwrap();
    let rx = TNTReceiver { queue: queue.clone(), rx };

    let tx = Arc::new(RwLock::new(tx));
    let tx = TNTSender { queue, tx };

    Ok((tx, rx))
}
