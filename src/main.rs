slint::include_modules!();

fn to_hsv(h: f32, s: f32, v: f32) -> slint::Color {
    let rgb: prisma::Rgb<f32> = prisma::Hsv::new(angular_units::Turns(h % 1.0), s, v).into();
    slint::Color::from_rgb_f32(rgb.red(), rgb.green(), rgb.blue())
}

fn to_hex(colour: slint::Color) -> slint::SharedString {
    format!(
        "#{:02x}{:02x}{:02x}",
        colour.red(),
        colour.green(),
        colour.blue()
    )
    .into()
}

fn main() -> Result<(), &'static str> {
    let main_window = MainWindow::new().map_err(|_| "Failed to create window")?;

    let helpers = main_window.global::<Helpers>();
    helpers.on_hsv(to_hsv);
    helpers.on_hex(to_hex);

    main_window.run().map_err(|_| "Failed to run main loop")?;
    Ok(())
}
