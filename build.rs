use std::process::Command;

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let full_version = format!("{}-{}", &version, &git_hash);
    // let description = std::env::var("CARGO_PKG_DESCRIPTION").unwrap_or("".to_string());

    println!("cargo:rustc-env=CARGO_PKG_VERSION_FULL={}", full_version);

    #[cfg(target_os = "windows")]
    {
        println!("enabled winres");
        use winres::WindowsResource;

        WindowsResource::new()
            .set("ProductName", "Hako")
            .set("FileDescription", "Hako")
            .set("ProductVersion", full_version.as_str())
            .set("LegalCopyright", "Copyright (c) 2025 bilirumble")
            .compile()
            .expect("Failed to compile resources");
    }
}
