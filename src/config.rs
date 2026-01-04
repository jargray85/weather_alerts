use dotenv::dotenv;

pub fn load_env_file() {
    const LOG_PATH: &str = "/tmp/weather_alerts.log";
    let mut log_msg = String::new();
    
    // Try to find .env file in multiple locations for packaged apps
    let mut env_paths = vec![".env".to_string()];
    
    // For packaged macOS apps, the executable is in .app/Contents/MacOS/
    // We want to look in:
    // 1. .app/Contents/Resources/.env
    // 2. .app/Contents/MacOS/.env
    // 3. Parent of .app bundle/.env
    // 4. Home directory
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_path_str = exe_path.to_string_lossy().to_string();
        log_msg.push_str(&format!("Executable path: {}\n", exe_path_str));
        
        // Try Contents/Resources (standard location for app resources)
        // This is where we bundle the .env file during build
        if let Some(macos_dir) = exe_path.parent() {
            if let Some(contents_dir) = macos_dir.parent() {
                let resources_env = contents_dir.join("Resources").join(".env");
                if let Some(path_str) = resources_env.to_str() {
                    env_paths.insert(1, path_str.to_string()); // Prioritize bundled .env
                }
            }
        }
        
        // Try Contents/MacOS (where the executable is)
        if let Some(macos_dir) = exe_path.parent() {
            let macos_env = macos_dir.join(".env");
            if let Some(path_str) = macos_env.to_str() {
                env_paths.push(path_str.to_string());
            }
        }
        
        // Try parent of .app bundle (if app is in a folder)
        // Go up from MacOS -> Contents -> .app -> parent
        if let Some(macos_dir) = exe_path.parent() {
            if let Some(contents_dir) = macos_dir.parent() {
                if let Some(app_bundle) = contents_dir.parent() {
                    if let Some(bundle_parent) = app_bundle.parent() {
                        let bundle_parent_env = bundle_parent.join(".env");
                        if let Some(path_str) = bundle_parent_env.to_str() {
                            env_paths.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // Add .env in home directory
    if let Ok(home) = std::env::var("HOME") {
        env_paths.push(format!("{}/.weather_alerts.env", home));
        env_paths.push(format!("{}/.env", home));
    }
    
    // Also check the project directory (for development)
    // This won't work for packaged apps, but helpful for debugging
    if let Ok(home) = std::env::var("HOME") {
        let project_env = format!("{}/Desktop/weather_alerts/.env", home);
        env_paths.push(project_env);
    }
    
    // Try to load .env from various locations
    let mut loaded = false;
    log_msg.push_str("Searching for .env file in these locations:\n");
    for path in &env_paths {
        log_msg.push_str(&format!("  Checking: {}\n", path));
        if std::path::Path::new(path).exists() {
            log_msg.push_str(&format!("  ✓ Found .env file at: {}\n", path));
            match dotenv::from_path(path) {
                Ok(_) => {
                    log_msg.push_str(&format!("  ✓ Successfully loaded .env from: {}\n", path));
                    loaded = true;
                    // Verify the key was loaded
                    if let Ok(key) = std::env::var("OPENWEATHERMAP_API_KEY") {
                        log_msg.push_str(&format!("  ✓ API key found (length: {})\n", key.len()));
                    } else {
                        log_msg.push_str("  ✗ API key not found in loaded .env file\n");
                    }
                    break;
                }
                Err(e) => {
                    log_msg.push_str(&format!("  ✗ Error loading .env from {}: {}\n", path, e));
                }
            }
        } else {
            log_msg.push_str(&format!("  ✗ Not found: {}\n", path));
        }
    }
    
    if !loaded {
        log_msg.push_str(&format!("⚠ No .env file found in any of these locations: {:?}\n", env_paths));
        // Also try the default dotenv() which looks in current directory
        dotenv().ok();
        if let Ok(key) = std::env::var("OPENWEATHERMAP_API_KEY") {
            log_msg.push_str(&format!("✓ API key found via default dotenv() (length: {})\n", key.len()));
        } else {
            log_msg.push_str("✗ API key still not found after default dotenv()\n");
        }
    }
    
    // Write to log file
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(log_msg.as_bytes())
        });
    
    // Also print to stderr for immediate visibility
    eprintln!("{}", log_msg);
}

