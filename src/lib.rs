use serde::{Serialize, Deserialize};

use std::collections::HashMap;

use std::sync::Arc;

mod value;
pub use value::{Value, ValueType};

#[ cfg(all(feature="json", feature="python-bindings")) ]
pub use value::{python_to_json, python_to_json_value};

pub type Tuple = Vec<(String, Value)>;

#[ derive(Serialize, Deserialize, Clone, Debug, PartialEq) ]
pub enum SchemaError {
    NoSuchField(String),
    EncodingError
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &*self {
            SchemaError::NoSuchField(fname) => { write!(fmt, "No such field: {}", fname) }
            SchemaError::EncodingError => { write!(fmt, "Failed to encode or decode data") }
        }
    }
}

impl std::error::Error for SchemaError {}

#[ derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize) ]
pub struct DataEntry {
    fields: Vec<Vec<u8>>
}

impl DataEntry {
    /// Construct the DataEntry directly from its raw fields
    /// It's the caller's responsibility to ensure these match the schema
    pub fn from_fields(fields: Vec<Vec<u8>>) -> Self {
        Self{ fields }
    }
}

type SchemaInner = Vec<(String, ValueType)>;

#[ derive(Debug) ]
pub struct Schema {
    key: ValueType,
    fields: Arc<SchemaInner>
}

impl Schema {
    pub fn from_parts(key: ValueType, fields: SchemaInner) -> Self {
        Self{ key, fields: Arc::new(fields) }
    }

    pub fn get_key_type(&self) -> ValueType {
        self.key
    }

    pub fn clone_inner(&self) -> (ValueType, SchemaInner) {
        (self.key, (*self.fields).clone())
    }

    pub fn set_field(&self, entry: &mut DataEntry, name: &str, value: &Value) -> Result<(), SchemaError> {
        if entry.fields.len() != self.fields.len() {
            return Err(SchemaError::EncodingError);
        }

        // FIXME typecheck here

        let bytes = value.serialize_inner();

        for (pos, (fname, _)) in self.fields.iter().enumerate() {
            if fname == name {
                *entry.fields.get_mut(pos).unwrap() = bytes;
                return Ok(());
            }
        }

        Err(SchemaError::NoSuchField(name.to_string()))
    }

    pub fn get_field(&self, entry: &DataEntry, name: &str) -> Result<Value, SchemaError> {
        if entry.fields.len() != self.fields.len() {
            return Err(SchemaError::EncodingError);
        }

        for (pos, (fname, ftype)) in self.fields.iter().enumerate() {
            if fname == name {
                let bytes = entry.fields.get(pos).unwrap();

                return match Value::from_bytes(bytes, ftype) {
                    Ok(v) => Ok(v),
                    Err(_) => {
                        log::error!("Failed to deserialize field of type {:?}", ftype);
                        Err(SchemaError::EncodingError)
                    }
                };
            }
        }

        Err(SchemaError::NoSuchField(name.to_string()))
    }

    pub fn get_fields(&self, entry: &DataEntry) -> Result<HashMap<String, Value>, SchemaError> {
        if entry.fields.len() != self.fields.len() {
            return Err(SchemaError::EncodingError);
        }

        let mut result = HashMap::new();

        for (pos, bytes) in entry.fields.iter().enumerate() {
            let (name, ftype) = self.fields.get(pos).unwrap();

            let value = match Value::from_bytes(bytes, ftype) {
                Ok(v) => v,
                Err(_) => {
                    log::error!("Failed to deserialize field of type {:?}", ftype);
                    return Err(SchemaError::EncodingError);
                }
            };

            result.insert(name.clone(), value);
        }

        Ok(result)
    }

    pub fn get_fields_with_filter(&self, entry: &DataEntry, filter: &[&str]) -> Result<HashMap<String, Value>, SchemaError> {
        if entry.fields.len() != filter.len() {
            return Err(SchemaError::EncodingError);
        }

        let mut result = HashMap::new();
        let mut filter_iter = filter.iter();

        for bytes in entry.fields.iter() {
            let name = filter_iter.next().expect("Filter length does not match entry length");

            let ftype = {
                let mut ftype = None;
                let mut fpos = 0;
                while ftype == None {
                    let (n, t) = self.fields.get(fpos).unwrap();

                    if n == name {
                       ftype = Some(t);
                    }
                    else {
                        fpos += 1;
                    }
                }

                ftype.expect("no such field")
            };

            let value = match Value::from_bytes(bytes, ftype) {
                Ok(v) => v,
                Err(_) => {
                    log::error!("Failed to deserialize field of type {:?}", ftype);
                    return Err(SchemaError::EncodingError);
                }
            };

            result.insert(name.to_string(), value);
        }

        Ok(result)
    }


    /// Same as get_fields but returns a vector instead
    pub fn get_fields_as_tuple(&self, entry: &DataEntry) -> Result<Tuple, SchemaError> {
        if entry.fields.len() != self.fields.len() {
            return Err(SchemaError::EncodingError);
        }

        let mut result = Vec::new();

        for (pos, bytes) in entry.fields.iter().enumerate() {
            let (name, ftype) = self.fields.get(pos).unwrap();

            let value = match Value::from_bytes(bytes, ftype) {
                Ok(v) => v,
                Err(_) => {
                    log::error!("Failed to deserialize field of type {:?}", ftype);
                    return Err(SchemaError::EncodingError);
                }
            };

            result.push((name.clone(), value));
        }

        Ok(result)
    }

    pub fn build_entry(&self) -> EntryBuilder {
        EntryBuilder{ fields: HashMap::new(), schema: self.fields.clone() }
    }
}

pub struct SchemaBuilder {
    key: ValueType,
    fields: SchemaInner
}

impl SchemaBuilder {
    pub fn new(key: ValueType) -> Self {
        Self{ key, fields: Vec::new() }
    }

    pub fn build(self) -> Schema {
        Schema{ key: self.key, fields: Arc::new(self.fields) }
    }

    pub fn add_field(mut self, name: String, vtype: ValueType) -> Self {
       for (fname, _) in self.fields.iter() {
            if &name == fname {
                panic!("Field defined more than once: {}", name);
            }
       }

       self.fields.push((name, vtype));

       self
    }
}

pub struct EntryBuilder {
    fields: HashMap<String, Vec<u8>>,
    schema: Arc<SchemaInner>
}

impl EntryBuilder {
    pub fn set_field<T: Serialize>(mut self, name: String, value: &T) -> Self {
        //TODO typecheck here
        
        let bytes = bincode::serialize(value).unwrap();
        self.fields.insert(name, bytes);

        self
    }

    pub fn set_field_from_value(mut self, name: String, value: &Value) -> Self {
        //TODO typecheck here

        let bytes = value.serialize_inner();
        self.fields.insert(name, bytes);

        self
    }

    pub fn done(mut self) -> DataEntry {
        let mut fields = Vec::new();

        for (fname, _ftype) in self.schema.iter() {
            let val = self.fields.remove(fname).expect("Field is missing");
            fields.push(val);
        }

        DataEntry{ fields }
    }
}

#[ cfg(test) ]
mod test {
    use super::*;

    fn test_init() {
         let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn get_is_set() {
        test_init();

        let schema = SchemaBuilder::new(ValueType::Bool)
            .add_field("value1".to_string(), ValueType::String)
            .add_field("value2".to_string(), ValueType::I64)
            .build();

        let mut entry = schema.build_entry()
            .set_field::<String>("value1".to_string(), &("foobar".to_string()))
            .set_field::<i64>("value2".to_string(), &42)
            .done();

        assert_eq!(schema.get_field(&entry, "value1").unwrap(), "foobar".into());
        assert_eq!(schema.get_field(&entry, "value2").unwrap(), 42.into());

        schema.set_field(&mut entry, "value1", &"foobaz".into()).unwrap();

        assert_eq!(schema.get_field(&entry, "value1").unwrap(), "foobaz".into());
        assert_eq!(schema.get_fields(&entry).unwrap().get("value1"), Some(&"foobaz".into()));
        assert_eq!(schema.get_fields_as_tuple(&entry).unwrap().get(1).unwrap().1, 42.into());
    }
}
