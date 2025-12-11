pub fn offline_uuid(username: &str) -> uuid::Uuid {
	uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, username.as_bytes())
}
