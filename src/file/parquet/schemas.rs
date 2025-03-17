use std::sync::OnceLock;

use crate::{
    file::{ReadAt, SubFile},
    pqarray::{schema_match, PqArrayMatcher, PqArrayReader},
};

macro_rules! declare_schema {
    ($name:ident { $( $variant:ident { $($token:tt)* } )* }) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub(super) enum $name {
            $($variant,)*
        }

        impl $name {
            fn matcher() -> &'static PqArrayMatcher<$name> {
                static MATCHER: OnceLock<PqArrayMatcher<$name>> = OnceLock::new();
                MATCHER.get_or_init(|| {
                    schema_match! {
                        $($name::$variant => schema { $($token)* })*
                    }
                })
            }

            pub fn check<R: ReadAt>(
                reader: &PqArrayReader<SubFile<R>>,
            ) -> Result<$name, crate::error::Error> {
                reader.matches(Self::matcher())
            }
        }
    };
}

declare_schema! {
    Scalar {
        F32 {
            required float scalar;
        }
        F64 {
            required double scalar;
        }
    }
}

declare_schema! {
    Vertex {
        F32 {
            required float x;
            required float y;
            required float z;
        }
        F64 {
            required double x;
            required double y;
            required double z;
        }
    }
}

declare_schema! {
    Segment {
        U32 {
            required int32 a (integer(32, false));
            required int32 b (integer(32, false));
        }
    }
}

declare_schema! {
    Triangle {
        U32 {
            required int32 a (integer(32, false));
            required int32 b (integer(32, false));
            required int32 c (integer(32, false));
        }
    }
}

declare_schema! {
    Name {
        String {
            required byte_array name (string);
        }
    }
}

declare_schema! {
    Gradient {
        Rgba8 {
            required int32 r (integer(8, false));
            required int32 g (integer(8, false));
            required int32 b (integer(8, false));
            required int32 a (integer(8, false));
        }
    }
}

declare_schema! {
    Texcoord {
        F32 {
            required float u;
            required float v;
        }
        F64 {
            required double u;
            required double v;
        }
    }
}

declare_schema! {
    Boundary {
        F32 {
            required float value;
            required boolean inclusive;
        }
        F64 {
            required double value;
            required boolean inclusive;
        }
        I64 {
            required int64 value;
            required boolean inclusive;
        }
        Date {
            required int32 value (date);
            required boolean inclusive;
        }
        DateTime {
            required int64 value (timestamp(micros, true));
            required boolean inclusive;
        }
    }
}

declare_schema! {
    RegularSubblock {
        U32U32 {
            required int32 parent_u (integer(32, false));
            required int32 parent_v (integer(32, false));
            required int32 parent_w (integer(32, false));
            required int32 corner_min_u (integer(32, false));
            required int32 corner_min_v (integer(32, false));
            required int32 corner_min_w (integer(32, false));
            required int32 corner_max_u (integer(32, false));
            required int32 corner_max_v (integer(32, false));
            required int32 corner_max_w (integer(32, false));
        }
    }
}

declare_schema! {
    FreeformSubblock {
        U32F32 {
            required int32 parent_u (integer(32, false));
            required int32 parent_v (integer(32, false));
            required int32 parent_w (integer(32, false));
            required float corner_min_u;
            required float corner_min_v;
            required float corner_min_w;
            required float corner_max_u;
            required float corner_max_v;
            required float corner_max_w;
        }
        U32F64 {
            required int32 parent_u (integer(32, false));
            required int32 parent_v (integer(32, false));
            required int32 parent_w (integer(32, false));
            required double corner_min_u;
            required double corner_min_v;
            required double corner_min_w;
            required double corner_max_u;
            required double corner_max_v;
            required double corner_max_w;
        }
    }
}

declare_schema! {
    Number {
        F32 {
            optional float number;
        }
        F64 {
            optional double number;
        }
        I64 {
            optional int64 number;
        }
        Date {
            optional int32 number (date);
        }
        DateTime {
            optional int64 number (timestamp(micros, true));
        }
    }
}

declare_schema! {
    Index {
        U32 {
            optional int32 index (integer(32, false));
        }
    }
}

declare_schema! {
    Vector {
        F32x2 {
            optional group vector {
                required float x;
                required float y;
            }
        }
        F64x2 {
            optional group vector {
                required double x;
                required double y;
            }
        }
        F32x3 {
            optional group vector {
                required float x;
                required float y;
                required float z;
            }
        }
        F64x3 {
            optional group vector {
                required double x;
                required double y;
                required double z;
            }
        }
    }
}

declare_schema! {
    Text {
        Text {
            optional byte_array text (string);
        }
    }
}

declare_schema! {
    Boolean {
        Boolean {
            optional boolean bool;
        }
    }
}

declare_schema! {
    Color {
        Rgba8 {
            optional group color {
                required int32 r (integer(8, false));
                required int32 g (integer(8, false));
                required int32 b (integer(8, false));
                required int32 a (integer(8, false));
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn dump_parquet_schemas() {
    use std::{
        fs::{create_dir_all, OpenOptions},
        io::Write,
        path::Path,
    };

    use parquet::schema::{printer::print_schema, types::Type};

    fn schema_string(ty: &Type) -> String {
        let mut buf = Vec::new();
        print_schema(&mut buf, ty);
        String::from_utf8_lossy(&buf).trim_end().to_owned()
    }

    let items = [
        ("Scalar.txt", Scalar::matcher().schemas()),
        ("Vertex.txt", Vertex::matcher().schemas()),
        ("Segment.txt", Segment::matcher().schemas()),
        ("Triangle.txt", Triangle::matcher().schemas()),
        ("Name.txt", Name::matcher().schemas()),
        ("Gradient.txt", Gradient::matcher().schemas()),
        ("Texcoord.txt", Texcoord::matcher().schemas()),
        ("Boundary.txt", Boundary::matcher().schemas()),
        ("RegularSubblock.txt", RegularSubblock::matcher().schemas()),
        (
            "FreeformSubblock.txt",
            FreeformSubblock::matcher().schemas(),
        ),
        ("Number.txt", Number::matcher().schemas()),
        ("Index.txt", Index::matcher().schemas()),
        ("Vector.txt", Vector::matcher().schemas()),
        ("Text.txt", Text::matcher().schemas()),
        ("Boolean.txt", Boolean::matcher().schemas()),
        ("Color.txt", Color::matcher().schemas()),
    ];
    let base_dir = Path::new("docs/parquet");
    create_dir_all(base_dir).unwrap();
    for (name, schemas) in items {
        let path = base_dir.join(name);
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .unwrap();
        let s = schemas
            .into_iter()
            .map(schema_string)
            .collect::<Vec<_>>()
            .join("\n\n");
        f.write_all(s.as_bytes()).unwrap();
    }
}
