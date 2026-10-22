use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(String);

impl EntityId {
    pub fn generate() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_uuid_v7(&self) -> bool {
        Uuid::parse_str(&self.0)
            .ok()
            .and_then(|id| id.get_version())
            == Some(uuid::Version::SortRand)
    }
}
