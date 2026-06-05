#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    DrawTriangle {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },
}

impl DrawCommand {
    pub fn triangle(values: [f32; 10]) -> Self {
        let [x1, y1, x2, y2, x3, y3, r, g, b, a] = values;
        DrawCommand::DrawTriangle {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
            r,
            g,
            b,
            a,
        }
    }
}

pub fn parse_command(input: &str) -> Option<DrawCommand> {
    let list = parse_list(input)?;
    let car = list.first()?;
    let cdr = &list[1..];

    match (car.as_str(), cdr.len()) {
        ("draw-triangle", 10) => parse_draw_triangle(cdr),
        _ => None,
    }
}

fn parse_list(input: &str) -> Option<Vec<String>> {
    let trimmed = input.trim();

    if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    Some(
        inner
            .split_whitespace()
            .map(|s| s.to_string())
            .collect(),
    )
}

fn parse_draw_triangle(params: &[String]) -> Option<DrawCommand> {
    let values: [f32; 10] = params
        .iter()
        .map(|s| s.parse::<f32>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?
        .try_into()
        .ok()?;

    Some(DrawCommand::triangle(values))
}

