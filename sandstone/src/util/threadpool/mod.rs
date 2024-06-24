//! Thread pools are used for connection handling for login procedures or status requests, thereby preventing
//! DOS attacks. After the login of a player has completed, a new dedicated thread is spawned for each 
//! player.
//! 
//! The idea of this thread pool came from the rust book guide on a multithreaded web server.
//! 
//! References:
//! https://doc.rust-lang.org/book/ch20-02-multithreaded.html

use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

pub struct ThreadPool<F: Fn() + Send + 'static> {
	workers: Vec<JoinHandle<()>>,
	pub sender: mpsc::Sender<F>,
	pub receiver: Arc<Mutex<Receiver<F>>>
}

impl<F: Fn() + Send + 'static> ThreadPool<F> {
	//TODO: continue
}