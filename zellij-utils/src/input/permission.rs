use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::{
    consts::ZELLIJ_PLUGIN_PERMISSIONS_CACHE, data::PermissionType, input::layout::RunPluginLocation,
};

pub type GrantedPermission = HashMap<String, Vec<PermissionType>>;

#[derive(Default, Debug)]
pub struct PermissionCache {
    path: PathBuf,
    granted: GrantedPermission,
}

impl PermissionCache {
    pub fn cache(&mut self, plugin_name: String, permissions: Vec<PermissionType>) {
        for plugin_alias in plugin_name_aliases(&plugin_name) {
            self.granted.insert(plugin_alias, permissions.clone());
        }
    }

    pub fn get_permissions(&self, plugin_name: String) -> Option<&Vec<PermissionType>> {
        for plugin_alias in plugin_name_aliases(&plugin_name) {
            if let Some(permissions) = self.granted.get(&plugin_alias) {
                return Some(permissions);
            }
        }
        None
    }

    pub fn check_permissions(
        &self,
        plugin_name: String,
        permissions_to_check: &Vec<PermissionType>,
    ) -> bool {
        if let Some(target) = self.get_permissions(plugin_name) {
            let mut all_granted = true;
            for permission in permissions_to_check {
                if !target.contains(permission) {
                    all_granted = false;
                }
            }
            return all_granted;
        }

        false
    }

    pub fn from_path_or_default(cache_path: Option<PathBuf>) -> Self {
        let cache_path = cache_path.unwrap_or(ZELLIJ_PLUGIN_PERMISSIONS_CACHE.to_path_buf());

        let granted = match fs::read_to_string(cache_path.clone()) {
            Ok(raw_string) => PermissionCache::from_string(raw_string).unwrap_or_default(),
            Err(e) => {
                log::error!("Failed to read permission cache file: {}", e);
                GrantedPermission::default()
            },
        };

        PermissionCache {
            path: cache_path,
            granted,
        }
    }

    pub fn write_to_file(&self) -> std::io::Result<()> {
        let mut f = File::create(&self.path)?;
        write!(f, "{}", PermissionCache::to_string(&self.granted))?;
        Ok(())
    }
}

fn plugin_name_aliases(plugin_name: &str) -> Vec<String> {
    let plugin_name = plugin_name.trim();
    let mut aliases = vec![];
    push_unique(&mut aliases, plugin_name.to_owned());

    if let Ok(plugin_location) = RunPluginLocation::parse(plugin_name, None) {
        match plugin_location {
            RunPluginLocation::File(path) => {
                for alias in file_location_aliases(path) {
                    push_unique(&mut aliases, alias);
                }
            },
            RunPluginLocation::Zellij(tag) => {
                push_unique(&mut aliases, tag.to_string());
                push_unique(&mut aliases, format!("zellij:{}", tag));
            },
            RunPluginLocation::Remote(url) => {
                push_unique(&mut aliases, url.clone());
                if let Ok(parsed_url) = url::Url::parse(&url) {
                    push_unique(&mut aliases, parsed_url.to_string());
                }
            },
        }
        return aliases;
    }

    if !plugin_name.contains(':') && !plugin_name.contains('/') && !plugin_name.contains('\\') {
        // Builtin plugin aliases are sometimes represented as "tag" and sometimes "zellij:tag".
        push_unique(&mut aliases, format!("zellij:{}", plugin_name));
    }

    let expanded = shellexpand::full(plugin_name)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| plugin_name.to_owned());
    let path = PathBuf::from(expanded);
    if path.is_absolute() {
        for alias in file_location_aliases(path) {
            push_unique(&mut aliases, alias);
        }
    }

    aliases
}

fn file_location_aliases(path: PathBuf) -> Vec<String> {
    let mut aliases = vec![];
    let path_as_string = path.to_string_lossy().to_string();
    push_unique(&mut aliases, path_as_string.clone());
    push_unique(&mut aliases, format!("file:{}", path_as_string));

    if path_as_string.starts_with('/') {
        push_unique(&mut aliases, format!("file://{}", path_as_string));
        push_unique(
            &mut aliases,
            format!("file:///{}", path_as_string.trim_start_matches('/')),
        );
    }

    aliases
}

fn push_unique(aliases: &mut Vec<String>, candidate: String) {
    if !aliases.contains(&candidate) {
        aliases.push(candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_cache_matches_file_url_and_path_variants() {
        let mut cache = PermissionCache::default();
        cache.cache(
            "/tmp/example-plugin.wasm".to_owned(),
            vec![PermissionType::ReadApplicationState],
        );

        assert!(cache.check_permissions(
            "file:/tmp/example-plugin.wasm".to_owned(),
            &vec![PermissionType::ReadApplicationState],
        ));
        assert!(cache.check_permissions(
            "file:///tmp/example-plugin.wasm".to_owned(),
            &vec![PermissionType::ReadApplicationState],
        ));
    }

    #[test]
    fn permission_cache_matches_zellij_tag_variants() {
        let mut cache = PermissionCache::default();
        cache.cache(
            "zellij:status-bar".to_owned(),
            vec![PermissionType::ReadApplicationState],
        );

        assert!(cache.check_permissions(
            "status-bar".to_owned(),
            &vec![PermissionType::ReadApplicationState],
        ));
    }
}
