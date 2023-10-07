// Enable "windows" subsystem for Windows release builds
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use std::fs;

mod css_gradients;
mod discord_rebuilder;

slint::include_modules!();

impl CSSThemeProperty {
    fn colour(h: f32, s: f32, v: f32) -> Self {
        CSSThemeProperty {
            colour_h: h,
            colour_s: s,
            colour_v: v,
            custom_css_active: false,
            custom_css_value: slint::SharedString::new(),
        }
    }

    fn svg_defs(&self) -> String {
        "".into()
    }

    fn svg_css(&self) -> String {
        // TODO: We need defs for gradients
        assert!(!self.custom_css_active);

        let (r, g, b) = hsv_to_rgb(self.colour_h, self.colour_s, self.colour_v);
        format!(
            "#{:02x}{:02x}{:02x}",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        )
    }

    fn web_css(&self) -> String {
        if self.custom_css_active {
            self.custom_css_value.to_string()
        } else {
            let (r, g, b) = hsv_to_rgb(self.colour_h, self.colour_s, self.colour_v);
            format!(
                "#{:02x}{:02x}{:02x}",
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8
            )
        }
    }
}

fn default_theme() -> Theme {
    Theme {
        background_primary: CSSThemeProperty::colour(0.6194444, 0.13, 0.22),
        background_secondary: CSSThemeProperty::colour(0.6111111, 0.13, 0.19),
        background_secondary_alt: CSSThemeProperty::colour(0.6333333, 0.12, 0.16),
        background_tertiary: CSSThemeProperty::colour(0.625, 0.11, 0.14),
        text_normal: CSSThemeProperty::colour(0.5833333, 0.03, 0.88),
        text_muted: CSSThemeProperty::colour(0.5944444, 0.1, 0.64),
        text_hyperlink: CSSThemeProperty::colour(0.5555555, 1.0, 0.98),
        header_primary: CSSThemeProperty::colour(0.5833333, 0.03, 0.88),
        header_secondary: CSSThemeProperty::colour(0.5972222, 0.06, 0.75),
        channels_default: CSSThemeProperty::colour(0.5944444, 0.1, 0.64),
    }
}

fn main() -> Result<(), &'static str> {
    let main_window = MainWindow::new().map_err(|_| "Failed to create window")?;

    let helpers = main_window.global::<Helpers>();

    helpers.on_hsv(move |h, s, v| {
        let (r, g, b) = hsv_to_rgb(h, s, v);
        slint::Color::from_rgb_f32(r, g, b)
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
            window.invoke_set_current_theme(default_theme());
            let _ = discord_rebuilder::reset_theme();
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
                    .invoke_show_error_dialog(message.into());
            }
        }
    });

    main_window.on_save_theme({
        let weak_window = main_window.as_weak();
        move |theme| {
            if let Err(message) = save_theme(&theme) {
                weak_window
                    .unwrap()
                    .invoke_show_error_dialog(message.into());
            }
        }
    });

    main_window.on_apply_theme({
        let weak_window = main_window.as_weak();
        move |theme| {
            if let Err(message) = discord_rebuilder::apply_theme(&theme) {
                weak_window
                    .unwrap()
                    .invoke_show_error_dialog(message.into());
            }
        }
    });

    main_window.on_generate_preview(move |theme| generate_preview(&theme));

    main_window.invoke_set_current_theme(default_theme());
    main_window.run().map_err(|_| "Failed to run main loop")?;
    Ok(())
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let rgb: prisma::Rgb<f32> = prisma::Hsv::new(angular_units::Turns(h % 1.0), s, v).into();
    (rgb.red(), rgb.green(), rgb.blue())
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

fn generate_preview(theme: &Theme) -> slint::Image {
    let generated_defs = &[
        theme.background_tertiary.svg_defs(),
        theme.background_secondary.svg_defs(),
        theme.background_tertiary.svg_defs(),
        theme.text_normal.svg_defs(),
        theme.text_muted.svg_defs(),
        theme.text_hyperlink.svg_defs(),
        theme.header_primary.svg_defs(),
        theme.header_secondary.svg_defs(),
    ]
    .concat();

    let generated_css = format!(
        r#"
        .background-primary{{ fill: {}; }}
        .background-secondary{{ fill: {}; }}
        .background-secondary-alt{{ fill: {}; }}
        .background-tertiary{{ fill: {}; }}
        .text-normal{{ fill: {}; }}
        .header-primary{{ stroke: {}; }}
        .header-secondary{{ stroke: {}; }}
        .channels-default{{ stroke: {}; }}
    "#,
        theme.background_primary.svg_css(),
        theme.background_secondary.svg_css(),
        theme.background_secondary_alt.svg_css(),
        theme.background_tertiary.svg_css(),
        theme.text_normal.svg_css(),
        theme.header_primary.svg_css(),
        theme.header_secondary.svg_css(),
        theme.channels_default.svg_css(),
    );

    let generated_svg = include_str!("../res/discord.svg")
        .replace("{!!!GENERATED_DEFS!!!}", generated_defs)
        .replace("{!!!GENERATED_CSS!!!}", &generated_css);
    match slint::Image::load_from_svg_data(generated_svg.as_bytes()) {
        Ok(img) => img,
        Err(_) => slint::Image::default(),
    }
}
