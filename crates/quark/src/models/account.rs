use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct, Default)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Account {
    /// User id of the owner
    #[serde(rename = "_id")]
    pub id: String,
    
    pub email: String,

}
