use std::{error::Error, fs, path::Path};

use base64::prelude::*;
use serde_json::json;

pub async fn inference_on_text(url_base:&str, cf_access_client_id:&str, cf_access_client_secret:&str, prompt:&str) -> Result<String, Box<dyn Error>> {
	let payload = json!({
		"messages": [
			{"role": "user", "content": [
				{"type": "text", "text": prompt},
			]}
		]
	});

	// println!("{}", payload);

	let url = format!("{url_base}/v1/chat/completions");
	let client = reqwest::Client::new();
	let resp = client.post(url)
		.header("CF-Access-Client-Id", cf_access_client_id)
		.header("CF-Access-Client-Secret", cf_access_client_secret)
		.body(payload.to_string())
		.send()
		.await?;

    if resp.status().is_success() {
        let body = resp.text().await?;
        // println!("Response: {:#?}", body);
		if body.contains("\"content\":") {
			let json: serde_json::Value = serde_json::from_str(&body)?;
			let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("");
			return Ok(content.to_string());
		}
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
		return Err(format!("Request failed: {status} | Body: {text}").into());
    }

	return Ok(String::new())
}

pub async fn get_image_description_from_file(url_base:&str, cf_access_client_id:&str, cf_access_client_secret:&str, path:&Path) -> Result<String, Box<dyn Error>> {
	match fs::read(path) {
		Ok(bytes) => {
			let extension = path.extension().unwrap_or_default().to_string_lossy().to_string();
			let mime_type = match extension.as_str() {
				"jpg" => "image/jpg",
				"png" => "image/png",
				_ => "",
			};
			return Ok(get_image_description_from_bytes(url_base, cf_access_client_id, cf_access_client_secret, mime_type, &bytes).await?);
		}
		Err(e) => {
			return Err(format!("Error reading file: {}\n{}", path.to_string_lossy(), e).into());
		}
	}
}

pub async fn get_image_description_from_bytes(url_base:&str, cf_access_client_id:&str, cf_access_client_secret:&str, mime_type:&str, bytes:&[u8]) -> Result<String, Box<dyn Error>> {
	let base64encoded = BASE64_STANDARD.encode(bytes);
	let payload = json!({
		"messages": [
			{"role": "user", "content": [
				{"type": "text", "text": r#"Describe the image. This is not a conversation, just describe the image, no follow-up."#},
				{"type": "image_url", "image_url":
					{"url": format!("data:{mime_type};base64,{base64encoded}")}
				},
			]}
		]
	});

	// println!("{}", payload);

	let url = format!("{url_base}/v1/chat/completions");
	let client = reqwest::Client::new();
	let resp = client.post(url)
		.header("CF-Access-Client-Id", cf_access_client_id)
		.header("CF-Access-Client-Secret", cf_access_client_secret)
		.body(payload.to_string())
		.send()
		.await?;

    if resp.status().is_success() {
        let body = resp.text().await?;
        // println!("Response: {:#?}", body);
		if body.contains("\"content\":") {
			let json: serde_json::Value = serde_json::from_str(&body)?;
			let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("");
			return Ok(content.to_string());
		}
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
		return Err(format!("Request failed: {status} | Body: {text}").into());
    }

	return Ok(String::new())
}

pub async fn get_token_count(url_base:&str, cf_access_client_id:&str, cf_access_client_secret:&str, text:&str) -> Result<u32, Box<dyn Error>> {
	let payload = json!({
		"content": text
	});

	let client = reqwest::Client::new();
	let resp = client.post(format!("{url_base}/tokenize"))
		.header("CF-Access-Client-Id", cf_access_client_id)
		.header("CF-Access-Client-Secret", cf_access_client_secret)
		.body(payload.to_string())
		.send()
		.await?;

    if resp.status().is_success() {
        let body = resp.text().await?;
        // println!("Response: {:#?}", body);
		if body.contains("\"tokens\":") {
			let token_count:u32 = (body.chars().filter(|c| *c==',').count() + 1) as u32;
			// println!("{}",token_count);
			return Ok(token_count)
		}
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
		return Err(format!("Request failed: {status} | Body: {text}").into());
    }

	return Err("No embeddings returned.".into());
}

async fn get_embeddings_internal(url_base:&str, cf_access_client_id:&str, cf_access_client_secret:&str, text:&str) -> Result<String, Box<dyn Error>> {
	let payload = json!({
		"LLAMA_UBATCH_SIZE": 2048,
		"content": text,
	});

	let client = reqwest::Client::new();
	let resp;
	if url_base.contains("localhost") || url_base.contains("127.0.0.1") || url_base.contains("192.168") {
		let mut port: u16 = 49173;
		let mut host = url_base;
		if url_base.contains(":") {
			let split = url_base.split_once(":").unwrap_or_default();
			host = split.0;
			port = split.1.parse()?;
		}
		resp = client.post(format!("http://{host}:{port}/embedding"))
			.body(payload.to_string())
			.send()
			.await?;
	} else {
		resp = client.post(format!("{url_base}/embedding"))
			.header("CF-Access-Client-Id", cf_access_client_id)
			.header("CF-Access-Client-Secret", cf_access_client_secret)
			.body(payload.to_string())
			.send()
			.await?;
	}

    if resp.status().is_success() {
        let body = resp.text().await?;
        // println!("Response: {:#?}", body);
		if body.contains("\"embedding\":") {
			let mut embeddings = body.split_once("\"embedding\":").expect("error splitting json at embeddings").1.to_string();
			embeddings.retain(|c| c.is_ascii_digit() || c==',' || c=='.' || c=='-' || c=='e');
        	// println!("embeddings: {:?}", embeddings);
			// let embeddings: Vec<f64> = embeddings
			// 	.split(',')
			// 	.map(|x| x.parse::<f64>().expect(&format!("error parsing: {}", x)))
			// 	.collect();
        	// println!("embeddings: {:?}", embeddings);
        	// println!("embeddings len: {}", embeddings.len());
			return Ok(embeddings)
		}
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
		return Err(format!("Request failed: {status} | Body: {text}").into());
    }

	return Err("No embeddings returned.".into());
}

pub async fn get_embeddings(url_base:&str, cf_access_client_id:&str, cf_access_client_secret:&str, text:&str) -> Result<Vec<f64>, Box<dyn Error>> {
	let embeddings = get_embeddings_internal(url_base, cf_access_client_id, cf_access_client_secret, text).await?;
	let embeddings: Vec<f64> = embeddings
		.split(',')
		.map(|x| x.parse::<f64>().expect(&format!("error parsing: {}", x)))
		.collect();
	
	Ok(embeddings)
}

pub async fn get_embeddings_f32(url_base:&str, cf_access_client_id:&str, cf_access_client_secret:&str, text:&str) -> Result<Vec<f32>, Box<dyn Error>> {
	let embeddings = get_embeddings_internal(url_base, cf_access_client_id, cf_access_client_secret, text).await?;
	let embeddings: Vec<f32> = embeddings
		.split(',')
		.map(|x| x.parse::<f32>().expect(&format!("error parsing: {}", x)))
		.collect();
	
	Ok(embeddings)
}
