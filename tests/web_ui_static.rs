use std::fs;
use std::path::Path;

#[test]
fn static_web_ui_assets_are_present_and_wired_to_http_controller() {
    let root = env!("CARGO_MANIFEST_DIR");
    let html_path = Path::new(root).join("web-ui/index.html");
    let css_path = Path::new(root).join("web-ui/styles.css");
    let js_path = Path::new(root).join("web-ui/app.js");

    let html = fs::read_to_string(&html_path).expect("web-ui/index.html is readable");
    let css = fs::read_to_string(&css_path).expect("web-ui/styles.css is readable");
    let js = fs::read_to_string(&js_path).expect("web-ui/app.js is readable");

    assert!(
        html.contains(r#"href="./styles.css""#),
        "index.html links the stylesheet"
    );
    assert!(
        html.contains(r#"src="./app.js""#),
        "index.html loads the vanilla JavaScript entrypoint"
    );
    assert!(
        html.contains(r#"<main"#),
        "index.html contains the application shell"
    );

    for endpoint in [
        "/state",
        "/events/drain",
        "/command",
        "/command-agent",
        "/agent/message",
        "/agent/interrupt",
        "/shutdown",
        "/transcript",
        "/loading-text",
    ] {
        assert!(js.contains(endpoint), "app.js uses {endpoint}");
    }

    for event_variant in [
        "AgentAdded",
        "AgentUpdated",
        "AgentStatusUpdated",
        "AgentUsageUpdated",
        "AgentLineAppended",
        "AgentSelected",
        "CommandTranscriptLine",
        "QuitRequested",
    ] {
        assert!(js.contains(event_variant), "app.js handles {event_variant}");
    }

    assert!(
        css.contains("@media"),
        "styles.css includes responsive layout rules"
    );
}
