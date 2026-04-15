use tokio::sync::mpsc::{self, error::SendError};

#[derive(Debug)]
pub enum TxLog {
    PrintLn { message: String },
    Error { message: String },
    Warn { message: String },
    Info { message: String },
    Debug { message: String },
    Trace { message: String },
}

pub async fn send_tx_msg(progress_tx: &mpsc::Sender<TxLog>, log_level: Option<log::Level>, message:&str) -> Result<(), SendError<TxLog>> {
	let message = message.to_string();
	let txlog: TxLog = match log_level {
		Some(log_level) => {
			match log_level {
				log::Level::Error => TxLog::Error { message },
				log::Level::Warn => TxLog::Warn { message },
				log::Level::Info => TxLog::Info { message },
				log::Level::Debug => TxLog::Debug { message },
				log::Level::Trace => TxLog::Trace { message },
			}
		},
		None => {
			TxLog::PrintLn { message }
		}
	};
	progress_tx
		.send(txlog)
		.await
}

pub fn send_tx_msg_sync(progress_tx: &mpsc::Sender<TxLog>, log_level: Option<log::Level>, message:&str) -> Result<(), SendError<TxLog>> {
	let message = message.to_string();
	let txlog: TxLog = match log_level {
		Some(log_level) => {
			match log_level {
				log::Level::Error => TxLog::Error { message },
				log::Level::Warn => TxLog::Warn { message },
				log::Level::Info => TxLog::Info { message },
				log::Level::Debug => TxLog::Debug { message },
				log::Level::Trace => TxLog::Trace { message },
			}
		},
		None => {
			TxLog::PrintLn { message }
		}
	};
	progress_tx
		.blocking_send(txlog)
}

pub async fn send_tx_msg_op(progress_tx: Option<&mpsc::Sender<TxLog>>, log_level: Option<log::Level>, message:&str) -> Result<(), SendError<TxLog>> {
	match progress_tx {
		Some(progress_tx) => {
			let message = message.to_string();
			let txlog: TxLog = match log_level {
				Some(log_level) => {
					match log_level {
						log::Level::Error => TxLog::Error { message },
						log::Level::Warn => TxLog::Warn { message },
						log::Level::Info => TxLog::Info { message },
						log::Level::Debug => TxLog::Debug { message },
						log::Level::Trace => TxLog::Trace { message },
					}
				},
				None => {
					TxLog::PrintLn { message }
				}
			};
			progress_tx
				.send(txlog)
				.await
		},
		None => { Ok(()) }
	}
}

pub fn send_tx_msg_op_sync(progress_tx: Option<&mpsc::Sender<TxLog>>, log_level: Option<log::Level>, message:&str) -> Result<(), SendError<TxLog>> {
	match progress_tx {
		Some(progress_tx) => {
			let message = message.to_string();
			let txlog: TxLog = match log_level {
				Some(log_level) => {
					match log_level {
						log::Level::Error => TxLog::Error { message },
						log::Level::Warn => TxLog::Warn { message },
						log::Level::Info => TxLog::Info { message },
						log::Level::Debug => TxLog::Debug { message },
						log::Level::Trace => TxLog::Trace { message },
					}
				},
				None => {
					TxLog::PrintLn { message }
				}
			};
			progress_tx
				.blocking_send(txlog)
		},
		None => { Ok(()) }
	}
}

#[cfg(test)]
mod tests {
    use super::*;
	use log::Level::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_send_tx_msg_sync() {
		let mut result:String = String::new();
		if let Ok(rt) = Runtime::new() {
			let _rt_result = rt.block_on(async {
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxLog>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::task::spawn_blocking( move || { send_tx_msg_sync(&progress_tx, Some(Info), "A message to send") });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxLog::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxLog::Error { message } => { result = format!("Error: {}", message); },
						TxLog::Warn { message } => { result = format!("Warning: {}", message); },
						TxLog::Info { message } => { result = format!("Info: {}", message); },
						TxLog::Debug { message } => { result = format!("Debug: {}", message); },
						TxLog::Trace { message } => { result = format!("Trace: {}", message); },
					}
				}

				// Wait for the work to finish and get the final result
				match work_handle.await.unwrap() {
					Ok(_) => (),
					Err(e) => eprintln!("[ERROR] {}", e),
				}            
			});
		}

		let expected = String::from("Info: A message to send");
		assert_eq!(result, expected);
    }

    #[test]
    fn test_send_tx_msg_op_sync() {
		let mut result:String = String::new();
		if let Ok(rt) = Runtime::new() {
			let _rt_result = rt.block_on(async {
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxLog>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::task::spawn_blocking(move || { send_tx_msg_op_sync(Some(&progress_tx), None, "A message to send") });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxLog::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxLog::Error { message } => { result = format!("Error: {}", message); },
						TxLog::Warn { message } => { result = format!("Warning: {}", message); },
						TxLog::Info { message } => { result = format!("Info: {}", message); },
						TxLog::Debug { message } => { result = format!("Debug: {}", message); },
						TxLog::Trace { message } => { result = format!("Trace: {}", message); },
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

	#[test]
    fn test_send_tx_msg() {
		let mut result:String = String::new();
		if let Ok(rt) = Runtime::new() {
			let _rt_result = rt.block_on(async {
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxLog>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::spawn(async move { send_tx_msg(&progress_tx, Some(Info), "A message to send").await });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxLog::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxLog::Error { message } => { result = format!("Error: {}", message); },
						TxLog::Warn { message } => { result = format!("Warning: {}", message); },
						TxLog::Info { message } => { result = format!("Info: {}", message); },
						TxLog::Debug { message } => { result = format!("Debug: {}", message); },
						TxLog::Trace { message } => { result = format!("Trace: {}", message); },
					}
				}

				// Wait for the work to finish and get the final result
				match work_handle.await.unwrap() {
					Ok(_) => (),
					Err(e) => eprintln!("[ERROR] {}", e),
				}            
			});
		}

		let expected = String::from("Info: A message to send");
		assert_eq!(result, expected);
    }

    #[test]
    fn test_send_tx_msg_op() {
		let mut result:String = String::new();
		if let Ok(rt) = Runtime::new() {
			let _rt_result = rt.block_on(async {
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxLog>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::spawn(async move { send_tx_msg_op(Some(&progress_tx), None, "A message to send").await });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxLog::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxLog::Error { message } => { result = format!("Error: {}", message); },
						TxLog::Warn { message } => { result = format!("Warning: {}", message); },
						TxLog::Info { message } => { result = format!("Info: {}", message); },
						TxLog::Debug { message } => { result = format!("Debug: {}", message); },
						TxLog::Trace { message } => { result = format!("Trace: {}", message); },
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
