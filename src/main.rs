use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::*;
use std::process;

struct Options {
    force: bool,
    interactive: bool,
    verbose: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        usage();
    }
    
    let mut options = Options {
        force: false,
        interactive: false,
        verbose: false,
    };
    
    let mut files = Vec::new();
    
    // 引数をパース
    for arg in args.iter().skip(1) {
        if arg.starts_with('-') {
            for ch in arg.chars().skip(1) {
                match ch {
                    'f' => options.force = true,
                    'i' => options.interactive = true,
                    'v' => options.verbose = true,
                    _ => {
                        eprintln!("rm: invalid option -- '{}'", ch);
                        usage();
                    }
                }
            }
        } else {
            files.push(arg.clone());
        }
    }
    
    if files.is_empty() {
        eprintln!("rm: missing operand");
        process::exit(1);
    }
    
    // -i と -f が同時に指定された場合、-i を優先
    if options.interactive {
        options.force = false;
    }

    let pwd = env::var("PWD").unwrap_or_else(|_| String::from("."));
    
    let mut exit_code = 0;
    
    for file in files {
        let file_path = resolve_path(&file, &pwd);
        if let Err(e) = remove_file(&file_path, &options) {
            if !options.force {
                eprintln!("rm: cannot remove '{}': {}", file, e);
                exit_code = 1;
            }
        }
    }
    
    process::exit(exit_code);
}

fn usage() -> ! {
    eprintln!("Usage: rm [OPTION]... FILE...");
    eprintln!("Remove (unlink) the FILE(s).");
    eprintln!();
    eprintln!("  -f    Attempt to remove the files without prompting for confirmation");
    eprintln!("  -i    Request confirmation before attempting to remove each file");
    eprintln!("  -v    Be verbose when deleting files, showing them as they are removed");
    process::exit(1);
}

fn resolve_path(file_path: &str, pwd: &str) -> String {
    let path = Path::new(file_path);
    
    // 絶対パスの場合はそのまま返す
    if path.is_absolute() {
        return file_path.to_string();
    }
    
    // 相対パスの場合はPWDと結合
    let mut full_path = PathBuf::from(pwd);
    full_path.push(file_path);
    
    full_path.to_string_lossy().to_string()
}

fn remove_file(path: &str, options: &Options) -> io::Result<()> {
    // ファイルが存在するか確認
    if !options.force {
        fs::metadata(path)?;
    } else {
        // -f オプション時は存在しなくてもエラーにしない
        if fs::metadata(path).is_err() {
            return Ok(());
        }
    }
    
    // インタラクティブモードで確認
    if options.interactive {
        if !confirm_removal(path)? {
            return Ok(());
        }
    }
    
    // ファイルを削除
    fs::remove_file(path)?;
    
    // 冗長モードでメッセージを表示
    if options.verbose {
        println!("removed '{}'", path);
    }
    
    Ok(())
}

fn confirm_removal(path: &str) -> io::Result<bool> {
    print!("rm: remove file '{}'? ", path);
    io::stdout().flush()?;
    
    let mut input = String::new();
    
    #[cfg(not(target_os = "wasi"))]
    {
        io::stdin().read_line(&mut input)?;
    }
    
    #[cfg(target_os = "wasi")]
    {
        // WASI環境では標準入力からの読み取りをサポート
        use std::io::Read;
        let stdin = io::stdin();
        let mut buffer = [0u8; 256];
        let bytes_read = stdin.lock().read(&mut buffer)?;
        input = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    }
    
    let response = input.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}