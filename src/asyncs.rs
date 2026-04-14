use anyhow::Result;
use tokio::sync::mpsc::{self, error::SendError};

#[derive(Debug)]
pub enum TxLog {
    PrintLn { message: String },
    Error { message: String },
    Warning { message: String },
    Info { message: String },
    Debug { message: String },
    Trace { message: String },
}

pub async fn send_tx_msg(progress_tx: &mpsc::Sender<TxLog>, tx_msg:TxLog) -> Result<(), SendError<TxLog>> {
	progress_tx
		.send(tx_msg)
		.await
}

pub async fn send_tx_msg_op(progress_tx: Option<&mpsc::Sender<TxLog>>, tx_msg:TxLog) -> Result<(), SendError<TxLog>> {
	match progress_tx {
		Some(progress_tx) => {
			progress_tx
				.send(tx_msg)
				.await
		},
		None => { Ok(()) }
	}
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_send_tx_msg() {
		let mut result:String = String::new();
		if let Ok(rt) = Runtime::new() {
			let _rt_result = rt.block_on(async {
				let (progress_tx, mut progress_rx) = mpsc::channel::<TxLog>(32);
				
				// Spawn the work task in a separate task so we can receive progress concurrently
				let work_handle = tokio::spawn(async move { send_tx_msg(&progress_tx, TxLog::Info { message: "A message to send".to_string() }).await });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxLog::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxLog::Error { message } => { result = format!("Error: {}", message); },
						TxLog::Warning { message } => { result = format!("Warning: {}", message); },
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
				let work_handle = tokio::spawn(async move { send_tx_msg_op(Some(&progress_tx), TxLog::PrintLn { message: "A message to send".to_string() }).await });
				
				// Receive and print progress messages as they arrive
				while let Some(status) = progress_rx.recv().await {
					match status {
						TxLog::PrintLn { message } => { result = format!("PrintLn: {}", message); },
						TxLog::Error { message } => { result = format!("Error: {}", message); },
						TxLog::Warning { message } => { result = format!("Warning: {}", message); },
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
