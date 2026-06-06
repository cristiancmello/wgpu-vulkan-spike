type CommandParser = fn(&[String]) -> Option<DrawCommand>;

struct CommandSpec {
    name: &'static str,
    param_count: usize,
    parser: CommandParser,
}

impl CommandSpec {
    fn matches(&self, cmd_name: &str, param_count: usize) -> bool {
        cmd_name == self.name && param_count == self.param_count
    }
}

trait FieldCount {
    const FIELD_COUNT: usize;
}

macro_rules! impl_field_count {
    ($name:ident, $count:expr) => {
        struct $name;
        impl FieldCount for $name {
            const FIELD_COUNT: usize = $count;
        }
    };
}


macro_rules! register_commands {
    (
        $(
            ($cmd_name:expr, $constructor:ident, $variant:ident, [$($field:ident),*])
        ),* $(,)?
    ) => {
        $(
            impl_field_count!($variant, count_fields!($($field),*));
        )*

        #[derive(Clone, Debug, PartialEq)]
        pub enum DrawCommand {
            $(
                $variant { $($field: f32),* }
            ),*,
            SetTransform { id: u32, tx: f32, ty: f32, sx: f32, sy: f32 },
            Present,
        }

        impl DrawCommand {
            $(
                pub fn $constructor(values: [f32; <$variant>::FIELD_COUNT]) -> Self {
                    let [$($field),*] = values;
                    DrawCommand::$variant { $($field),* }
                }
            )*
        }

        const COMMAND_SPECS: &[CommandSpec] = &[
            $(
                CommandSpec {
                    name: $cmd_name,
                    param_count: <$variant>::FIELD_COUNT,
                    parser: {
                        const fn make_parser() -> CommandParser {
                            fn parser(params: &[String]) -> Option<DrawCommand> {
                                parse_and_construct(params, DrawCommand::$constructor)
                            }
                            parser
                        }
                        make_parser()
                    }
                },
            )*
        ];
    };
}

macro_rules! count_fields {
    ($($field:ident),*) => {
        {
            let mut count = 0;
            $(
                let _ = stringify!($field);
                count += 1;
            )*
            count
        }
    };
}

register_commands!(
    ("clear", clear, Clear, [r, g, b, a]),
    ("draw-triangle", triangle, DrawTriangle, [x1, y1, x2, y2, x3, y3, r, g, b, a]),
    ("draw-rect", rect, DrawRect, [x, y, w, h, r, g, b, a]),
);

pub fn parse_command(input: &str) -> Option<DrawCommand> {
    let list = parse_list(input)?;
    let head = list.first()?;
    let tail = &list[1..];

    for spec in COMMAND_SPECS {
        if spec.matches(head, tail.len()) {
            return (spec.parser)(tail);
        }
    }

    if head == "present" && tail.is_empty() {
        Some(DrawCommand::Present)
    } else if head == "set-transform" && tail.len() == 5 {
        parse_set_transform(tail)
    } else {
        None
    }
}

fn parse_set_transform(params: &[String]) -> Option<DrawCommand> {
    let id = params[0].parse::<u32>().ok()?;
    let tx = params[1].parse::<f32>().ok()?;
    let ty = params[2].parse::<f32>().ok()?;
    let sx = params[3].parse::<f32>().ok()?;
    let sy = params[4].parse::<f32>().ok()?;
    Some(DrawCommand::SetTransform { id, tx, ty, sx, sy })
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

fn parse_floats(params: &[String]) -> Option<Vec<f32>> {
    params
        .iter()
        .map(|s| s.parse::<f32>())
        .collect::<Result<Vec<_>, _>>()
        .ok()
}

fn parse_and_construct<const N: usize, F>(
    params: &[String],
    constructor: F,
) -> Option<DrawCommand>
where
    F: Fn([f32; N]) -> DrawCommand,
{
    let values = parse_floats(params)?;
    let array: [f32; N] = values.as_slice().try_into().ok()?;
    Some(constructor(array))
}

pub fn extract_draw_data(commands: &[DrawCommand]) -> (wgpu::Color, Vec<crate::renderer::Vertex>) {
    let mut clear_color = wgpu::Color::BLACK;
    let mut vertices = Vec::new();

    for cmd in commands {
        match cmd {
            DrawCommand::Clear { r, g, b, a } => {
                clear_color = wgpu::Color {
                    r: *r as f64,
                    g: *g as f64,
                    b: *b as f64,
                    a: *a as f64,
                };
            }
            DrawCommand::DrawTriangle { x1, y1, x2, y2, x3, y3, r, g, b, a } => {
                let color = [*r, *g, *b, *a];
                vertices.push(crate::renderer::Vertex { position: [*x1, *y1], color });
                vertices.push(crate::renderer::Vertex { position: [*x2, *y2], color });
                vertices.push(crate::renderer::Vertex { position: [*x3, *y3], color });
            }
            DrawCommand::DrawRect { x, y, w, h, r, g, b, a } => {
                let color = [*r, *g, *b, *a];

                let x1 = *x;
                let y1 = *y;
                let x2 = *x + *w;
                let y2 = *y - *h;

                vertices.push(crate::renderer::Vertex { position: [x1, y1], color });
                vertices.push(crate::renderer::Vertex { position: [x2, y1], color });
                vertices.push(crate::renderer::Vertex { position: [x1, y2], color });

                vertices.push(crate::renderer::Vertex { position: [x2, y1], color });
                vertices.push(crate::renderer::Vertex { position: [x2, y2], color });
                vertices.push(crate::renderer::Vertex { position: [x1, y2], color });
            }
            DrawCommand::SetTransform { .. } => {}
            DrawCommand::Present => {}
        }
    }

    (clear_color, vertices)
}

