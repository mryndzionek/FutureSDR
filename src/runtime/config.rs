use config::Value;
#[cfg(not(target_arch = "wasm32"))]
use config::{File, Source};
use log::LevelFilter;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

pub fn config() -> &'static Config {
    &*CONFIG
}

pub fn get_value(name: &str) -> Option<Value> {
    CONFIG.misc.get(name).cloned()
}

pub fn get<T: FromStr>(name: &str) -> Option<T> {
    CONFIG
        .misc
        .get(name)
        .and_then(|v| v.clone().into_str().ok())
        .and_then(|v| v.parse::<T>().ok())
}

pub fn get_or_default<T: FromStr>(name: &str, default: T) -> T {
    get(name).unwrap_or(default)
}

#[cfg(not(target_arch = "wasm32"))]
static CONFIG: Lazy<Config> = Lazy::new(|| {
    let mut settings = ::config::Config::new();

    // user config
    if let Some(mut path) = dirs::config_dir() {
        path.push("futuresdr");
        path.push("config.toml");

        if let Err(e) = settings.merge(File::from(path.clone()).required(false)) {
            println!("user config error ({:?}): {:?}", path, e);
        }
    }

    // project config
    if let Err(e) =
        settings.merge(File::new("config.toml", config::FileFormat::Toml).required(false))
    {
        println!("project config error (config.toml): {:?}", e);
    }

    // env config
    if let Err(e) = settings.merge(config::Environment::with_prefix("futuresdr")) {
        println!("env config error: {:?}", e);
    }

    // start from default config
    let mut c = Config::default();

    if let Ok(config) = settings.collect() {
        for (k, v) in config.iter() {
            match k.as_str() {
                "queue_size" => {
                    c.queue_size = config_parse::<usize>(v);
                }
                "buffer_size" => {
                    c.buffer_size = config_parse::<usize>(v);
                }
                "log_level" => {
                    c.log_level = config_parse::<LevelFilter>(v);
                }
                "ctrlport_enable" => {
                    c.ctrlport_enable = config_parse::<bool>(v);
                }
                "ctrlport_bind" => {
                    c.ctrlport_bind = Some(config_parse::<SocketAddr>(v));
                }
                "frontend_path" => {
                    c.frontend_path = Some(config_parse::<PathBuf>(v));
                }
                _ => {
                    c.misc.insert(k.clone(), v.clone());
                }
            }
        }
    }
    if !c.validate() {
        panic!("invalid config");
    }
    c
});

#[cfg(target_arch = "wasm32")]
static CONFIG: Lazy<Config> = Lazy::new(|| Config::default());

#[derive(Debug)]
pub struct Config {
    pub queue_size: usize,
    pub buffer_size: usize,
    pub log_level: LevelFilter,
    pub ctrlport_enable: bool,
    pub ctrlport_bind: Option<SocketAddr>,
    pub frontend_path: Option<PathBuf>,
    misc: HashMap<String, Value>,
}

impl Config {
    #[cfg(not(target_arch = "wasm32"))]
    fn validate(&self) -> bool {
        if self.ctrlport_enable && self.ctrlport_bind.is_none() {
            println!("ctrlport enabled but socket not set");
            return false;
        }
        true
    }
}

impl Default for Config {
    #[cfg(debug_assertions)]
    fn default() -> Self {
        Config {
            queue_size: 8192,
            buffer_size: 32768,
            log_level: LevelFilter::Debug,
            ctrlport_enable: true,
            ctrlport_bind: "127.0.0.1:26125".parse::<SocketAddr>().ok(),
            frontend_path: None,
            misc: HashMap::new(),
        }
    }

    #[cfg(not(debug_assertions))]
    fn default() -> Self {
        Config {
            queue_size: 8192,
            buffer_size: 32768,
            log_level: LevelFilter::Info,
            ctrlport_enable: false,
            ctrlport_bind: None,
            frontend_path: None,
            misc: HashMap::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn config_parse<T: FromStr>(v: &Value) -> T {
    if let Ok(v) = v.clone().into_str() {
        if let Ok(v) = v.parse::<T>() {
            return v;
        }
    }

    println!("invalid config value {:?}", v);
    panic!();
}
