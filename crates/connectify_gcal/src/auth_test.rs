#[cfg(test)]
mod tests {
    use crate::auth::create_calendar_hub;
    use connectify_config::load_config;

    #[tokio::test]
    async fn test_create_calendar_hub_missing_key_path() {
        // Create a GcalConfig with missing key_path
        let config = load_config()
            .expect("Failed to load config")
            .gcal
            .expect("Failed to load gcal config");

        // Call the function and check that it returns an error
        let result = create_calendar_hub(&config).await;
        assert!(
            result.is_err(),
            "Should return an error when key_path is missing"
        );

        // Check the error message
        match result {
            Ok(_) => panic!("Expected an error but got Ok"),
            Err(err) => {
                let err_string = err.to_string();
                assert!(
                    err_string.contains("No such file or directory (os error 2)"),
                    "Error message should mention missing key_path, got: {}",
                    err_string
                );
            }
        }
    }

    #[tokio::test]
    async fn test_create_calendar_hub_invalid_key_path() {
        // Create a GcalConfig with an invalid key_path
        let config = load_config()
            .expect("Failed to load config")
            .gcal
            .expect("Failed to load gcal config");

        // Call the function and check that it returns an error
        let result = create_calendar_hub(&config).await;
        assert!(
            result.is_err(),
            "Should return an error when key_path is invalid"
        );

        // Check that the error is related to file not found
        // The exact error message might vary depending on the OS and implementation
        match result {
            Ok(_) => panic!("Expected an error but got Ok"),
            Err(err) => {
                let err_string = err.to_string();
                assert!(
                    err_string.contains("No such file")
                        || err_string.contains("not found")
                        || err_string.contains("cannot find")
                        || err_string.contains("doesn't exist"),
                    "Error message should indicate file not found, got: {}",
                    err_string
                );
            }
        }
    }

    // Note: We can't easily test the success case without a real service account key file
    // In a real test suite, you would mock the file reading and authentication parts
}
