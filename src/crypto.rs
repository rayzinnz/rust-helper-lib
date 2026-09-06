use aes_gcm::{
    Aes256Gcm, Nonce, aead::{Aead, Generate, Key, KeyInit, consts}, aes::{cipher::{Array, InvalidLength}}
};

pub fn generate_key_256() -> Array<u8, consts::U32> {
    // Generate a random 256-bit encryption key using the modern Generate trait
    // Note: This requires the `getrandom` feature enabled (usually on by default via rand/OsRng)
    Key::<Aes256Gcm>::generate()
}
pub fn key_256_as_bytes() -> [u8;32] {
	generate_key_256().into()
}
pub fn generate_key_256_bytes() -> [u8;32] {
    Key::<Aes256Gcm>::generate().into()
}

pub fn cipher_256_from_key(key:&[u8;32]) -> Result<Aes256Gcm, InvalidLength> {
	Aes256Gcm::new_from_slice(key)
}

pub fn nonce_96() -> Array<u8, consts::U12> {
	// Generate a unique 96-bit nonce using the modern Generate trait
	Nonce::generate()
}
pub fn nonce_96_as_bytes() -> [u8;12] {
	nonce_96().into()
}
pub fn nonce_96_from_bytes(nonce_bytes:[u8;12]) -> Array<u8, consts::U12> {
	nonce_bytes.into()
}

pub fn encrypt(cipher:&Aes256Gcm, nonce:&[u8;12], bytes:&[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
	//first 12 bytes will store the nonce
	let mut encrypted_bytes = cipher.encrypt(&nonce_96_from_bytes(*nonce), bytes)?;
	let mut payload = Vec::with_capacity(12 + encrypted_bytes.len());
    payload.extend_from_slice(nonce);
    payload.append(&mut encrypted_bytes);
    Ok(payload)
}

pub fn decrypt(cipher:&Aes256Gcm, encrypted_bytes:&[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
	//frist 12 bytes are the nonce
	if encrypted_bytes.len() < 12 {
		eprintln!("Data too short to contain a 12-byte nonce");
        return Err(aes_gcm::Error);
    }
	let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
    // Safely convert the 12-byte slice into a reference to a fixed array
    let nonce: &[u8; 12] = nonce_bytes.try_into().unwrap();
	let nonce = nonce_96_from_bytes(*nonce);
	
	cipher.decrypt(&nonce, ciphertext)
}

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn test_encrypt() {
		let nonce = [90u8, 47, 45, 252, 200, 75, 34, 109, 78, 105, 11, 163];
		let key_bytes = [33u8, 39, 233, 131, 244, 2, 18, 144, 209, 45, 48, 131, 66, 202, 118, 215, 88, 72, 124, 206, 123, 162, 123, 245, 160, 126, 169, 13, 252, 253, 174, 123];
		let cipher = cipher_256_from_key(&key_bytes).unwrap();

		let unencrypted_bytes = [238u8,150,105,170,177,66,189,129,85,147,33,243,219,251,138,51,56,195,72,37,232,122,193,75,41,192,67,2,139,199,204,39,253,190,252,98,80,63,152,190,60,54,58,118,100,13,105,55,52,101,50,203,82,98,114,236,16,9,97,7,168,0,189,102];

		let result = encrypt(&cipher, &nonce, &unencrypted_bytes).unwrap();
		let expected = [90u8, 47, 45, 252, 200, 75, 34, 109, 78, 105, 11, 163, 56u8, 239, 201, 242, 104, 254, 92, 18, 186, 49, 165, 166, 60, 211, 118, 19, 92, 218, 63, 76, 101, 225, 16, 43, 15, 123, 95, 201, 251, 203, 194, 43, 89, 226, 9, 237, 183, 74, 160, 218, 169, 13, 104, 105, 238, 178, 18, 161, 15, 30, 24, 148, 21, 110, 12, 138, 57, 30, 100, 29, 108, 110, 206, 226, 110, 57, 57, 81, 80, 99, 205, 106, 52, 9, 41, 152, 185, 185, 171, 187];
		assert_eq!(result, expected);
    }

    #[test]
    fn test_decrypt() {
		// let nonce_bytes = [90u8, 47, 45, 252, 200, 75, 34, 109, 78, 105, 11, 163];
		let key_bytes = [33u8, 39, 233, 131, 244, 2, 18, 144, 209, 45, 48, 131, 66, 202, 118, 215, 88, 72, 124, 206, 123, 162, 123, 245, 160, 126, 169, 13, 252, 253, 174, 123];
		let cipher = cipher_256_from_key(&key_bytes).unwrap();

		let encrypted_bytes = [90u8, 47, 45, 252, 200, 75, 34, 109, 78, 105, 11, 163, 56u8, 239, 201, 242, 104, 254, 92, 18, 186, 49, 165, 166, 60, 211, 118, 19, 92, 218, 63, 76, 101, 225, 16, 43, 15, 123, 95, 201, 251, 203, 194, 43, 89, 226, 9, 237, 183, 74, 160, 218, 169, 13, 104, 105, 238, 178, 18, 161, 15, 30, 24, 148, 21, 110, 12, 138, 57, 30, 100, 29, 108, 110, 206, 226, 110, 57, 57, 81, 80, 99, 205, 106, 52, 9, 41, 152, 185, 185, 171, 187];

		let result = decrypt(&cipher, &encrypted_bytes).unwrap();
		let expected = [238u8,150,105,170,177,66,189,129,85,147,33,243,219,251,138,51,56,195,72,37,232,122,193,75,41,192,67,2,139,199,204,39,253,190,252,98,80,63,152,190,60,54,58,118,100,13,105,55,52,101,50,203,82,98,114,236,16,9,97,7,168,0,189,102];
		assert_eq!(result, expected);
    }

    #[test]
    fn test_encrypt_and_decrypt() {
		let nonce = nonce_96_as_bytes();
		let key_bytes = generate_key_256_bytes();
		let cipher = cipher_256_from_key(&key_bytes).unwrap();

		let plaintext = String::from("Here is some plain text 😜.");

		let encrypted = encrypt(&cipher, &nonce, &plaintext.as_bytes()).unwrap();
		let decrypted = decrypt(&cipher, &encrypted).unwrap();
		let result = String::from_utf8(decrypted).unwrap();

		assert_eq!(result, plaintext);
    }

}
