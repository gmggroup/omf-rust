use std::fmt::Write;

use schemars::{
    gen::SchemaSettings,
    schema::{
        InstanceType, Metadata, RootSchema, Schema, SchemaObject, SingleOrVec, SubschemaValidation,
    },
    visit::visit_schema_object,
    JsonSchema,
};
use serde_json::Value;

use crate::{format_full_name, Project};

use schemars::visit::Visitor;

fn simple_enum_variant(outer_schema: &Schema) -> Option<(String, String)> {
    if let Schema::Object(schema) = outer_schema {
        let Some([Value::String(variant)]) = schema.enum_values.as_deref() else {
            return None;
        };
        let Some(Metadata {
            description: Some(descr),
            ..
        }) = schema.metadata.as_deref()
        else {
            return None;
        };
        Some((variant.clone(), descr.clone()))
    } else {
        None
    }
}

#[derive(Debug, Clone, Default)]
struct TweakSchema {
    remove_descr: bool,
}

impl Visitor for TweakSchema {
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        // Add a maximum for uint8 values.
        if schema.format.as_deref() == Some("uint8") {
            schema.number().maximum = Some(255.0);
        }
        // Move descriptions of simple enum values into the parent.
        if let Some(SubschemaValidation {
            one_of: Some(variants),
            ..
        }) = schema.subschemas.as_deref()
        {
            if let Some(v) = variants
                .iter()
                .map(simple_enum_variant)
                .collect::<Option<Vec<_>>>()
            {
                schema.subschemas = None;
                schema.enum_values = Some(v.iter().map(|(name, _)| (&name[..]).into()).collect());
                schema.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::String)));
                let mut descr = schema.metadata().description.clone().unwrap_or_default();
                descr += "\n\n### Values\n\n";
                for (n, d) in v {
                    let body = d.replace("\n\n", "\n\n    ");
                    write!(&mut descr, "`{n}`\n:   {body}\n\n").unwrap();
                }
                schema.metadata().description = Some(descr);
            }
        }
        // Optionally remove descriptions. These get transformed into the documentation,
        // they're a bit too complex to be readable in the schema itself.
        if self.remove_descr {
            let mut empty = false;
            if let Some(m) = schema.metadata.as_deref_mut() {
                m.description = None;
                empty = m == &Metadata::default();
            }
            if empty {
                schema.metadata = None;
            }
        }
        // Change references to the generics Array_for_* to just Array.
        if let Some(r) = schema.reference.as_mut() {
            if r.starts_with("#/definitions/Array_for_") {
                *r = "#/definitions/Array".to_owned();
            }
        }
        // Then delegate to default implementation to visit any subschemas.
        visit_schema_object(self, schema);
    }
}

pub(crate) fn schema_for<T: JsonSchema>(remove_descr: bool) -> RootSchema {
    SchemaSettings::draft2019_09()
        .with_visitor(TweakSchema { remove_descr })
        .into_generator()
        .into_root_schema_for::<T>()
}

pub(crate) fn project_schema(remove_descr: bool) -> RootSchema {
    let mut root = schema_for::<Project>(remove_descr);
    root.schema.metadata().title = Some(format_full_name());
    root.schema.metadata().id = Some("https://github.com/seequent/omf2".to_owned());
    let array_def = root.definitions.get("Array_for_Boolean").unwrap().clone();
    root.definitions
        .retain(|name, _| !name.starts_with("Array_for"));
    root.definitions.insert("Array".to_owned(), array_def);
    root
}

pub fn json_schema() -> RootSchema {
    project_schema(true)
}

#[cfg(test)]
pub(crate) mod tests {
    use schemars::schema::RootSchema;

    use crate::schema::json_schema;

    const SCHEMA: &str = "omf.schema.json";

    #[ignore = "used to get schema"]
    #[test]
    fn update_schema() {
        std::fs::write(
            SCHEMA,
            serde_json::to_string_pretty(&json_schema())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        crate::schema_doc::update_schema_docs();
    }

    #[ignore = "used to get schema docs"]
    #[test]
    fn update_schema_docs() {
        crate::schema_doc::update_schema_docs();
        #[cfg(feature = "parquet")]
        crate::file::parquet::schemas::dump_parquet_schemas();
    }

    #[test]
    fn schema() {
        let schema = json_schema();
        let expected: RootSchema =
            serde_json::from_reader(std::fs::File::open(SCHEMA).unwrap()).unwrap();
        assert!(schema == expected, "schema has changed");
    }
}
