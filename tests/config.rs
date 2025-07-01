#[cfg(test)]
mod tests {
    use llm_router::config::load_config;
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

        let config = load_config(temp_file.path().to_str().unwrap());

        assert_eq!(config.refresh_interval, 300);
        assert_eq!(config.backends.len(), 1);
        assert_eq!(config.backends[0].name, "test-backend");
    }
}
