use std::{net::{TcpStream, ToSocketAddrs}, time::Duration};

pub fn can_connect(host: &str, port: u16, timeout_ms: u64) -> bool {
    if let Ok(addrs) = (host, port).to_socket_addrs() {
        let timeout = Duration::from_millis(timeout_ms);
        for addr in addrs {
            match TcpStream::connect_timeout(&addr, timeout) {
                Ok(_) => return true,
                Err(_e) => return false,
            }
        }
    }
    return false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn test_can_connect_true_windows() {
		// 135: RPC
        assert_eq!(can_connect("127.0.0.1", 135, 100), true);
    }

	#[cfg(target_os = "linux")]
    #[test]
    fn test_can_connect_true_linux() {
		// 22: SSH
        assert_eq!(can_connect("127.0.0.1", 22, 100), true);
    }

	#[test]
    fn test_can_connect_false() {
        assert_eq!(can_connect("fakehost", 80, 100), false);
    }
}
