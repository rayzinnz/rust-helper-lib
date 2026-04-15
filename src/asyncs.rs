use tokio::sync::mpsc::{self, error::SendError};

#[derive(Debug)]
pub enum TxMsg {
    ProgressNumber { progress: i64 },
    Progress { message: String },
    PrintLn { message: String },
    Error { message: String },
    Warn { message: String },
    Info { message: String },
    Debug { message: String },
    Trace { message: String },
}

pub async fn send_tx_msg(progress_tx: &mpsc::Sender<TxMsg>, log_level: Option<log::Level>, message:&str) -> Result<(), SendError<TxMsg>> {
	let message = message.to_string();
	let txlog: TxMsg = match log_level {
		Some(log_level) => {
			match log_level {
				log::Level::Error => TxMsg::Error { message },
				log::Level::Warn => TxMsg::Warn { message },
				log::Level::Info => TxMsg::Info { message },
				log::Level::Debug => TxMsg::Debug { message },
				log::Level::Trace => TxMsg::Trace { message },
			}
		},
		None => {
			TxMsg::PrintLn { message }
		}
	};
	progress_tx
		.send(txlog)
		.await
}

pub fn send_tx_msg_sync(progress_tx: &mpsc::Sender<TxMsg>, log_level: Option<log::Level>, message:&str) -> Result<(), SendError<TxMsg>> {
	let message = message.to_string();
	let txlog: TxMsg = match log_level {
		Some(log_level) => {
			match log_level {
				log::Level::Error => TxMsg::Error { message },
				log::Level::Warn => TxMsg::Warn { message },
				log::Level::Info => TxMsg::Info { message },
				log::Level::Debug => TxMsg::Debug { message },
				log::Level::Trace => TxMsg::Trace { message },
			}
		},
		None => {
			TxMsg::PrintLn { message }
		}
	};
	progress_tx
		.blocking_send(txlog)
}

pub async fn send_tx_msg_op(progress_tx: Option<&mpsc::Sender<TxMsg>>, log_level: Option<log::Level>, message:&str) -> Result<(), SendError<TxMsg>> {
	match progress_tx {
		Some(progress_tx) => {
			let message = message.to_string();
			let txlog: TxMsg = match log_level {
				Some(log_level) => {
					match log_level {
						log::Level::Error => TxMsg::Error { message },
						log::Level::Warn => TxMsg::Warn { message },
						log::Level::Info => TxMsg::Info { message },
						log::Level::Debug => TxMsg::Debug { message },
						log::Level::Trace => TxMsg::Trace { message },
					}
				},
				None => {
					TxMsg::PrintLn { message }
				}
			};
			progress_tx
				.send(txlog)
				.await
		},
		None => { Ok(()) }
	}
}

pub fn send_tx_msg_op_sync(progress_tx: Option<&mpsc::Sender<TxMsg>>, log_level: Option<log::Level>, message:&str) -> Result<(), SendError<TxMsg>> {
	match progress_tx {
		Some(progress_tx) => {
			let message = message.to_string();
			let txlog: TxMsg = match log_level {
				Some(log_level) => {
					match log_level {
						log::Level::Error => TxMsg::Error { message },
						log::Level::Warn => TxMsg::Warn { message },
						log::Level::Info => TxMsg::Info { message },
						log::Level::Debug => TxMsg::Debug { message },
						log::Level::Trace => TxMsg::Trace { message },
					}
				},
				None => {
					TxMsg::PrintLn { message }
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
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxMsg>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::task::spawn_blocking( move || { send_tx_msg_sync(&progress_tx, Some(Info), "A message to send") });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxMsg::ProgressNumber { progress } => { result = format!("ProgressNumber: {}", progress); },
						TxMsg::Progress { message } => { result = format!("Progress: {}", message); },
						TxMsg::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxMsg::Error { message } => { result = format!("Error: {}", message); },
						TxMsg::Warn { message } => { result = format!("Warning: {}", message); },
						TxMsg::Info { message } => { result = format!("Info: {}", message); },
						TxMsg::Debug { message } => { result = format!("Debug: {}", message); },
						TxMsg::Trace { message } => { result = format!("Trace: {}", message); },
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
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxMsg>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::task::spawn_blocking(move || { send_tx_msg_op_sync(Some(&progress_tx), None, "A message to send") });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxMsg::ProgressNumber { progress } => { result = format!("ProgressNumber: {}", progress); },
						TxMsg::Progress { message } => { result = format!("Progress: {}", message); },
						TxMsg::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxMsg::Error { message } => { result = format!("Error: {}", message); },
						TxMsg::Warn { message } => { result = format!("Warning: {}", message); },
						TxMsg::Info { message } => { result = format!("Info: {}", message); },
						TxMsg::Debug { message } => { result = format!("Debug: {}", message); },
						TxMsg::Trace { message } => { result = format!("Trace: {}", message); },
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
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxMsg>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::spawn(async move { send_tx_msg(&progress_tx, Some(Info), "A message to send").await });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxMsg::ProgressNumber { progress } => { result = format!("ProgressNumber: {}", progress); },
						TxMsg::Progress { message } => { result = format!("Progress: {}", message); },
						TxMsg::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxMsg::Error { message } => { result = format!("Error: {}", message); },
						TxMsg::Warn { message } => { result = format!("Warning: {}", message); },
						TxMsg::Info { message } => { result = format!("Info: {}", message); },
						TxMsg::Debug { message } => { result = format!("Debug: {}", message); },
						TxMsg::Trace { message } => { result = format!("Trace: {}", message); },
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
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxMsg>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::spawn(async move { send_tx_msg_op(Some(&progress_tx), None, "A message to send").await });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxMsg::ProgressNumber { progress } => { result = format!("ProgressNumber: {}", progress); },
						TxMsg::Progress { message } => { result = format!("Progress: {}", message); },
						TxMsg::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxMsg::Error { message } => { result = format!("Error: {}", message); },
						TxMsg::Warn { message } => { result = format!("Warning: {}", message); },
						TxMsg::Info { message } => { result = format!("Info: {}", message); },
						TxMsg::Debug { message } => { result = format!("Debug: {}", message); },
						TxMsg::Trace { message } => { result = format!("Trace: {}", message); },
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
