use std::{fs, path::Path};

use anyhow::{Result, bail};
use base64::prelude::*;
use serde_json::json;

pub struct LLMHost {
	host: String,
	port: u16, //if 0, use default for specific function
}

pub struct LLMCloudflare {
	url: String,
	access_client_id: String,
	access_client_secret: String,
}

pub enum LLMEndpoint {
    Local(LLMHost),
    Cloudflare(LLMCloudflare),
}

impl From<LLMHost> for LLMEndpoint {
    fn from(value: LLMHost) -> Self {
        LLMEndpoint::Local(value)
    }
}

impl From<LLMCloudflare> for LLMEndpoint {
    fn from(value: LLMCloudflare) -> Self {
        LLMEndpoint::Cloudflare(value)
    }
}

// pub async fn inference_on_text(endpoint:LLMEndpoint, prompt:&str) -> Result<String> {
// 	match endpoint {
// 		LLMEndpoint::Local(host) => {
// 			Ok(format!("Local response from {}:{}", host.host, host.port))
// 		}
// 		LLMEndpoint::Cloudflare(cf) => {
// 			Ok(format!("Cloudflare response from {}", cf.url))
// 		}
// 	}
// }

// inference_on_text(
//     (LLMHost { host: "127.0.0.1".into(), port: 8080 }).into(),
//     "Hello",
// ).await?;


// inference_on_text(
//     LLMEndpoint::Local(LLMHost { host: "127.0.0.1".into(), port: 8080 }),
//     "Hello",
// ).await?;

// inference_on_text(
//     LLMEndpoint::Cloudflare(LLMCloudflare {
//         url: "https://llm.example.com".into(),
//         access_client_id: "...".into(),
//         access_client_secret: "...".into(),
//     }),
//     "Hello",
// ).await?;


async fn llm_call(endpoint:LLMEndpoint, url_path:&str, payload:String) -> Result<String> {
	let client = reqwest::Client::new();
	let resp;
	let mut url:String;
	match endpoint {
		LLMEndpoint::Local(host) => {
			url = host.host;
			if !url.to_lowercase().starts_with("http") {
				url.insert_str(0, "http://");
			}
			let port = host.port;
			resp = client.post(format!("{url}:{port}{url_path}"))
				.body(payload)
				.send()
				.await?;
		}
		LLMEndpoint::Cloudflare(cf) => {
			url = cf.url;
			if !url.to_lowercase().starts_with("http") {
				url.insert_str(0, "https://");
			}
			url.push_str(url_path);
			resp = client.post(url)
				.header("CF-Access-Client-Id", cf.access_client_id)
				.header("CF-Access-Client-Secret", cf.access_client_secret)
				.body(payload)
				.send()
				.await?;
		}
	}

    if resp.status().is_success() {
        let body = resp.text().await?;
		Ok(body)
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
		bail!("Request failed: {status} | Body: {text}")
    }
}

pub async fn inference_on_text(endpoint:LLMEndpoint, prompt:&str) -> Result<String> {
	let payload = json!({
		"messages": [
			{"role": "user", "content": [
				{"type": "text", "text": prompt},
			]}
		]
	});

	// println!("{}", payload);

	let body = llm_call(endpoint, "/v1/chat/completions", payload.to_string()).await?;

	if body.contains("\"content\":") {
		let json: serde_json::Value = serde_json::from_str(&body)?;
		let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("");
		return Ok(content.into());
	} else {
		bail!("Body parse error. Content not found: {}", body);
	}
}

pub async fn get_image_description_from_file(endpoint:LLMEndpoint, path:&Path) -> Result<String> {
	match fs::read(path) {
		Ok(bytes) => {
			let extension = path.extension().unwrap_or_default().to_string_lossy().to_string();
			let mime_type = match extension.as_str() {
				"jpg" => "image/jpg",
				"png" => "image/png",
				_ => "",
			};
			return Ok(get_image_description_from_bytes(endpoint, mime_type, &bytes).await?);
		}
		Err(e) => {
			bail!("Error reading file: {}\n{}", path.to_string_lossy(), e)
		}
	}
}

pub async fn get_image_description_from_bytes(endpoint:LLMEndpoint, mime_type:&str, bytes:&[u8]) -> Result<String> {
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

	let body = llm_call(endpoint, "/v1/chat/completions", payload.to_string()).await?;
	
	if body.contains("\"content\":") {
		let json: serde_json::Value = serde_json::from_str(&body)?;
		let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("");
		return Ok(content.to_string());
	} else {
		bail!("Body parse error. Content not found: {}", body)
	}
}

pub async fn get_token_count(endpoint:LLMEndpoint, text:&str) -> Result<u32> {
	let payload = json!({
		"content": text
	});

	let body = llm_call(endpoint, "/tokenize", payload.to_string()).await?;

	// println!("Response: {:#?}", body);
	if body.contains("\"tokens\":") {
		let token_count:u32 = (body.chars().filter(|c| *c==',').count() + 1) as u32;
		// println!("{}",token_count);
		return Ok(token_count)
	} else {
		bail!("Body parse error. Tokens not found: {}", body)
	}
}

async fn get_embeddings_internal(endpoint:LLMEndpoint, text:&str) -> Result<String> {
	let payload = json!({
		"LLAMA_UBATCH_SIZE": 2048,
		"content": text,
	});

	let body = llm_call(endpoint, "/embedding", payload.to_string()).await?;

	// println!("Response: {:#?}", body);
	if body.contains("\"embedding\":") {
		let mut embeddings = body.split_once("\"embedding\":").expect("error splitting json at embeddings").1.to_string();
		embeddings.retain(|c| c.is_ascii_digit() || c==',' || c=='.' || c=='-' || c=='e');
		return Ok(embeddings)
	} else {
		bail!("Body parse error. Embedding not found: {}", body)
	}
}

pub async fn get_embeddings(endpoint:LLMEndpoint, text:&str) -> Result<Vec<f64>> {
	let embeddings = get_embeddings_internal(endpoint, text).await?;
	let embeddings: Vec<f64> = embeddings
		.split(',')
		.map(|x| x.parse::<f64>().expect(&format!("error parsing: {}", x)))
		.collect();
	
	Ok(embeddings)
}

pub async fn get_embeddings_f32(endpoint:LLMEndpoint, text:&str) -> Result<Vec<f32>> {
	let embeddings = get_embeddings_internal(endpoint, text).await?;
	let embeddings: Vec<f32> = embeddings
		.split(',')
		.map(|x| x.parse::<f32>().expect(&format!("error parsing: {}", x)))
		.collect();
	
	Ok(embeddings)
}
