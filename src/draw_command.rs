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
    let floats: Result<Vec<f32>, _> = params.iter().map(|s| s.parse()).collect();

    floats.ok().map(|v| DrawCommand::DrawTriangle {
        x1: v[0],
        y1: v[1],
        x2: v[2],
        y2: v[3],
        x3: v[4],
        y3: v[5],
        r: v[6],
        g: v[7],
        b: v[8],
        a: v[9],
    })
}

