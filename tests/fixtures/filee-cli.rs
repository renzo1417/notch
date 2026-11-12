//! Deterministic CLI contract fixture; never included in the shipped app.
use std::{env, fs, path::Path, thread, time::Duration};
fn json(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("version") => println!("filee fixture"),
        Some("formats") if args.get(1).map(String::as_str) == Some("--json") => {
            println!("{{\"formats\":[{{\"id\":\"png\"}}]}}")
        }
        Some("formats") => {
            println!("{{\"targets\":[\"jpg\",\"png\",\"pdf\"],\"withEngines\":[\"docx\"]}}")
        }
        Some("--child") => {
            fs::write(&args[1], std::process::id().to_string()).unwrap();
            thread::sleep(Duration::from_secs(30));
        }
        Some("convert") => {
            let source = Path::new(&args[1]);
            let folder = Path::new(&args[args.iter().position(|s| s == "-o").unwrap() + 1]);
            fs::create_dir_all(folder).unwrap();
            let output = folder.join("result.png");
            fs::write(&output, b"fixture result").unwrap();
            if source.file_stem().unwrap() == "slow" {
                let _child = std::process::Command::new(env::current_exe().unwrap())
                    .args(["--child", &folder.join("child.pid").to_string_lossy()])
                    .spawn()
                    .unwrap();
                thread::sleep(Duration::from_secs(30));
            }
            if source.file_stem().unwrap() == "noisy" {
                print!("{}", "x".repeat(2 * 1024 * 1024));
                return;
            }
            if source.file_stem().unwrap() == "memory" {
                let mut chunks = Vec::new();
                for _ in 0..100 {
                    let mut chunk = Vec::new();
                    if chunk.try_reserve_exact(16 * 1024 * 1024).is_err() {
                        eprintln!("OutOfMemory");
                        thread::sleep(Duration::from_millis(100));
                        return;
                    }
                    chunk.resize(16 * 1024 * 1024, 1u8);
                    chunks.push(chunk);
                }
                return;
            }
            eprintln!("{{\"event\":\"progress\",\"fraction\":0.5}}");
            thread::sleep(Duration::from_millis(50));
            println!("{{\"converted\":1,\"files\":[{{\"source\":{},\"state\":\"done\",\"outputs\":[{}]}}]}}", json(&args[1]), json(&output.to_string_lossy()));
        }
        _ => std::process::exit(2),
    }
}
