#[cfg(test)]
mod tests {
    use llm_router::config::{load_config, try_load_config};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config() {
        let config_content = r#"
            refresh_interval: 300
            backends:
              - name: "test-backend"
                url: "http://localhost:8000"
                auth:
                  type: "bearer"
                  token: "test-token"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", config_content).unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        let new_path = format!("{}.yml", path);
        std::fs::rename(temp_file.path(), &new_path).unwrap();

        let config = load_config(&new_path);
        assert_eq!(config.refresh_interval, 300);
    }

    #[test]
    fn test_load_config_yml() {
        let config_content = r#"
            refresh_interval: 300
            backends:
              - name: "test-backend"
                url: "http://localhost:8000"
                auth:
                  type: "bearer"
                  token: "test-token"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        let new_path = format!("{}.yml", path);

        std::fs::write(&new_path, config_content).unwrap();
        let config = load_config(&new_path);
        assert_eq!(config.refresh_interval, 300);
        std::fs::remove_file(&new_path).unwrap();
    }

    #[test]
    fn test_load_config_yaml() {
        let config_content = r#"
            refresh_interval: 300
            backends:
              - name: "test-backend"
                url: "http://localhost:8000"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        let new_path = format!("{}.yaml", path);

        std::fs::write(&new_path, config_content).unwrap();
        let config = load_config(&new_path);
        assert_eq!(config.refresh_interval, 300);
        std::fs::remove_file(&new_path).unwrap();
    }

    #[test]
    #[should_panic(expected = "Configuration file not found. Check if config.yml or config.yaml file exists")]
    fn test_load_config_not_found() {
        load_config("nonexistent_config");
    }

    #[test]
    #[should_panic(expected = "Configuration file not found. Check if config.yml or config.yaml file exists")]
    fn test_load_config_invalid_extension() {
        let config_content = r#"
            refresh_interval: 300
            backends: []
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        let new_path = format!("{}.txt", path);

        std::fs::write(&new_path, config_content).unwrap();
        let result = load_config(&new_path);
        std::fs::remove_file(&new_path).unwrap();
        drop(result);
    }


    #[test]
    fn test_try_load_config_invalid_yaml() {
        let invalid_content = "invalid: : yaml: content:";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", invalid_content).unwrap();

        let result = try_load_config(temp_file.path());
        assert!(result.is_none());
    }

    #[test]
    #[should_panic(expected = "Configuration file not found. Check if config.yml or config.yaml file exists")]
    fn test_load_config_invalid_path() {
        load_config(".yml");
    }

    #[test]
    #[should_panic(expected = "Configuration file not found. Check if config.yml or config.yaml file exists")]
    fn test_load_config_empty_path() {
        load_config("");
    }

}
