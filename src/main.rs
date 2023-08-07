use std::fs;

slint::include_modules!();

impl CSSThemeProperty {
    pub fn colour(h: f32, s: f32, v: f32) -> Self {
        CSSThemeProperty {
            colour_h: h,
            colour_s: s,
            colour_v: v,
            custom_css_active: false,
            custom_css_value: slint::SharedString::new(),
        }
    }
}

fn default_theme() -> Theme {
    Theme {
        background_primary: CSSThemeProperty::colour(0.6194444, 0.13, 0.22),
        background_secondary: CSSThemeProperty::colour(0.6333333, 0.13, 0.16),
        background_tertiary: CSSThemeProperty::colour(0.625, 0.11, 0.14),
        text_primary: CSSThemeProperty::colour(0.5833333, 0.03, 0.88),
        text_muted: CSSThemeProperty::colour(0.5944444, 0.1, 0.64),
        text_hyperlink: CSSThemeProperty::colour(0.5555555, 1.0, 0.98),
        header_primary: CSSThemeProperty::colour(0.5833333, 0.03, 0.88),
        header_secondary: CSSThemeProperty::colour(0.5972222, 0.06, 0.75),
    }
}

fn main() -> Result<(), &'static str> {
    let main_window = MainWindow::new().map_err(|_| "Failed to create window")?;

    let helpers = main_window.global::<Helpers>();

    helpers.on_hsv(move |h, s, v| {
        let rgb: prisma::Rgb<f32> = prisma::Hsv::new(angular_units::Turns(h % 1.0), s, v).into();
        slint::Color::from_rgb_f32(rgb.red(), rgb.green(), rgb.blue())
    });

    helpers.on_hex(move |colour| {
        format!(
            "#{:02x}{:02x}{:02x}",
            colour.red(),
            colour.green(),
            colour.blue()
        )
        .into()
    });

    main_window.on_reset_theme_to_default({
        let weak_window = main_window.as_weak();
        move || {
            let window = weak_window.unwrap();
            window.invoke_show_error_dialog(slint::SharedString::from("Failed to open file"));
            window.invoke_set_current_theme(default_theme());
        }
    });

    main_window.on_load_theme({
        let weak_window = main_window.as_weak();
        move || match load_theme() {
            Ok(Some(theme)) => weak_window.unwrap().invoke_set_current_theme(theme),
            Ok(None) => {}
            Err(message) => {
                weak_window
                    .unwrap()
                    .invoke_show_error_dialog(slint::SharedString::from(message));
            }
        }
    });

    main_window.on_save_theme({
        let weak_window = main_window.as_weak();
        move |theme| {
            if let Err(message) = save_theme(&theme) {
                weak_window
                    .unwrap()
                    .invoke_show_error_dialog(slint::SharedString::from(message));
            }
        }
    });

    main_window.on_apply_theme(apply_theme);

    main_window.invoke_set_current_theme(default_theme());
    main_window.run().map_err(|_| "Failed to run main loop")?;
    Ok(())
}

fn save_theme(theme: &Theme) -> Result<(), &'static str> {
    let picked_file = rfd::FileDialog::new()
        .add_filter("DTE Theme Description", &["dthm"])
        .set_file_name("theme.dthm")
        .save_file();
    if let Some(file_path) = picked_file {
        let file = fs::File::create(file_path).map_err(|_| "Failed to open file")?;
        serde_json::to_writer(file, theme).map_err(|_| "Failed to write theme to file")?;
    }
    Ok(())
}

fn load_theme() -> Result<Option<Theme>, &'static str> {
    let picked_file = rfd::FileDialog::new()
        .add_filter("DTE Theme Description", &["dthm"])
        .pick_file();
    if let Some(file_path) = picked_file {
        let file = fs::File::open(file_path).map_err(|_| "Failed to open file")?;
        let theme: Theme =
            serde_json::from_reader(file).map_err(|_| "Failed to read theme file")?;
        Ok(Some(theme))
    } else {
        Ok(None)
    }
}

fn apply_theme(theme: Theme) {
    println!("{}", theme.background_primary.colour_h);
    todo!("apply_theme() not implemented yet");
}
