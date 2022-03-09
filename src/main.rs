#![windows_subsystem = "windows"]

use std::path::PathBuf;
use crate::idle_windows::get_idle_time;
use core::time::Duration;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs::File;
use std::path::Path;
use std::process::Child;
use std::process::Stdio;

use error_chain::error_chain;

pub mod console;
pub mod idle_windows;
pub mod tray;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Toml(toml::de::Error);

    }
}

fn main() {
    console::alloc();
    console::hide_console();

    std::thread::spawn(move || {
        let mut program = Idlerunner::default();
        let result = program.run();
        match result {
            Err(err) => {
                use error_chain::ChainedError;
                println!("main, run: {}", err.display_chain());
            }
            Ok(_) => {}
        }
    });

    tray::tray_main();
}

// Structure of .toml file
#[derive(Serialize, Deserialize)]
struct IdlerConfig {
    path: String,
    cmd: String,
    args: Vec<String>,
    run_after_seconds: Option<u64>,
}

#[derive(Default)]
struct Idlerunner {
    last_idle_seconds: u64,
    config: Option<IdlerConfig>,
}

impl Idlerunner {
    fn read_config() -> Result<IdlerConfig> {
        let mut contents = vec![];

        {
            let mut file = File::open("Idlerunner.toml")?;
            use std::io::prelude::*;
            file.read_to_end(&mut contents)?;
        }

        let config: IdlerConfig = toml::de::from_slice(&contents)?;

        

        Ok(config)
    }

    fn test_config(&self) {
        let config = self.config.as_ref().unwrap();

        let mut path = PathBuf::new();
        path.push(&config.path);
        path.push(&config.cmd);

        let path = path.as_path();
        if !path.exists() {
            println!("warn: file at path does not exist: {:?}", path);
        }else{
            println!("ok: file exists at: {:?}", path);
        }
    }

    fn load_config(&mut self) -> Result<()> {
        self.config = Some(Self::read_config().chain_err(|| "read_config")?);
        self.test_config();
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        self.load_config()?;

        // Run after how many seconds
        let threshold = self
            .config
            .as_ref()
            .unwrap()
            .run_after_seconds
            .unwrap_or(20);

        for _i in 0..999999999 {
            self.check_idle()?;

            std::thread::sleep(Duration::from_millis(2000));

            let s = self.last_idle_seconds;

            // If it should start
            if s > threshold {
                println!("spawn_task");
                let mut child = self.spawn_task()?;

                //child.wait().chain_err(|| "wait")?;

                loop {
                    let changed = self.check_idle()?;

                    if changed {
                        println!("killing");
                        child.kill().chain_err(|| "kill")?;
                    }
                    std::thread::sleep(Duration::from_millis(500));

                    let exit_status = child.try_wait().chain_err(|| "try_wait")?;
                    if let Some(_status) = exit_status {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    // Returns if state of idle has changed
    fn check_idle(&mut self) -> Result<bool> {
        let idle = get_idle_time().chain_err(|| "get_time")?;
        let idle_seconds = idle.as_secs();
        let changed = idle_seconds < self.last_idle_seconds;

        //println!("idle: {}s", idle_seconds);

        self.last_idle_seconds = idle_seconds;

        Ok(changed)
    }

    fn spawn_task(&mut self) -> Result<Child> {
        let config = self.config.as_ref().ok_or("no config loaded")?;
        use std::process::Command;

        let root = Path::new(&config.path);
        std::env::set_current_dir(&root).chain_err(|| "set_current_dir")?;

        let child = Command::new(&config.cmd)
            .args(&config.args)
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn()
            .chain_err(|| "spawn: can't spawn the process")?;

        Ok(child)
    }
}
