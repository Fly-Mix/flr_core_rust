use glob::glob;
use serde_yaml::Value;
use std::{collections::HashMap, fs::File, io::Read, path::Path};
extern crate notify;
use super::flr::FLR;
use notify::{FsEventWatcher, RecursiveMode, Watcher};

#[derive(Debug)]
pub struct FlutterProject {
    pub path: String,
    pub name: String,
    pub flr: Option<FLR>,
    watchers: HashMap<String, FsEventWatcher>,
}
// Instance
impl FlutterProject {
    pub fn toggle_watch(&mut self, to_watch: bool) {
        if let Some(flr_value) = &self.flr {
            let paths = flr_value.asset_root_path();

            for ele in paths {
                let mut str = self.path.to_owned();
                str.push_str(&ele);
                if !to_watch {
                    if let Some(mut wc) = self.watchers.remove(&ele) {
                        _ = wc.unwatch(Path::new(&str));
                    }
                } else {
                    if let Ok(mut watcher) = notify::recommended_watcher(|res| match res {
                        Ok(event) => println!("event: {:?}", event),
                        Err(e) => println!("watch error: {:?}", e),
                    }) {
                        _ = watcher.watch(Path::new(&str), RecursiveMode::Recursive);
                        println!("watch: {:?}", &str);
                        self.watchers.insert(str, watcher);
                    }
                }
            }
        }
    }
}
// Static
impl FlutterProject {
    pub fn get_all_project() -> std::io::Result<Vec<FlutterProject>> {
        let mut v: Vec<FlutterProject> = vec![];

        for entry in glob("**/pubspec.yaml") {
            entry.for_each(|item| {
                if let Ok(path) = item {
                    let raw_path = path.as_path();
                    if let Ok(mut file) = File::open(raw_path) {
                        let mut content = String::new();
                        _ = file.read_to_string(&mut content);
                        if let Ok(value) = serde_yaml::from_str::<Value>(&content) {
                            if let (Some(name), Some(p)) =
                                (value["name"].as_str(), raw_path.to_str())
                            {
                                let flr = value["flr"].as_mapping();
                                let paths = p.split("/");
                                let mut vec1: Vec<&str> = paths.collect();
                                vec1.remove(vec1.len() - 1);
                                let project = FlutterProject {
                                    path: vec1.join("/") + "/",
                                    name: name.to_string(),
                                    flr: FLR::from(flr),
                                    watchers: HashMap::new(),
                                };
                                v.push(project.into());
                            }
                        }
                    }
                }
            })
        }

        return Ok(v);
    }
}

//
#[cfg(test)]
mod tests {
    use super::FlutterProject;

    #[test]
    fn list_all_pubspec() {
        if let Ok(mut list) = FlutterProject::get_all_project() {
            println!("{:?}, {:?}", list.len(), list);
            let mut first = list.remove(0);
            first.toggle_watch(true);
            list.insert(0, first);
            loop {}
        }
    }
}
