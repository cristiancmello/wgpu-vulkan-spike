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
    } else {
        None
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


