use chrono::Local;
use std::{fs::File, io::Write, panic};

use log::error;

pub fn hook() {
    panic::set_hook(Box::new(|info| {
        generate_crash_report(info);

        error!("Hako crashed!");
    }));
}

fn generate_crash_report(info: &panic::PanicHookInfo) {
    const VERSION: &str = env!("CARGO_PKG_VERSION_FULL");
    let formatted_time = Local::now().format("%Y_%m_%d-%H:%M:%S").to_string();
    let filename = format!("crash_report_{}.log", formatted_time);

    if let Ok(mut file) = File::create(&filename) {
        let _ = writeln!(file, "Hako v{} Crash report", VERSION);
        let _ = writeln!(file, "{}\n", formatted_time);

        if let Some(s) = info.payload().downcast_ref::<&str>() {
            let _ = writeln!(file, "Cause: {}", s);
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            let _ = writeln!(file, "Cause: {}", s);
        }
        if let Some(location) = info.location() {
            let _ = writeln!(
                file,
                "Location: {}:{}:{}\n",
                location.file(),
                location.line(),
                location.column()
            );
        }

        let _ = writeln!(file, "No Backtrace :D\n");

        let _ = writeln!(
            file,
            "Please report this issue at: https://github.com/bilirumble/Hako/issues"
        );
    }
}
