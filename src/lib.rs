use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::sync::Mutex;
use walkdir::WalkDir;

lazy_static! {
    static ref CONFIG_COLLECT: Mutex<BTreeMap<String, BTreeMap<String, serde_yml::Value>>> =
        Mutex::new(BTreeMap::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    // cargo test run -- --show-output
    #[test]
    fn run() {
        load_config("./env".to_string());
        print(|key, value| {
            println!("{:?}: {:?}", key, value);
        });
        let version = get::<bool>("VERSION".to_string());
        println!("{:?}", version);
    }
}

#[allow(unused)]
pub fn load_config(config_dir: String) {
    for entry in WalkDir::new(config_dir) {
        let entry = entry.unwrap();
        let file_path = entry.path();
        let extension = file_path.extension().and_then(|s| s.to_str());

        // 检查扩展名是否为".yaml"
        if file_path.is_file() && extension == Some("yaml") {
            println!("{:?}", file_path);
            let yaml_content =
                fs::read_to_string(file_path).expect(format!("读取{:?}失败", file_path).as_str());

            // 解析YAML内容到BTreeMap中，自动保持顺序
            let deserialized_map: BTreeMap<String, serde_yml::Value> =
                serde_yml::from_str(&yaml_content).unwrap();

            CONFIG_COLLECT
                .lock()
                .unwrap()
                .insert(String::from(file_path.clone().to_str()), deserialized_map.clone());
        }
    }
}

type PrintFn = fn(key: &String, value: &serde_yml::Value);

#[allow(unused)]
pub fn print(f: PrintFn) {
    for (k, v) in CONFIG_COLLECT.lock().unwrap().iter() {
        f(k, v);
    }
}

#[allow(unused)]
pub fn get<'a, T: Deserialize<'a>>(key: String) -> Option<T> {
    let value: Option<serde_yml::Value> = CONFIG_COLLECT.lock().unwrap().get(&key).cloned();
    value.and_then(|v| T::deserialize(v).ok())
}
