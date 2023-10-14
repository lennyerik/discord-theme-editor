pub fn css_to_svg_def(css_value: &str) -> Option<String> {
    let gradient = parse_css_gradient(css_value)?;
    todo!("css_to_svg_def {:?}", gradient)
}

fn parse_css_gradient(css_value: &str) -> Option<Gradient> {
    let css_value = css_value.trim().strip_suffix(')')?;
    let (gradient_type, gradient_def) =
        if let Some(gradient_def) = css_value.strip_prefix("linear-gradient(") {
            (GradientType::Linear, gradient_def)
        } else if let Some(gradient_def) = css_value.strip_prefix("radial-gradient(") {
            (GradientType::Radial, gradient_def)
        } else {
            return None;
        };

    let mut tokens = gradient_def.split(',').map(|t| t.trim());
    let first = tokens.next()?;
    let mut stops = Vec::new();

    let direction = parse_direction(first).or_else(|| {
        stops.push(parse_stop(first)?);
        None
    });
    stops.extend(tokens.filter_map(parse_stop));

    Some(Gradient {
        r#type: gradient_type,
        direction,
        stops,
    })
}

fn parse_direction(dir_value: &str) -> Option<Direction> {
    let mut parts = dir_value.split_whitespace();

    let first = parts.next()?;
    let direction = if first == "to" {
        match (parts.next()?, parts.next()) {
            ("left", None) => Some(Direction::Left),
            ("right", None) => Some(Direction::Right),
            ("top", None) => Some(Direction::Top),
            ("bottom", None) => Some(Direction::Bottom),
            ("top", Some("left")) => Some(Direction::TopLeft),
            ("top", Some("right")) => Some(Direction::TopRight),
            ("bottom", Some("left")) => Some(Direction::BottomLeft),
            ("bottom", Some("right")) => Some(Direction::BottomRight),
            _ => None,
        }
    } else if first.ends_with("deg")
        || first.ends_with("rad")
        || first.ends_with("grad")
        || first.ends_with("turn")
    {
        Some(Direction::Rotation(first))
    } else {
        None
    };

    // Check that there are no more parts left
    if parts.next().is_none() {
        direction
    } else {
        None
    }
}

fn parse_stop(stop_value: &str) -> Option<Stop> {
    let mut parts = stop_value.split_whitespace();
    let colour = parts.next()?;
    let offset = parts.next();

    if parts.next().is_none() {
        Some(Stop { offset, colour })
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Gradient<'s> {
    r#type: GradientType,
    direction: Option<Direction<'s>>,
    stops: Vec<Stop<'s>>,
}

#[derive(Debug, PartialEq, Eq)]
struct Stop<'s> {
    offset: Option<&'s str>,
    colour: &'s str,
}

#[derive(Debug, PartialEq, Eq)]
enum Direction<'s> {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Rotation(&'s str),
}

#[derive(Debug, PartialEq, Eq)]
enum GradientType {
    Linear,
    Radial,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_linear_gradient() {
        assert_eq!(
            parse_css_gradient("linear-gradient(red, blue, green)"),
            Some(Gradient {
                r#type: GradientType::Linear,
                direction: None,
                stops: vec!(
                    Stop {
                        offset: None,
                        colour: "red"
                    },
                    Stop {
                        offset: None,
                        colour: "blue"
                    },
                    Stop {
                        offset: None,
                        colour: "green"
                    },
                ),
            })
        );
    }

    #[test]
    fn parse_linear_gradient_with_direction() {
        assert_eq!(
            parse_css_gradient("linear-gradient(to top left, #ff0000, #00ff00 85%, #0000ff)"),
            Some(Gradient {
                r#type: GradientType::Linear,
                direction: Some(Direction::TopLeft),
                stops: vec!(
                    Stop {
                        offset: None,
                        colour: "#ff0000"
                    },
                    Stop {
                        offset: Some("85%"),
                        colour: "#00ff00"
                    },
                    Stop {
                        offset: None,
                        colour: "#0000ff"
                    },
                ),
            })
        );
    }

    /*
    #[test]
    fn parse_simple_radial_gradient() {
        assert_eq!(
            parse_css_gradient("radial-gradient(black, purple)"),
            Some(Gradient {
                r#type: GradientType::Radial,
                direction: None,
                stops: vec!(
                    Stop {
                        offset: None,
                        colour: "black"
                    },
                    Stop {
                        offset: None,
                        colour: "purple"
                    },
                ),
            })
        );
    }*/

    #[test]
    fn parse_sided_direction() {
        assert_eq!(parse_direction("to left"), Some(Direction::Left));
        assert_eq!(parse_direction("to right"), Some(Direction::Right));
        assert_eq!(parse_direction("to top"), Some(Direction::Top));
        assert_eq!(parse_direction("to bottom"), Some(Direction::Bottom));

        assert_eq!(parse_direction("to    left"), Some(Direction::Left));

        assert_eq!(parse_direction("to nowhere"), None);
    }

    #[test]
    fn parse_corner_direction() {
        assert_eq!(parse_direction("to top left"), Some(Direction::TopLeft));
        assert_eq!(parse_direction("to top right"), Some(Direction::TopRight));
        assert_eq!(
            parse_direction("to bottom left"),
            Some(Direction::BottomLeft)
        );
        assert_eq!(
            parse_direction("to bottom right"),
            Some(Direction::BottomRight)
        );

        assert_eq!(parse_direction("to    top  left"), Some(Direction::TopLeft));

        assert_eq!(parse_direction("to be free"), None);
    }

    #[test]
    fn parse_rotational_direction() {
        assert_eq!(parse_direction("45deg"), Some(Direction::Rotation("45deg")));
        assert_eq!(
            parse_direction("3.1416rad"),
            Some(Direction::Rotation("3.1416rad"))
        );
        assert_eq!(
            parse_direction("-50grad"),
            Some(Direction::Rotation("-50grad"))
        );
        assert_eq!(
            parse_direction("1.75turn"),
            Some(Direction::Rotation("1.75turn"))
        );
    }

    #[test]
    fn parse_colour_stop() {
        assert_eq!(
            parse_stop("red"),
            Some(Stop {
                offset: None,
                colour: "red"
            })
        );

        assert_eq!(
            parse_stop("#00ff00"),
            Some(Stop {
                offset: None,
                colour: "#00ff00"
            })
        );

        assert_eq!(parse_stop("red 50% blue"), None);
    }

    #[test]
    fn parse_colour_length_stop() {
        assert_eq!(
            parse_stop("red 50%"),
            Some(Stop {
                offset: Some("50%"),
                colour: "red"
            })
        );
        assert_eq!(
            parse_stop("purple -5%"),
            Some(Stop {
                offset: Some("-5%"),
                colour: "purple"
            })
        );

        assert_eq!(
            parse_stop("#00ff00 10px"),
            Some(Stop {
                offset: Some("10px"),
                colour: "#00ff00"
            })
        );

        assert_eq!(parse_stop("1% green red"), None);
    }
}
