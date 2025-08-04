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
        let version = get::<String>("app.SSL.OUT".to_string());
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
            let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap();
            let file_name = file_name.replace(".yaml", "");
            let yaml_content =
                fs::read_to_string(file_path).expect(format!("读取{:?}失败", file_path).as_str());

            // 解析YAML内容到BTreeMap中，自动保持顺序
            let deserialized_map: BTreeMap<String, serde_yml::Value> =
                serde_yml::from_str(&yaml_content).unwrap();

            CONFIG_COLLECT
                .lock()
                .unwrap()
                .insert(String::from(file_name), deserialized_map.clone());
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
pub fn get<'a, T: Deserialize<'a>>(key: String) -> Option<T> {
    if key.contains(".") {
        let keys = key.split(".").collect::<Vec<&str>>();
        let mut config_collect: Mutex<BTreeMap<String, serde_yml::Value>> =
            Mutex::new(BTreeMap::new());
        let mut value: Option<serde_yml::Value> = None;
        let mut mapping: Option<serde_yml::Mapping> = None;
        let mut i = 0;
        for key in keys {
            println!("{:?}: {:?}", &key, i);
            if i == 0 {
                config_collect = Mutex::new(
                    CONFIG_COLLECT
                        .lock()
                        .unwrap()
                        .get(&key.to_string())
                        .cloned()
                        .unwrap(),
                );
            } else if let Some(m) = &mapping {
                value = m.get(&key.to_string()).cloned();
                println!("{} value {:?}", line!(), &value);
                if value.is_some() {
                    if value.clone().unwrap().is_mapping() {
                        mapping = value.clone().unwrap().as_mapping().cloned();
                    } else {
                        mapping = None;
                        return value.and_then(|v| T::deserialize(v).ok());
                    }
                } else {
                    mapping = None;
                    return value.and_then(|v| T::deserialize(v).ok());
                }
            } else if let Some(vv) = value.clone() {
                if vv.is_mapping() {
                    match vv.as_mapping() {
                        Some(m) => {
                            mapping = Some(m.clone());
                            value = None;
                        }
                        None => return None,
                    }
                } else {
                    return value.and_then(|v| T::deserialize(v).ok());
                }
            } else {
                value = config_collect
                    .lock()
                    .unwrap()
                    .get(&key.to_string())
                    .cloned();
                println!("{} value {:?}", line!(), &value);
                if let Some(vv) = value.clone() {
                    if vv.is_mapping() {
                        match vv.as_mapping() {
                            Some(m) => {
                                mapping = Some(m.clone());
                                value = None;
                            }
                            None => return None,
                        }
                    } else {
                        return value.and_then(|v| T::deserialize(v).ok());
                    }
                } else {
                    return None;
                }
            }
            i += 1;
        }
        None
    } else {
        None
    }
}
