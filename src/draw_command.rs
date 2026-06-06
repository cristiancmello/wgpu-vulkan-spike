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
                $variant { id: u32, $($field: f32),* }
            ),*,
            Clear { r: f32, g: f32, b: f32, a: f32 },
            SetTransform { id: u32, tx: f32, ty: f32, sx: f32, sy: f32 },
            Reset,
            Present,
        }

        impl DrawCommand {
            $(
                pub fn $constructor(id: u32, values: [f32; <$variant>::FIELD_COUNT]) -> Self {
                    let [$($field),*] = values;
                    DrawCommand::$variant { id, $($field),* }
                }
            )*
        }

        const COMMAND_SPECS: &[CommandSpec] = &[
            $(
                CommandSpec {
                    name: $cmd_name,
                    param_count: <$variant>::FIELD_COUNT + 1,
                    parser: {
                        const fn make_parser() -> CommandParser {
                            fn parser(params: &[String]) -> Option<DrawCommand> {
                                let id = params[0].parse::<u32>().ok()?;
                                parse_and_construct(&params[1..], id, DrawCommand::$constructor)
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
    ("draw-triangle", triangle, DrawTriangle, [x1, y1, x2, y2, x3, y3, r, g, b, a]),
    ("draw-rect", rect, DrawRect, [x, y, w, h, r, g, b, a]),
);

pub fn parse_command(input: &str) -> Option<DrawCommand> {
    let list = parse_list(input)?;
    let head = list.first()?;
    let tail = &list[1..];

    if head == "clear" && tail.len() == 4 {
        parse_clear(tail)
    } else if head == "present" && tail.is_empty() {
        Some(DrawCommand::Present)
    } else if head == "reset" && tail.is_empty() {
        Some(DrawCommand::Reset)
    } else if head == "set-transform" && tail.len() == 6 {
        parse_set_transform(tail)
    } else {
        for spec in COMMAND_SPECS {
            if spec.matches(head, tail.len()) {
                return (spec.parser)(tail);
            }
        }
        None
    }
}

fn parse_clear(params: &[String]) -> Option<DrawCommand> {
    let values = parse_floats(params)?;
    let [r, g, b, a] = values.as_slice() else {
        return None;
    };
    Some(DrawCommand::Clear {
        r: *r,
        g: *g,
        b: *b,
        a: *a,
    })
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
    id: u32,
    constructor: F,
) -> Option<DrawCommand>
where
    F: Fn(u32, [f32; N]) -> DrawCommand,
{
    let values = parse_floats(params)?;
    let array: [f32; N] = values.as_slice().try_into().ok()?;
    Some(constructor(id, array))
}

pub fn extract_draw_data(commands: &[DrawCommand]) -> (wgpu::Color, crate::primitive::PrimitiveBuffer) {
    let mut clear_color = wgpu::Color::BLACK;
    let mut primitives = crate::primitive::PrimitiveBuffer::new();

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
            DrawCommand::DrawTriangle { id, x1, y1, x2, y2, x3, y3, r, g, b, a } => {
                let color = [*r, *g, *b, *a];
                let mut vertices = Vec::new();
                vertices.push(crate::renderer::Vertex { position: [*x1, *y1], color });
                vertices.push(crate::renderer::Vertex { position: [*x2, *y2], color });
                vertices.push(crate::renderer::Vertex { position: [*x3, *y3], color });
                primitives.add(crate::primitive::Primitive::new(*id, vertices));
            }
            DrawCommand::DrawRect { id, x, y, w, h, r, g, b, a } => {
                let color = [*r, *g, *b, *a];

                let x1 = *x;
                let y1 = *y;
                let x2 = *x + *w;
                let y2 = *y - *h;

                let mut vertices = Vec::new();
                vertices.push(crate::renderer::Vertex { position: [x1, y1], color });
                vertices.push(crate::renderer::Vertex { position: [x2, y1], color });
                vertices.push(crate::renderer::Vertex { position: [x1, y2], color });

                vertices.push(crate::renderer::Vertex { position: [x2, y1], color });
                vertices.push(crate::renderer::Vertex { position: [x2, y2], color });
                vertices.push(crate::renderer::Vertex { position: [x1, y2], color });

                primitives.add(crate::primitive::Primitive::new(*id, vertices));
            }
            DrawCommand::SetTransform { .. } => {}
            DrawCommand::Reset => {}
            DrawCommand::Present => {}
        }
    }

    (clear_color, primitives)
}

