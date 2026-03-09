[package]
name = "{{PACKAGE_NAME}}"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
dioxus = { version = "0.5", features = ["desktop"] }
dioxus-desktop = "0.5"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
