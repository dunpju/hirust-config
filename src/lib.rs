use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::sync::{LazyLock, Mutex};
use walkdir::WalkDir;

static CONFIG_COLLECT: LazyLock<Mutex<BTreeMap<String, BTreeMap<String, serde_yml::Value>>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));

#[cfg(test)]
mod tests {
    use super::*;

    // cargo test run -- --show-output
    #[test]
    fn run() {
        load_config("./env");
        print(|key, value| {
            println!("{:?}: {:?}", key, value);
        });
        let version = get::<String>("env.config.app.SSL.OUT");
        println!("{} {:?}", line!(), version);
    }
}

#[allow(unused)]
pub fn load_config(config_dir: &str) {
    for entry in WalkDir::new(config_dir) {
        let entry = entry.unwrap();
        let file_path = entry.path();
        let extension = file_path.extension().and_then(|s| s.to_str());

        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap();
        // 检查扩展名是否为".yaml"
        if file_path.is_file() && extension == Some("yaml") {
            let mut keys: Vec<String> = vec![];
            for p in file_path.iter() {
                if p.ne(".") {
                    if p.ne(file_name) {
                        keys.push(format!("{:?}", p).replace("\"", ""));
                    } else {
                        keys.push(format!("{:?}", p).replace(".yaml", "").replace("\"", ""));
                    }
                }
            }

            let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap();
            let file_name = file_name.replace(".yaml", "");
            let yaml_content =
                fs::read_to_string(file_path).expect(format!("读取{:?}失败", file_path).as_str());

            // 解析YAML内容到BTreeMap中，自动保持顺序
            let deserialized_map: BTreeMap<String, serde_yml::Value> = serde_yml::from_str(&yaml_content).unwrap();

            CONFIG_COLLECT
                .lock()
                .unwrap()
                .insert(String::from(keys.join(".")), deserialized_map.clone());
        }
    }
}

type PrintFn = fn(key: &String, value: &BTreeMap<String, serde_yml::Value>);

#[allow(unused)]
pub fn print(f: PrintFn) {
    for (k, v) in CONFIG_COLLECT.lock().unwrap().iter() {
        f(k, v);
    }
}

#[allow(unused)]
pub fn get<'a, T: Deserialize<'a>>(key: &str) -> Option<T> {
    let mut result = None;
    if key.contains(".") {
        let keys = key.split(".").collect::<Vec<&str>>();
        let mut config_collect: Option<BTreeMap<String, serde_yml::Value>> = None;
        let mut value: Option<serde_yml::Value> = None;
        let mut mapping: Option<serde_yml::Mapping> = None;

        let mut temp_keys: Vec<String> = vec![];

        for key in keys {
            result = None;

            temp_keys.push(key.to_string());

            if config_collect.is_none() {
                if let Some(bm) = CONFIG_COLLECT.lock().unwrap().get(&temp_keys.join(".").to_string()){
                    config_collect = Some(bm.clone());
                }
            } else {
                if let Some(m) = &mapping {
                    value = m.get(&key.to_string()).cloned();
                    if value.is_some() {
                        if value.clone().unwrap().is_mapping() {
                            mapping = value.clone().unwrap().as_mapping().cloned();
                            value = None;
                        } else {
                            result = value.clone().and_then(|v| T::deserialize(v).ok());
                        }
                    } else {
                        result = value.clone().and_then(|v| T::deserialize(v).ok());
                        value = None;
                    }
                } else if let Some(vv) = value.clone() {
                    if vv.is_mapping() {
                        match vv.as_mapping() {
                            Some(m) => {
                                mapping = Some(m.clone());
                                value = None;
                            }
                            None => (),
                        }
                    } else {
                        result = value.clone().and_then(|v| T::deserialize(v).ok());
                        value = None;
                    }
                } else {
                    value = config_collect.clone().unwrap().get(&key.to_string()).cloned();
                    if let Some(vv) = value.clone() {
                        if vv.is_mapping() {
                            match vv.as_mapping() {
                                Some(m) => {
                                    mapping = Some(m.clone());
                                    value = None;
                                }
                                None => (),
                            }
                        } else {
                            result = value.clone().and_then(|v| T::deserialize(v).ok());
                            value = None;
                        }
                    }
                }
            }
        }
    }
    result
}
