// Note: this outputs Python Markdown for mkdocs rather than the CommonMark that rustdoc uses.
use core::panic;
use std::{
    collections::BTreeMap,
    fs::{create_dir_all, OpenOptions},
    io::Write,
    path::Path,
    sync::OnceLock,
};

use schemars::{
    schema::{
        ArrayValidation, InstanceType, Metadata, ObjectValidation, Schema, SchemaObject,
        SingleOrVec, SubschemaValidation,
    },
    visit::{visit_schema_object, Visitor},
};
use serde_json::Value;

use crate::schema::{project_schema, schema_for};

pub(crate) fn update_schema_docs() {
    let schema = project_schema(false);
    let base_dir = Path::new("docs/schema");
    create_dir_all(Path::new(base_dir)).unwrap();
    object(base_dir, "Project", &schema.schema).unwrap();
    for (name, def) in &schema.definitions {
        let Schema::Object(schema) = def else {
            panic!("unknown definition: \"{name}\" = {def:#?}");
        };
        if name == "Geometry" {
            geometry(base_dir, schema).unwrap();
        } else if name == "NumberRange" {
            number_colormap_range(base_dir, name, schema).unwrap();
        } else {
            object(base_dir, name, schema).unwrap();
        }
    }
}

fn geometry(base_dir: &Path, schema: &SchemaObject) -> std::io::Result<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(base_dir.join("Geometry.md"))?;
    write!(f, "<!-- Generated documentation, do not edit -->\n\n")?;
    // Title.
    write!(f, "# Geometry\n\n")?;
    // Description paragraph.
    let descr = description(&schema);
    write!(f, "{descr}\n\n")?;
    // List of options.
    write!(f, "## Options\n\n")?;
    let Some(SubschemaValidation {
        one_of: Some(variants),
        ..
    }) = schema.subschemas.as_deref()
    else {
        panic!("unknown geometry = {schema:#?}");
    };
    for item in variants {
        let Schema::Object(child_schema) = item else {
            panic!("unknown geometry option = {item:#?}");
        };
        let name = variant_name(child_schema);
        write!(f, "- [{name}]({name}.md)\n")?;
        object(base_dir, name, child_schema)?;
    }
    write!(f, "\n")?;
    // Code.
    schema_object_code(&mut f, schema)
}

fn object(base_dir: &Path, name: &str, schema: &SchemaObject) -> std::io::Result<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(base_dir.join(format!("{name}.md")))?;
    write!(f, "<!-- Generated documentation, do not edit -->\n\n")?;
    // Title.
    write!(f, "# {name}\n\n")?;
    // Description paragraph.
    let descr = description(&schema);
    write!(f, "{descr}\n\n")?;
    if let Some(ObjectValidation {
        properties,
        additional_properties,
        ..
    }) = schema.object.as_deref()
    {
        // Struct.
        write!(f, "## Fields\n\n")?;
        struct_fields(&mut f, properties, additional_properties.is_some())?;
    } else if let Some(SubschemaValidation {
        one_of: Some(variants),
        ..
    }) = schema.subschemas.as_deref()
    {
        // Enum with rich variants.
        enum_variants(&mut f, variants)?;
    } else if let Some(values) = &schema.enum_values {
        // Simple enum: no data and no docs on individual items. Don't add options if the
        // description already has them because `json_schema` can add them too.
        if !descr.contains("## Values") {
            simple_enum_values(&mut f, values)?;
        }
    } else {
        // Something we don't understand.
        panic!("unknown schema: {schema:#?}");
    }
    // Code.
    schema_object_code(&mut f, schema)
}

fn number_colormap_range(
    base_dir: &Path,
    name: &str,
    schema: &SchemaObject,
) -> std::io::Result<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(base_dir.join(format!("{name}.md")))?;
    write!(f, "<!-- Generated documentation, do not edit -->\n\n")?;
    // Title.
    write!(f, "# {name}\n\n")?;
    // Description paragraph.
    let descr = description(&schema);
    write!(f, "{descr}\n\n")?;
    ty_defn_list_item(
        &mut f,
        "min",
        "double, integer, date, or date-time",
        "Minimum value of the range. The type should match the associated number array type.",
    )?;
    ty_defn_list_item(
        &mut f,
        "max",
        "double, integer, date, or date-time",
        "Maximum value of the range. Must have the same type as `min`.",
    )?;
    // Code.
    schema_object_code(&mut f, schema)
}

fn description(schema: &SchemaObject) -> String {
    use regex::{Captures, Regex};

    static RE: OnceLock<Regex> = OnceLock::new();
    let re =
        RE.get_or_init(|| Regex::new(r#"\]\(crate::(?<page>\w+)(::(?<anchor>\w+))?\)"#).unwrap());

    let Some(Metadata {
        description: Some(descr),
        ..
    }) = schema.metadata.as_deref()
    else {
        return String::new();
    };
    re.replace_all(descr, |caps: &Captures| {
        let page = caps.name("page").unwrap().as_str();
        if let Some(anchor) = caps.name("anchor") {
            format!("]({page}.md#{anchor})", anchor = anchor.as_str())
        } else {
            format!("]({page}.md)")
        }
    })
    .into_owned()
}

fn struct_fields(
    f: &mut impl Write,
    properties: &BTreeMap<String, Schema>,
    additional: bool,
) -> std::io::Result<()> {
    for (name, child) in properties {
        if name == "type" {
            continue;
        }
        if let Schema::Object(schema) = child {
            ty_defn_list_item(f, name, &field_type(schema), &description(schema))?;
        } else {
            defn_list_item(f, name, "")?;
        }
    }
    if additional {
        write!(f, "- Plus any custom values.\n\n")?;
    }
    Ok(())
}

fn simple_enum_values(f: &mut impl Write, values: &Vec<Value>) -> std::io::Result<()> {
    write!(f, "## Values\n\n")?;
    for value in values {
        if let Value::String(s) = value {
            let fixed = s.replace("\n\n", "\n\n    ");
            write!(f, "- {fixed}\n")?;
        } else {
            panic!("enum variant not a string: {value:#?}")
        }
    }
    write!(f, "\n")
}

fn enum_variants(f: &mut impl Write, variants: &Vec<Schema>) -> std::io::Result<()> {
    for variant in variants {
        if let Schema::Object(schema) = variant {
            let name = variant_name(schema);
            write!(f, "## {name}\n\n")?;
            write!(f, "{}\n\n", description(&schema))?;
            if let Some(ObjectValidation {
                properties,
                additional_properties,
                ..
            }) = schema.object.as_deref()
            {
                write!(f, "### Fields\n\n")?;
                defn_list_item(f, "type", &format!("`\"{name}\"`"))?;
                struct_fields(f, properties, additional_properties.is_some())?;
            }
        }
    }
    Ok(())
}

fn variant_name(schema: &SchemaObject) -> &str {
    if let Some(ObjectValidation { properties, .. }) = schema.object.as_deref() {
        for (name, value) in properties {
            if let Schema::Object(SchemaObject {
                enum_values: Some(v),
                ..
            }) = value
            {
                if name == "type" && v.len() == 1 {
                    return v[0].as_str().expect("string for enum type");
                }
            }
        }
    }
    panic!("expected enum variant: {schema:#?}");
}

fn field_type(schema: &SchemaObject) -> String {
    if let Some(ty) = &schema.reference {
        let name = ty.strip_prefix("#/definitions/").unwrap_or(ty);
        format!("[`{name}`]({name}.md)")
    } else if let Some(name) = known_type(schema) {
        name
    } else if let Some(name) = optional_type(schema) {
        name
    } else if let Some(name) = array_type(schema) {
        name
    } else if let Some(SingleOrVec::Single(t)) = &schema.instance_type {
        instance_type(**t)
            .unwrap_or_else(|| panic!("unknown array: {schema:#?}"))
            .to_owned()
    } else {
        panic!("unknown type: {schema:#?}");
    }
}

fn instance_type(t: InstanceType) -> Option<&'static str> {
    match t {
        InstanceType::Object => Some("object"),
        InstanceType::Null => Some("null"),
        InstanceType::Boolean => Some("bool"),
        InstanceType::Number => Some("number"),
        InstanceType::String => Some("string"),
        InstanceType::Integer => Some("integer"),
        InstanceType::Array => None,
    }
}

fn known_type(schema: &SchemaObject) -> Option<String> {
    fn no_meta(schema: &SchemaObject) -> SchemaObject {
        let mut copy = schema.clone();
        copy.metadata = None;
        copy
    }

    static TYPES: OnceLock<[(&str, SchemaObject); 3]> = OnceLock::new();
    let t = TYPES.get_or_init(|| {
        [
            (
                "RGB color (uint8, 3 items)",
                no_meta(&schema_for::<[u8; 3]>(true).schema),
            ),
            (
                "RGB color (uint8, 3 items) or null",
                no_meta(&schema_for::<Option<[u8; 3]>>(true).schema),
            ),
            ("3D vector", no_meta(&schema_for::<[f64; 3]>(true).schema)),
        ]
    });

    let s = no_meta(schema);
    t.iter().find_map(|(name, ty)| {
        if ty == &s {
            Some((*name).to_owned())
        } else {
            None
        }
    })
}

fn optional_type(schema: &SchemaObject) -> Option<String> {
    let null = SchemaObject {
        instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Null))),
        ..Default::default()
    };
    if let Some(SubschemaValidation {
        any_of: Some(items),
        ..
    }) = schema.subschemas.as_deref()
    {
        if let [Schema::Object(first), Schema::Object(second)] = &items[..] {
            if second == &null {
                return Some(format!("{} or null", field_type(first)));
            }
        }
    }
    if let Some(SingleOrVec::Vec(types)) = schema.instance_type.as_ref() {
        if let [first, InstanceType::Null] = &types[..] {
            let t = instance_type(*first)?;
            return Some(format!("{t} or null"));
        }
    }
    None
}

fn array_type(schema: &SchemaObject) -> Option<String> {
    if let Some(ArrayValidation {
        items: Some(SingleOrVec::Single(item)),
        additional_items: None,
        max_items,
        min_items,
        unique_items: None,
        contains: None,
    }) = schema.array.as_deref()
    {
        let Schema::Object(ty) = item.as_ref() else {
            panic!("not an object");
        };
        let ty = field_type(ty);
        match (min_items, max_items) {
            (None, None) => Some(format!("array of {ty}")),
            (None, Some(m)) => Some(format!("array of {ty}, up to {m} items")),
            (Some(n), None) => Some(format!("array of {ty}, at least {n} items")),
            (Some(n), Some(m)) if n == m => Some(format!("array of {ty}, {n} items")),
            (Some(n), Some(m)) => Some(format!("array of {ty}, {n} to {m} items")),
        }
    } else {
        None
    }
}

fn defn_list_item(f: &mut impl Write, title: &str, body: &str) -> std::io::Result<()> {
    let fixed_body = body.trim().replace("\n\n", "\n\n    ");
    write!(f, "`{title}`\n:   {fixed_body}\n\n")
}

fn ty_defn_list_item(f: &mut impl Write, title: &str, ty: &str, body: &str) -> std::io::Result<()> {
    let fixed_body = body.trim().replace("\n\n", "\n\n    ");
    write!(f, "`{title}`: {ty}\n:   {fixed_body}\n\n")
}

struct RemoveDescriptions {}

impl Visitor for RemoveDescriptions {
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        schema.metadata().description = None;
        schemars::visit::visit_schema_object(self, schema)
    }
}

fn schema_object_code(f: &mut impl Write, schema: &SchemaObject) -> std::io::Result<()> {
    write!(f, "## Schema\n\n")?;
    let mut no_descr = schema.clone();
    no_descr.metadata().description = None;
    visit_schema_object(&mut RemoveDescriptions {}, &mut no_descr);
    let code = serde_json::to_string_pretty(&no_descr).unwrap();
    write!(f, "```json\n{code}\n```\n")
}
