use crate::octra_accounts::{
	self, OctraAccountSession, OctraCommunitySnapshot,
};

pub async fn session() -> crate::Result<Option<OctraAccountSession>> {
	octra_accounts::session().await
}

pub async fn register(password: &str) -> crate::Result<OctraAccountSession> {
	octra_accounts::register(password).await
}

pub async fn login(username: &str, password: &str) -> crate::Result<OctraAccountSession> {
	octra_accounts::login(username, password).await
}

pub async fn logout() -> crate::Result<()> {
	octra_accounts::logout().await
}

pub async fn community() -> crate::Result<OctraCommunitySnapshot> {
	octra_accounts::community().await
}
