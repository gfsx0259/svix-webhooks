// SPDX-FileCopyrightText: © 2022 Svix Authors
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use chrono::Utc;
use http::Method;
use sea_orm::{ActiveValue::Set, entity::prelude::*};

use crate::core::types::MessageId;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "messagecontent")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: MessageId,
    pub created_at: DateTimeWithTimeZone,
    pub payload: Vec<u8>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub headers: Option<Json>,
    pub method: Option<String>,
    pub query: Option<String>,
    pub expiration: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::message::Entity",
        from = "Column::Id",
        to = "super::message::Column::Id"
    )]
    Message,
}

impl Related<super::message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Message.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new(
        msg_id: MessageId,
        payload: Vec<u8>,
        headers: Option<Json>,
        method: Option<String>,
        query: Option<String>,
        expiration: DateTimeWithTimeZone,
    ) -> Self {
        let timestamp = Utc::now();
        Self {
            id: Set(msg_id),
            created_at: Set(timestamp.into()),
            payload: Set(payload),
            headers: Set(headers),
            method: Set(method),
            query: Set(query),
            expiration: Set(expiration),
        }
    }
}

impl Model {
    pub fn parsed_headers(&self) -> HashMap<String, String> {
        self.headers
            .as_ref()
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }

    pub fn parsed_method(&self) -> Method {
        match self
            .method
            .as_deref()
            .map(|m| m.trim().to_ascii_uppercase())
            .as_deref()
        {
            Some("GET") => Method::GET,
            _ => Method::POST,
        }
    }

    pub fn parsed_query(&self) -> String {
        self.query
            .as_deref()
            .map(|q| q.trim().trim_start_matches('?'))
            .filter(|q| !q.is_empty())
            .unwrap_or_default()
            .to_owned()
    }
}

impl Entity {
    pub fn secure_find_by_id_in(ids: impl IntoIterator<Item = MessageId>) -> Select<Entity> {
        Self::find().filter(Column::Id.is_in(ids))
    }
}
