use tokio::sync::mpsc::{self, error::SendError};

#[derive(Debug)]
pub struct TxMsg {
	pub txlevel:TxLevel,
	pub message: String,
}

#[derive(Debug)]
pub enum TxLevel {
    Progress,
    PrintLn,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

pub async fn send_tx_msg(progress_tx: &mpsc::Sender<TxMsg>, level: TxLevel, message:&str) -> Result<(), SendError<TxMsg>> {
	let message = message.to_string();
	let txmsg: TxMsg = TxMsg { txlevel: level, message };
	progress_tx
		.send(txmsg)
		.await
}

pub fn send_tx_msg_sync(progress_tx: &mpsc::Sender<TxMsg>, level: TxLevel, message:&str) -> Result<(), SendError<TxMsg>> {
	let message = message.to_string();
	let txmsg: TxMsg = TxMsg { txlevel: level, message };
	progress_tx
		.blocking_send(txmsg)
}

pub async fn send_tx_msg_op(progress_tx: Option<&mpsc::Sender<TxMsg>>, level: TxLevel, message:&str) -> Result<(), SendError<TxMsg>> {
	match progress_tx {
		Some(progress_tx) => {
			let message = message.to_string();
			let txmsg: TxMsg = TxMsg { txlevel: level, message };
			progress_tx
				.send(txmsg)
				.await
		},
		None => { Ok(()) }
	}
}

pub fn send_tx_msg_op_sync(progress_tx: Option<&mpsc::Sender<TxMsg>>, level: TxLevel, message:&str) -> Result<(), SendError<TxMsg>> {
	match progress_tx {
		Some(progress_tx) => {
			let message = message.to_string();
			let txmsg: TxMsg = TxMsg { txlevel: level, message };
			progress_tx
				.blocking_send(txmsg)
		},
		None => { Ok(()) }
	}
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_send_tx_msg_sync() {
		let mut result:String = String::new();

		let (progress_tx, mut progress_rx) = mpsc::channel::<TxMsg>(32);
		
		// Spawn the work task in a separate thread so we can receive progress concurrently
		let join_handle = thread::Builder::new()
			.spawn(move || { send_tx_msg_sync(&progress_tx, TxLevel::Info, "A message to send") })
			.expect("Failed to spawn thread");
		
		// Receive and print progress messages as they arrive
		while let Some(txmsg) = progress_rx.blocking_recv() {
			match txmsg.txlevel {
				TxLevel::Progress => { result = format!("Progress: {}", txmsg.message); },
				TxLevel::PrintLn => { result = format!("PrintLn: {}", txmsg.message); },
				TxLevel::Error => { result = format!("Error: {}", txmsg.message); },
				TxLevel::Warn => { result = format!("Warn: {}", txmsg.message); },
				TxLevel::Info => { result = format!("Info: {}", txmsg.message); },
				TxLevel::Debug => { result = format!("Debug: {}", txmsg.message); },
				TxLevel::Trace => { result = format!("Trace: {}", txmsg.message); },
			}
		}

		// Wait for the work to finish and get the final result
		if let Err(e) = join_handle.join() {
			eprintln!("thread join error: {:?}", e);
		}

		let expected = String::from("Info: A message to send");
		assert_eq!(result, expected);
    }

    #[test]
    fn test_send_tx_msg_op_sync() {
		let mut result:String = String::new();
		let (progress_tx, mut progress_rx) = mpsc::channel::<TxMsg>(32);
		
		// Spawn the work task in a separate thread so we can receive progress concurrently
		let join_handle = thread::Builder::new()
			.spawn(move || { send_tx_msg_op_sync(Some(&progress_tx), TxLevel::Progress, "A message to send") })
			.expect("Failed to spawn thread");
		
		// Receive and print progress messages as they arrive
		while let Some(txmsg) = progress_rx.blocking_recv() {
			match txmsg.txlevel {
				TxLevel::Progress => { result = format!("Progress: {}", txmsg.message); },
				TxLevel::PrintLn => { result = format!("PrintLn: {}", txmsg.message); },
				TxLevel::Error => { result = format!("Error: {}", txmsg.message); },
				TxLevel::Warn => { result = format!("Warn: {}", txmsg.message); },
				TxLevel::Info => { result = format!("Info: {}", txmsg.message); },
				TxLevel::Debug => { result = format!("Debug: {}", txmsg.message); },
				TxLevel::Trace => { result = format!("Trace: {}", txmsg.message); },
			}
		}

		if let Err(e) = join_handle.join() {
			eprintln!("thread join error: {:?}", e);
		}

		let expected = String::from("Progress: A message to send");
		assert_eq!(result, expected);
    }

	#[test]
    fn test_send_tx_msg() {
		let mut result:String = String::new();
		if let Ok(rt) = Runtime::new() {
			let _rt_result = rt.block_on(async {
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxMsg>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::spawn(async move { send_tx_msg(&progress_tx, TxLevel::Warn, "A message to send").await });
				
				// Receive and print progress messages as they arrive
				while let Some(txmsg) = progress_rx.recv().await {
					match txmsg.txlevel {
						TxLevel::Progress => { result = format!("Progress: {}", txmsg.message); },
						TxLevel::PrintLn => { result = format!("PrintLn: {}", txmsg.message); },
						TxLevel::Error => { result = format!("Error: {}", txmsg.message); },
						TxLevel::Warn => { result = format!("Warn: {}", txmsg.message); },
						TxLevel::Info => { result = format!("Info: {}", txmsg.message); },
						TxLevel::Debug => { result = format!("Debug: {}", txmsg.message); },
						TxLevel::Trace => { result = format!("Trace: {}", txmsg.message); },
					}
				}

				// Wait for the work to finish and get the final result
				match work_handle.await.unwrap() {
					Ok(_) => (),
					Err(e) => eprintln!("[ERROR] {}", e),
				}            
			});
		}

		let expected = String::from("Warn: A message to send");
		assert_eq!(result, expected);
    }

    #[test]
    fn test_send_tx_msg_op() {
		let mut result:String = String::new();
		if let Ok(rt) = Runtime::new() {
			let _rt_result = rt.block_on(async {
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxMsg>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::spawn(async move { send_tx_msg_op(Some(&progress_tx), TxLevel::PrintLn, "A message to send").await });
				
				// Receive and print progress messages as they arrive
				while let Some(txmsg) = progress_rx.recv().await {
					match txmsg.txlevel {
						TxLevel::Progress => { result = format!("Progress: {}", txmsg.message); },
						TxLevel::PrintLn => { result = format!("PrintLn: {}", txmsg.message); },
						TxLevel::Error => { result = format!("Error: {}", txmsg.message); },
						TxLevel::Warn => { result = format!("Warn: {}", txmsg.message); },
						TxLevel::Info => { result = format!("Info: {}", txmsg.message); },
						TxLevel::Debug => { result = format!("Debug: {}", txmsg.message); },
						TxLevel::Trace => { result = format!("Trace: {}", txmsg.message); },
					}
				}

				// Wait for the work to finish and get the final result
				match work_handle.await.unwrap() {
					Ok(_) => (),
					Err(e) => eprintln!("[ERROR] {}", e),
				}            
			});
		}

		let expected = String::from("PrintLn: A message to send");
		assert_eq!(result, expected);
    }
}
