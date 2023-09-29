#![allow(unused)]
use std::{
    fs,
    path::Path,
    env,
};
use lightningcss::{
    stylesheet::{ParserOptions, MinifyOptions, PrinterOptions, StyleSheet, ParserFlags},
    targets::{Browsers, Targets},
};
use reqwest;
use walkdir::{DirEntry, WalkDir};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;

struct KernelDirectories {
    kernel_dir: &'static str,
    rust_target: &'static str,
    include_dirs: &'static [&'static str],
}

const DIRS: [KernelDirectories; 1] = [KernelDirectories {
    kernel_dir: "examples/custom-ops/kernels/",
    rust_target: "examples/custom-ops/cuda_kernels.rs",
    include_dirs: &[],
}];

impl KernelDirectories {
    fn maybe_build_ptx(
        &self,
        cu_file: &std::path::Path,
        ptx_file: &std::path::Path,
        compute_cap: usize,
    ) -> Result<()> {
        let should_compile = if ptx_file.exists() {
            let ptx_modified = ptx_file.metadata()?.modified()?;
            let cu_modified = cu_file.metadata()?.modified()?;
            cu_modified.duration_since(ptx_modified).is_ok()
        } else {
            true
        };
        if should_compile {
            #[cfg(feature = "cuda")]
            {
                let mut command = std::process::Command::new("nvcc");
                let out_dir = ptx_file.parent().context("no parent for ptx file")?;
                let include_dirs: Vec<String> =
                    self.include_dirs.iter().map(|c| format!("-I{c}")).collect();
                command
                    .arg(format!("--gpu-architecture=sm_{compute_cap}"))
                    .arg("--ptx")
                    .args(["--default-stream", "per-thread"])
                    .args(["--output-directory", out_dir.to_str().unwrap()])
                    .arg(format!("-I/{}", self.kernel_dir))
                    .args(include_dirs)
                    .arg(cu_file);
                let output = command
                    .spawn()
                    .context("failed spawning nvcc")?
                    .wait_with_output()?;
                if !output.status.success() {
                    anyhow::bail!(
                    "nvcc error while compiling {cu_file:?}:\n\n# stdout\n{:#}\n\n# stderr\n{:#}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )
                }
            }
            #[cfg(not(feature = "cuda"))]
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(ptx_file)?;
        }
        Ok(())
    }
    fn process(&self, out_dir: &std::path::Path, compute_cap: usize) -> Result<()> {
        println!("cargo:rerun-if-changed={}", self.kernel_dir);
        let kernel_dir = PathBuf::from(self.kernel_dir);
        let out_dir = out_dir.join(self.kernel_dir);
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir)?;
        }
        let mut cu_files = vec![];
        let mut cuh_files = vec![];
        for file in std::fs::read_dir(kernel_dir)?.flatten() {
            let file = file.path();
            match file.extension().and_then(|v| v.to_str()) {
                Some("cu") => cu_files.push(file),
                Some("cuh") => cuh_files.push(file),
                _ => {}
            }
        }

        let mut ptx_paths = vec![];
        for cu_file in cu_files.iter() {
            let file_stem = cu_file
                .file_stem()
                .with_context(|| format!("no stem {cu_file:?}"))?;
            let file_stem = file_stem.to_string_lossy().into_owned();
            let ptx_file = out_dir.join(&format!("{file_stem}.ptx"));
            self.maybe_build_ptx(cu_file, &ptx_file, compute_cap)?;
            ptx_paths.push(ptx_file);
        }

        let regenerate_rs_file = true;
        if regenerate_rs_file {
            let mut file = std::fs::File::create(self.rust_target)?;
            for ptx_path in ptx_paths {
                let name = ptx_path
                    .file_stem()
                    .context("empty stem")?
                    .to_string_lossy();
                file.write_all(b"#[rustfmt::skip]\n")?;
                let const_definition = format!(
                    r#"pub const {}: &str = include_str!(concat!(env!("OUT_DIR"), "/{}/{name}.ptx"));"#,
                    name.to_uppercase().replace('.', "_"),
                    self.kernel_dir,
                );
                file.write_all(const_definition.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_path = env::var("OUT_DIR").unwrap();
    /*
    let out_dir_path = PathBuf::from(out_path.clone());
    #[cfg(feature = "cuda")]
    set_cuda_include_dir()?;
    #[cfg(feature = "cuda")]
    let compute_cap = compute_cap()?;
    #[cfg(not(feature = "cuda"))]
    let compute_cap = 0;
    for d in DIRS {
        d.process(&out_dir_path, compute_cap)?
    }
    */

    let assets_path = format!("{out_path}/assets");
    if !Path::new(&assets_path).is_dir() {
        fs::create_dir_all(&assets_path).expect("Should be able to create assets directory if not there");
    }

    let models_path = format!("{out_path}/models");
    if !Path::new(&models_path).is_dir() {
        fs::create_dir_all(&models_path).expect("Should be able to create models directory if not there");
    }

    // Download htmx
    let htmx_file_path = format!("{assets_path}/htmx.min.js");
    if !Path::new(&htmx_file_path).is_file() {
        let htmx_body = reqwest::blocking::get("https://unpkg.com/htmx.org@1.9.4/dist/htmx.min.js")
            .expect("Should be able to download htmx source code");
        let htmx_text = htmx_body.text().expect("Should be able to convert htmx body to text");
        fs::write(htmx_file_path, htmx_text).expect("Should be able to write htmx text to file");
    }

    // Download htmx sse extension
    let htmx_sse_file_path = format!("{assets_path}/sse.js");
    if !Path::new(&htmx_sse_file_path).is_file() {
        let htmx_sse_body = reqwest::blocking::get("https://unpkg.com/htmx.org/dist/ext/sse.js")
            .expect("Should be able to download htmx sse extension source code");
        let htmx_sse_text = htmx_sse_body.text().expect("Should be able to convert htmx sse extension body to text");
        fs::write(htmx_sse_file_path, htmx_sse_text).expect("Should be able to write htmx sse extension text to file");
    }

    // Download hyperscript
    let hyperscript_file_path = format!("{assets_path}/hyperscript.min.js");
    if !Path::new(&hyperscript_file_path).is_file() {
        let hyperscript_body = reqwest::blocking::get("https://unpkg.com/hyperscript.org@0.9.11")
            .expect("Should be able to download htmx source code");
        let hyperscript_text = hyperscript_body.text().expect("Should be able to convert hyperscript body to text");
        fs::write(hyperscript_file_path, hyperscript_text).expect("Should be able to write hyperscript text to file");
    }

    // Download Model
    //let llama_small_model_path = format!("{models_path}/llama-2-7b.ggmlv3.q2_K.bin");
    //if !Path::new(&llama_small_model_path).is_file() {
    //    let model_body = reqwest::blocking::get("https://huggingface.co/TheBloke/Llama-2-7B-GGML/resolve/main/llama-2-7b.ggmlv3.q2_K.bin")
    //        .expect("Couldn't download llama 2 model from huggingface");
    //    let model_text = model_body.text().expect("Couldn't convert model body to text");
    //    fs::write(llama_small_model_path, model_text).expect("Couldn't write model text to file");
    //}


    // Walk the assets directory and copy them into the assets folder in the build directory
    let walker = WalkDir::new("assets").into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.expect("Couldn't get entry from assets dir");
        if entry.file_type().is_dir() {
            continue;  // Skip directories
        }
        let filename = entry.file_name().to_str().expect("Couldn't convert filename to string");
        dbg!(filename);
        if filename == "utils.css" {
            continue;
        }
        let text = fs::read_to_string(entry.path()).expect("Couldn't read entry");
        let path = format!("{}/{}", assets_path, filename);
        fs::write(path, text).expect("Couldn't write entry text in assets directory");
    }

    // lightning css
    let targets: Targets = Targets::from(Browsers {
        safari: Some((9 << 16) | (3 << 8)),
        chrome: Some(69 << 16),
        edge: Some(107 << 16),
        android: Some(50 << 16),
        firefox: Some(102 << 16),
        ie: Some(8 << 16),
        ios_saf: Some((11 << 16) | (3 << 8)),
        opera: Some(94 << 16),
        samsung: Some(4 << 16),
    });

    let css_string = fs::read_to_string("assets/utils.css")
        .expect("Should have been able to read string from css file");


    let styles_file_name = "utils.css";
    let styles_file_path = format!("{assets_path}/{styles_file_name}");

    let mut stylesheet = StyleSheet::parse(
        &css_string, 
        ParserOptions {
            filename: styles_file_name.to_string(),
            flags: ParserFlags::NESTING,
            ..ParserOptions::default()
        }
    ).unwrap();
      
    stylesheet.minify(MinifyOptions {
        targets,
        ..MinifyOptions::default()
    }).unwrap();
    
    let res = stylesheet.to_css(PrinterOptions {
        minify: true,
        targets,
        ..PrinterOptions::default()
    }).unwrap();

    fs::write(styles_file_path, res.code).expect("Should be able to write minified css string to file");

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

fn set_cuda_include_dir() -> Result<()> {
    // NOTE: copied from cudarc build.rs.
    let env_vars = [
        "CUDA_PATH",
        "CUDA_ROOT",
        "CUDA_TOOLKIT_ROOT_DIR",
        "CUDNN_LIB",
    ];
    let env_vars = env_vars
        .into_iter()
        .map(std::env::var)
        .filter_map(Result::ok)
        .map(Into::<PathBuf>::into);

    let roots = [
        "/usr",
        "/usr/local/cuda",
        "/opt/cuda",
        "/usr/lib/cuda",
        "C:/Program Files/NVIDIA GPU Computing Toolkit",
        "C:/CUDA",
    ];
    let roots = roots.into_iter().map(Into::<PathBuf>::into);
    let root = env_vars
        .chain(roots)
        .find(|path| path.join("include").join("cuda.h").is_file())
        .context("cannot find include/cuda.h")?;
    println!(
        "cargo:rustc-env=CUDA_INCLUDE_DIR={}",
        root.join("include").display()
    );
    Ok(())
}

#[allow(unused)]
fn compute_cap() -> Result<usize> {
    // Grab compute code from nvidia-smi
    let mut compute_cap = {
        let out = std::process::Command::new("nvidia-smi")
                    .arg("--query-gpu=compute_cap")
                    .arg("--format=csv")
                    .output()
                    .context("`nvidia-smi` failed. Ensure that you have CUDA installed and that `nvidia-smi` is in your PATH.")?;
        let out = std::str::from_utf8(&out.stdout).context("stdout is not a utf8 string")?;
        let mut lines = out.lines();
        assert_eq!(
            lines.next().context("missing line in stdout")?,
            "compute_cap"
        );
        let cap = lines
            .next()
            .context("missing line in stdout")?
            .replace('.', "");
        cap.parse::<usize>()
            .with_context(|| format!("cannot parse as int {cap}"))?
    };

    // Grab available GPU codes from nvcc and select the highest one
    let max_nvcc_code = {
        let out = std::process::Command::new("nvcc")
                    .arg("--list-gpu-code")
                    .output()
                    .expect("`nvcc` failed. Ensure that you have CUDA installed and that `nvcc` is in your PATH.");
        let out = std::str::from_utf8(&out.stdout).unwrap();

        let out = out.lines().collect::<Vec<&str>>();
        let mut codes = Vec::with_capacity(out.len());
        for code in out {
            let code = code.split('_').collect::<Vec<&str>>();
            if !code.is_empty() && code.contains(&"sm") {
                if let Ok(num) = code[1].parse::<usize>() {
                    codes.push(num);
                }
            }
        }
        codes.sort();
        if !codes.contains(&compute_cap) {
            anyhow::bail!(
                "nvcc cannot target gpu arch {compute_cap}. Available nvcc targets are {codes:?}."
            );
        }
        *codes.last().unwrap()
    };

    // If nvidia-smi compute_cap is higher than the highest gpu code from nvcc,
    // then choose the highest gpu code in nvcc
    if compute_cap > max_nvcc_code {
        println!(
            "cargo:warning=Lowering gpu arch {compute_cap} to max nvcc target {max_nvcc_code}."
        );
        compute_cap = max_nvcc_code;
    }

    println!("cargo:rerun-if-env-changed=CUDA_COMPUTE_CAP");

    if let Ok(compute_cap_str) = std::env::var("CUDA_COMPUTE_CAP") {
        compute_cap = compute_cap_str
            .parse::<usize>()
            .with_context(|| format!("cannot parse as usize '{compute_cap_str}'"))?;
        println!("cargo:warning=Using gpu arch {compute_cap} from $CUDA_COMPUTE_CAP");
    }
    println!("cargo:rustc-env=CUDA_COMPUTE_CAP=sm_{compute_cap}");
    Ok(compute_cap)
}