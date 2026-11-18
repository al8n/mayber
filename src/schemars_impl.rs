use crate::Maybe;
use schemars_1::{json_schema, JsonSchema, Schema, SchemaGenerator};
use std::{borrow::Cow, format};

impl<R: JsonSchema, T: JsonSchema> JsonSchema for Maybe<R, T> {
  fn schema_name() -> Cow<'static, str> {
    format!("Maybe_{}_{}", R::schema_name(), T::schema_name()).into()
  }

  fn schema_id() -> Cow<'static, str> {
    format!("mayber::Maybe<{},{}>", R::schema_id(), T::schema_id()).into()
  }

  fn json_schema(generator: &mut SchemaGenerator) -> Schema {
    let owned = generator.subschema_for::<T>();
    if generator.contract().is_deserialize() {
      return owned;
    }
    let borrowed = generator.subschema_for::<R>();
    if borrowed == owned {
      owned
    } else {
      json_schema!({ "anyOf": [borrowed, owned] })
    }
  }
}
