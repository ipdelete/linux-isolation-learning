// Tests for the `show` subcommand (displaying config.json)
// Lesson: docs/03-runc/01-bundle.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor as needed

#[test]
fn test_show_displays_config() {
    // TODO: Write a test that verifies showing config.json contents
    //
    // Hints:
    // - The `show` subcommand should read and display config.json
    // - Can display as formatted JSON for readability
    // - Should output the full config to stdout
    //
    // Test approach:
    // 1. Create a test bundle with known config.json
    // 2. Run `oci-tool show /tmp/test-bundle`
    // 3. Verify output contains config.json content
    // 4. Verify it's valid JSON
    // 5. Clean up

    todo!("Implement test for showing config.json")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_show_formats_json_pretty() {
    // TODO: Write a test that verifies JSON is pretty-printed
    //
    // Hints:
    // - Output should be formatted with indentation
    // - Makes it easier to read
    // - Use serde_json::to_string_pretty()

    todo!("Implement test for pretty-printed JSON output")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_show_fails_if_bundle_missing() {
    // TODO: Write a test that verifies error when bundle doesn't exist
    //
    // Hints:
    // - Try to show a non-existent bundle
    // - Should return clear error message

    todo!("Implement test for error handling with missing bundle")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_show_fails_if_config_missing() {
    // TODO: Write a test that verifies error when config.json is missing
    //
    // Hints:
    // - Create bundle directory without config.json
    // - Try to show it
    // - Should return clear error

    todo!("Implement test for error handling with missing config.json")
}
