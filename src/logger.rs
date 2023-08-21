use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
    sync::Mutex,
};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::default());
}

pub struct Logger {
    log_files: HashMap<String, File>,
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            log_files: HashMap::default(),
        }
    }
}

impl Logger {
    pub fn log_to_file(&mut self, filepath: &str, message: &str) {
        let file = self
            .log_files
            .entry(filepath.to_string())
            .or_insert_with(|| {
                println!("LOGGER: Create file {filepath}");
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(filepath)
                    .expect("Could not open log file.")
            });

        writeln!(file, "{}", message).expect("Failed to write to log file.");
    }
}

#[macro_export]
macro_rules! log_if_enabled {
    ($filepath:expr, $($arg:tt)*) => {
        #[cfg(feature = "logging")]
        {
            crate::logger::LOGGER.lock().unwrap().log_to_file($filepath, &format!($($arg)*));
        }
    };
}
