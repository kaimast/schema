use crate::{DataEntry, FieldTypeList, Schema, Value, ValueType};

use std::collections::HashMap;

use serde::Serialize;

pub struct SchemaBuilder {
    key: ValueType,
    fields: FieldTypeList,
}

impl SchemaBuilder {
    pub fn new(key: ValueType) -> Self {
        Self {
            key,
            fields: Vec::new(),
        }
    }

    #[must_use]
    pub fn build(self) -> Schema {
        Schema {
            key: self.key,
            fields: self.fields,
        }
    }

    #[must_use]
    pub fn add_field<S: ToString>(mut self, name: S, vtype: ValueType) -> Self {
        let name = name.to_string();

        for (fname, _) in self.fields.iter() {
            if &name == fname {
                panic!("Field defined more than once: {}", name);
            }
        }

        self.fields.push((name, vtype));

        self
    }
}

pub struct EntryBuilder<'a> {
    fields: HashMap<&'a str, Vec<u8>>,
    schema: &'a FieldTypeList,
}

impl<'a> EntryBuilder<'a> {
    pub(crate) fn new(schema: &'a FieldTypeList) -> Self {
        Self {
            fields: HashMap::new(),
            schema,
        }
    }

    #[must_use]
    pub fn set_field<T: Serialize>(mut self, name: &'a str, value: &T) -> Self {
        //TODO typecheck here

        let bytes = bincode::serialize(value).unwrap();
        self.fields.insert(name, bytes);

        self
    }

    #[must_use]
    pub fn set_field_from_value(mut self, name: &'a str, value: &Value) -> Self {
        //TODO typecheck here

        let bytes = value.serialize_inner();
        self.fields.insert(name, bytes);

        self
    }

    #[must_use]
    pub fn build(mut self) -> DataEntry {
        let mut fields = Vec::new();

        for (fname, _ftype) in self.schema.iter() {
            let val = self
                .fields
                .remove(fname.as_str())
                .expect("Field is missing");
            fields.push(val);
        }

        DataEntry { fields }
    }
}
