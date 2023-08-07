const THEME: &str = "fluent-dark";

fn main() {
    let config = slint_build::CompilerConfiguration::default().with_style(THEME.into());
    slint_build::compile_with_config("ui/main-window.slint", config).unwrap();
}
