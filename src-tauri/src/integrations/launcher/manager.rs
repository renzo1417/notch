use super::*;

impl LaunchableApp {
    pub fn launch_target(&self) -> LaunchTarget {
        self.target.clone().unwrap_or_else(|| {
            if self.executable_path.to_ascii_lowercase().ends_with(".lnk") {
                LaunchTarget::Shortcut {
                    path: self.executable_path.clone(),
                }
            } else {
                LaunchTarget::Executable {
                    path: self.executable_path.clone(),
                    args: self.args.clone(),
                    working_directory: None,
                }
            }
        })
    }
}

impl LauncherManager {
    pub fn new() -> Self {
        Self {
            apps: Mutex::new(Vec::new()),
            storage_path: Mutex::new(None),
            storage_error: Mutex::new(None),
            catalog: Mutex::new(None),
            icons: Mutex::new(HashMap::new()),
        }
    }

    pub fn init(&self, data_dir: PathBuf) {
        let _ = std::fs::create_dir_all(&data_dir);
        let path = data_dir.join("launcher_apps.json");
        *self.storage_path.lock().unwrap() = Some(path.clone());
        *self.storage_error.lock().unwrap() = None;
        let loaded = match std::fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str::<Vec<LaunchableApp>>(&contents)
                .map(Some)
                .map_err(|_| ()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(()),
        };
        match loaded {
            Ok(Some(mut apps)) => {
                for app in &mut apps {
                    app.exists = catalog::target_exists(&app.launch_target());
                }
                apps.sort_by_key(|app| app.order);
                *self.apps.lock().unwrap() = apps;
            }
            Ok(None) => {}
            Err(()) => {
                *self.storage_error.lock().unwrap() = Some("Saved apps could not be read. The existing file was left unchanged. Restore a valid launcher_apps.json and restart before changing apps.".into());
            }
        }
    }

    fn save_to_disk(&self, apps: &[LaunchableApp]) -> Result<(), String> {
        if let Some(error) = self.storage_error.lock().unwrap().as_ref() {
            return Err(error.clone());
        }
        if let Some(path) = self.storage_path.lock().unwrap().as_ref() {
            let contents = serde_json::to_vec_pretty(apps).map_err(|error| error.to_string())?;
            let temporary = path.with_extension("json.tmp");
            std::fs::write(&temporary, contents)
                .map_err(|error| format!("Couldn’t save apps: {error}"))?;
            std::fs::rename(&temporary, path)
                .map_err(|error| format!("Couldn’t replace saved apps: {error}"))?;
        } else {
            return Err(
                "App storage is unavailable. Restart the app before changing saved apps.".into(),
            );
        }
        Ok(())
    }

    pub fn get_apps(&self) -> Vec<LaunchableApp> {
        let mut apps = self.apps.lock().unwrap().clone();
        for app in &mut apps {
            app.exists = catalog::target_exists(&app.launch_target());
        }
        apps.sort_by_key(|app| app.order);
        apps
    }

    pub fn inspect_file(&self, path: &str) -> Result<InspectedApp, String> {
        let file = Path::new(path);
        if !file.is_file() {
            return Err("Selected application was not found".into());
        }
        let shortcut = file
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"));
        let executable = file
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
        if !shortcut && !executable {
            return Err("Choose a Windows executable (.exe) or shortcut (.lnk)".into());
        }
        let target = if shortcut {
            LaunchTarget::Shortcut { path: path.into() }
        } else {
            LaunchTarget::Executable {
                path: path.into(),
                args: Vec::new(),
                working_directory: None,
            }
        };
        Ok(InspectedApp {
            name: file
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("App")
                .into(),
            executable_path: path.into(),
            args: Vec::new(),
            icon_data_url: extract_icon_data_url(path),
            is_shortcut: shortcut,
            target,
        })
    }

    pub fn add_app(&self, mut app: LaunchableApp) -> Result<LaunchableApp, String> {
        catalog::validate_target(&app.launch_target())?;
        app.exists = catalog::target_exists(&app.launch_target());
        let mut apps = self.apps.lock().unwrap();
        if let Some(existing) = apps
            .iter()
            .find(|existing| existing.launch_target().identity() == app.launch_target().identity())
        {
            return Ok(existing.clone());
        }
        app.order = apps.len() as u32;
        let mut updated = apps.clone();
        updated.push(app.clone());
        self.save_to_disk(&updated)?;
        *apps = updated;
        Ok(app)
    }

    pub fn update_app(&self, mut app: LaunchableApp) -> Result<LaunchableApp, String> {
        catalog::validate_target(&app.launch_target())?;
        app.exists = catalog::target_exists(&app.launch_target());
        let mut apps = self.apps.lock().unwrap();
        let position = apps
            .iter()
            .position(|entry| entry.id == app.id)
            .ok_or("App not found")?;
        if apps.iter().any(|existing| {
            existing.id != app.id
                && existing.launch_target().identity() == app.launch_target().identity()
        }) {
            return Err("That application is already added".into());
        }
        let mut updated = apps.clone();
        updated[position] = app.clone();
        self.save_to_disk(&updated)?;
        *apps = updated;
        Ok(app)
    }

    pub fn remove_app(&self, id: &str) -> Result<(), String> {
        let mut apps = self.apps.lock().unwrap();
        if !apps.iter().any(|app| app.id == id) {
            return Err("App not found".into());
        }
        let mut updated: Vec<_> = apps.iter().filter(|app| app.id != id).cloned().collect();
        for (index, app) in updated.iter_mut().enumerate() {
            app.order = index as u32;
        }
        self.save_to_disk(&updated)?;
        *apps = updated;
        Ok(())
    }

    pub fn reorder_apps(&self, ids: &[String]) -> Result<Vec<LaunchableApp>, String> {
        let mut apps = self.apps.lock().unwrap();
        let mut updated = apps.clone();
        for (index, id) in ids.iter().enumerate() {
            if let Some(app) = updated.iter_mut().find(|app| &app.id == id) {
                app.order = index as u32;
            }
        }
        updated.sort_by_key(|app| app.order);
        self.save_to_disk(&updated)?;
        *apps = updated.clone();
        Ok(updated)
    }

    pub fn locate_app(&self, id: &str, path: &str) -> Result<LaunchableApp, String> {
        let inspected = self.inspect_file(path)?;
        let mut app = self
            .apps
            .lock()
            .unwrap()
            .iter()
            .find(|app| app.id == id)
            .cloned()
            .ok_or("App not found")?;
        let existing_target = app.launch_target();
        // Keep explicit arguments when relocating an executable; shortcuts own their launch arguments.
        app.target = Some(match inspected.target {
            LaunchTarget::Executable { path, .. } => {
                let (args, working_directory) = match existing_target {
                    LaunchTarget::Executable {
                        args,
                        working_directory,
                        ..
                    } => (args, working_directory),
                    _ => (app.args.clone(), None),
                };
                app.args = args.clone();
                LaunchTarget::Executable {
                    path,
                    args,
                    working_directory,
                }
            }
            target => target,
        });
        app.executable_path = inspected.executable_path;
        app.icon_data_url = inspected.icon_data_url;
        self.update_app(app)
    }

    pub fn launch_app(&self, id: &str) -> Result<(), String> {
        let app = self
            .apps
            .lock()
            .unwrap()
            .iter()
            .find(|app| app.id == id)
            .cloned()
            .ok_or("App not found")?;
        let target = app.launch_target();
        if !catalog::target_exists(&target) {
            return Err(
                "AppNotFound: This application is no longer available. Replace or remove it."
                    .into(),
            );
        }
        catalog::launch_target(&target)
    }

    pub fn list_installed_apps(&self, refresh: bool) -> Result<InstalledAppCatalog, String> {
        let mut cached = self.catalog.lock().unwrap();
        if refresh || cached.is_none() {
            *cached = Some(catalog::discover_installed_apps()?);
            self.icons.lock().unwrap().clear();
        }
        Ok(cached.as_ref().unwrap().clone())
    }

    pub fn installed_app_icon(&self, id: &str) -> Result<Option<String>, String> {
        if let Some(icon) = self.icons.lock().unwrap().get(id) {
            return Ok(icon.clone());
        }
        let app = self
            .catalog
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|catalog| catalog.apps.iter().find(|app| app.id == id))
            .cloned()
            .ok_or("Application is no longer in the catalog. Refresh and try again.")?;
        let icon = catalog::target_icon(&app.target);
        self.icons.lock().unwrap().insert(id.into(), icon.clone());
        Ok(icon)
    }

    pub fn pin_installed_app(
        &self,
        id: &str,
        replace_id: Option<&str>,
    ) -> Result<LaunchableApp, String> {
        let entry = self
            .catalog
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|catalog| catalog.apps.iter().find(|app| app.id == id))
            .cloned()
            .ok_or("Application is no longer in the catalog. Refresh and try again.")?;
        if !catalog::target_exists(&entry.target) {
            return Err("This application is no longer installed. Refresh the list.".into());
        }
        let icon = self.installed_app_icon(id)?;
        if let Some(replace_id) = replace_id {
            let mut app = self
                .apps
                .lock()
                .unwrap()
                .iter()
                .find(|app| app.id == replace_id)
                .cloned()
                .ok_or("App to replace was not found")?;
            app.target = Some(entry.target.clone());
            app.executable_path = entry.target.file_path().unwrap_or("").into();
            app.args = Vec::new();
            app.icon_data_url = icon;
            return self.update_app(app);
        }
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        self.add_app(LaunchableApp {
            id: format!("app_{time}"),
            name: entry.name,
            executable_path: entry.target.file_path().unwrap_or("").into(),
            args: Vec::new(),
            target: Some(entry.target),
            icon_path: None,
            icon_data_url: icon,
            enabled: true,
            order: 0,
            exists: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_pins_keep_arguments_and_file_format() {
        let old = r#"[{"id":"legacy","name":"Editor","executablePath":"C:\\Editor.exe","args":["--profile","Work profile"],"iconPath":null,"iconDataUrl":null,"enabled":false,"order":4,"exists":true}]"#;
        let apps: Vec<LaunchableApp> = serde_json::from_str(old).unwrap();
        assert!(apps[0].target.is_none());
        assert!(
            matches!(apps[0].launch_target(), LaunchTarget::Executable {args, ..} if args == vec!["--profile", "Work profile"])
        );
        assert!(!apps[0].enabled);
        assert_eq!(apps[0].order, 4);
        assert!(serde_json::to_string(&apps).unwrap().starts_with('['));
    }

    #[test]
    fn shortcut_identity_preserves_distinct_profiles() {
        let one = LaunchTarget::Shortcut {
            path: "C:\\Menu\\Work.lnk".into(),
        };
        let case = LaunchTarget::Shortcut {
            path: "c:/menu/WORK.lnk".into(),
        };
        let two = LaunchTarget::Shortcut {
            path: "C:\\Menu\\Personal.lnk".into(),
        };
        assert_eq!(one.identity(), case.identity());
        assert_ne!(one.identity(), two.identity());
    }

    #[test]
    fn packaged_target_roundtrips_without_executable_path() {
        let target = LaunchTarget::Packaged {
            aumid: "Example.Package!App".into(),
        };
        let encoded = serde_json::to_string(&target).unwrap();
        assert_eq!(
            serde_json::from_str::<LaunchTarget>(&encoded).unwrap(),
            target
        );
        assert!(target.file_path().is_none());
    }

    #[test]
    fn unreadable_saved_data_is_never_overwritten_by_adding_an_app() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "project-notch-launcher-test-{}-{suffix}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        let saved = directory.join("launcher_apps.json");
        let executable = directory.join("test.exe");
        let original = b"[{invalid-existing-launcher-data";
        std::fs::write(&saved, original).unwrap();
        std::fs::write(&executable, []).unwrap();
        let manager = LauncherManager::new();
        manager.init(directory.clone());
        let result = manager.add_app(LaunchableApp {
            id: "new-app".into(),
            name: "Test".into(),
            executable_path: executable.to_string_lossy().into_owned(),
            args: Vec::new(),
            icon_path: None,
            icon_data_url: None,
            enabled: true,
            order: 0,
            exists: true,
            target: None,
        });
        assert!(result
            .unwrap_err()
            .contains("existing file was left unchanged"));
        assert_eq!(std::fs::read(&saved).unwrap(), original);
        assert!(manager.get_apps().is_empty());
        std::fs::remove_file(executable).unwrap();
        std::fs::remove_file(saved).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
