#[cfg(feature = "mkl")]
extern crate intel_mkl_src;

#[cfg(feature = "accelerate")]
extern crate accelerate_src;

use tokenizers::Tokenizer;

use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;

use candle_transformers::models::quantized_llama as model;
use model::ModelWeights;

use futures_core::stream::Stream;
use tokio::sync::Mutex;
use std::sync::Arc;

fn extract_token(next_token: u32, tokenizer: &Tokenizer) -> String {
    // Extracting the last token as a string is complicated, here we just apply some simple
    // heuristics as it seems to work well enough for this example. See the following for more
    // details:
    // https://github.com/huggingface/tokenizers/issues/1141#issuecomment-1562644141
    if let Some(text) = tokenizer.id_to_token(next_token) {
        let text = text.replace('▁', " ");
        let ascii = text
            .strip_prefix("<0x")
            .and_then(|t| t.strip_suffix('>'))
            .and_then(|t| u8::from_str_radix(t, 16).ok());
        match ascii {
            None => return text,
            Some(ascii) => {
                if let Some(chr) = char::from_u32(ascii as u32) {
                    if chr.is_ascii() {
                        return chr.to_string();
                    }
                }
                return String::from("");
            }
        }
    }
    String::from("")
}

fn format_size(size_in_bytes: usize) -> String {
    if size_in_bytes < 1_000 {
        format!("{}B", size_in_bytes)
    } else if size_in_bytes < 1_000_000 {
        format!("{:.2}KB", size_in_bytes as f64 / 1e3)
    } else if size_in_bytes < 1_000_000_000 {
        format!("{:.2}MB", size_in_bytes as f64 / 1e6)
    } else {
        format!("{:.2}GB", size_in_bytes as f64 / 1e9)
    }
}

pub struct Config {
    pub sample_len: usize,
    pub top_p: Option<f64>,
    pub seed: u64,
    pub temperature: Option<f64>,
    pub tracing: bool,
    pub verbose_prompt: bool,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
    //pub gqa: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sample_len: 100,
            top_p: None,
            seed: 299792458,
            temperature: Some(0.8),
            tracing: true,
            verbose_prompt: false,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            //gqa: None,
        }
    }
}

pub struct Llama {
    model: Arc<Mutex<ModelWeights>>,
    tokenizer: Arc<Tokenizer>,
    sample_len: usize,
    top_p: Option<f64>,
    seed: u64,
    temperature: Option<f64>,
    verbose_prompt: bool,
    repeat_penalty: f32,
    repeat_last_n: usize,
    //gqa: Option<usize>,
}

impl Llama {
    pub fn new(model_path: &str, tokenizer_path: &str, c: Config) -> anyhow::Result<Self> {
        /*
        use tracing_chrome::ChromeLayerBuilder;
        use tracing_subscriber::prelude::*;
        let _guard = if c.tracing {
            let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
            tracing_subscriber::registry().with(chrome_layer).init();
            Some(guard)
        } else {
            None
        };
        */
        println!(
            "avx: {}, neon: {}, simd128: {}, f16c: {}",
            candle_core::utils::with_avx(),
            candle_core::utils::with_neon(),
            candle_core::utils::with_simd128(),
            candle_core::utils::with_f16c()
        );

        let tokenizer_path = std::path::PathBuf::from(tokenizer_path);
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(anyhow::Error::msg)?;


        let mut file = std::fs::File::open(model_path)?;
        let start = std::time::Instant::now();
    
        let model = gguf_file::Content::read(&mut file)?;
        let mut total_size_in_bytes = 0;
        for (_, tensor) in model.tensor_infos.iter() {
            let elem_count = tensor.shape.elem_count();
            total_size_in_bytes +=
                elem_count * tensor.ggml_dtype.type_size() / tensor.ggml_dtype.blck_size();
        }
        println!(
            "loaded {:?} tensors ({}) in {:.2}s",
            model.tensor_infos.len(),
            &format_size(total_size_in_bytes),
            start.elapsed().as_secs_f32(),
        );
        let model = ModelWeights::from_gguf(model, &mut file)?;
        
        println!("model built");

        Ok(Llama {
            model: Arc::new(Mutex::new(model)),
            tokenizer: Arc::new(tokenizer),
            sample_len: c.sample_len.saturating_sub(1),
            top_p: c.top_p,
            seed: c.seed,
            temperature: c.temperature,
            verbose_prompt: c.verbose_prompt,
            repeat_penalty: c.repeat_penalty,
            repeat_last_n: c.repeat_last_n,
            //gqa: c.gqa,
        })
    }

    pub fn run(&self, prompt: String) -> impl Stream<Item = Result<String, String>> {
        let sample_len = self.sample_len;
        let top_p = self.top_p;
        let seed = self.seed;
        let temperature = self.temperature;
        let verbose_prompt = self.verbose_prompt;
        let repeat_penalty = self.repeat_penalty;
        let repeat_last_n = self.repeat_last_n;
        let tokenizer = self.tokenizer.clone();
        let model = self.model.clone();
        async_stream::try_stream! {
            let mut model = model.lock().await;
            tracing::info!("Got lock on model in tokio::spawn");
            let tokens = tokenizer.encode(prompt, true).map_err(anyhow::Error::msg)
                .map_err(|e| format!("Error: {}", e))?;
            if verbose_prompt {
                for (token, id) in tokens.get_tokens().iter().zip(tokens.get_ids().iter()) {
                    let token = token.replace('▁', " ").replace("<0x0A>", "\n");
                    println!("{id:7} -> '{token}'");
                }
            }
            let pre_prompt_tokens = vec![];
            let prompt_tokens = [&pre_prompt_tokens, tokens.get_ids()].concat();
            let prompt_tokens = if prompt_tokens.len() + sample_len > model::MAX_SEQ_LEN - 10 {
                let to_remove = prompt_tokens.len() + sample_len + 10 - model::MAX_SEQ_LEN;
                prompt_tokens[prompt_tokens.len().saturating_sub(to_remove)..].to_vec()
            } else {
                prompt_tokens
            };
            let mut all_tokens = vec![];
            let mut logits_processor = LogitsProcessor::new(seed, temperature, top_p);

            let start_prompt_processing = std::time::Instant::now();
            let mut next_token = {
                let input = Tensor::new(prompt_tokens.as_slice(), &Device::Cpu)
                    .map_err(|e| format!("Error: {}", e))?
                    .unsqueeze(0).map_err(|e| format!("Error: {}", e))?;
                let logits = model.forward(&input, 0)
                    .map_err(|e| format!("Error: {}", e))?
                    .squeeze(0).map_err(|e| format!("Error: {}", e))?;
                logits_processor.sample(&logits).map_err(|e| format!("Error: {}", e))?
            };
            let prompt_dt = start_prompt_processing.elapsed();
            all_tokens.push(next_token);
            yield extract_token(next_token, &tokenizer);

            let start_post_prompt = std::time::Instant::now();
            tracing::info!("About to enter into for loop to generate tokens");
            for index in 0..sample_len {
                let input = Tensor::new(&[next_token], &Device::Cpu)
                    .map_err(|e| format!("Error: {}", e))?
                    .unsqueeze(0).map_err(|e| format!("Error: {}", e))?;
                let logits = model.forward(&input, prompt_tokens.len() + index)
                    .map_err(|e| format!("Error: {}", e))?
                    .squeeze(0).map_err(|e| format!("Error: {}", e))?;
                let logits = if repeat_penalty == 1. {
                    logits
                } else {
                    let start_at = all_tokens.len().saturating_sub(repeat_last_n);
                    candle_transformers::utils::apply_repeat_penalty(
                        &logits,
                        repeat_penalty,
                        &all_tokens[start_at..],
                    ).map_err(|e| format!("Error: {}", e))?
                };
                next_token = logits_processor.sample(&logits).map_err(|e| format!("Error: {}", e))?;
                all_tokens.push(next_token);
                yield extract_token(next_token, &tokenizer);
            }
            let dt = start_post_prompt.elapsed();
            println!(
                "\n\n{:4} prompt tokens processed: {:.2} token/s",
                prompt_tokens.len(),
                prompt_tokens.len() as f64 / prompt_dt.as_secs_f64(),
            );
            println!(
                "{:4} tokens generated: {:.2} token/s",
                sample_len,
                sample_len as f64 / dt.as_secs_f64(),
            );
        }
    }

}