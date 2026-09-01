use crate::octra_accounts::{self, OctraAccountSession};

pub async fn session() -> crate::Result<Option<OctraAccountSession>> {
	octra_accounts::session().await
}

pub async fn register(
	username: &str,
	password: &str,
	minecraft_nick: &str,
) -> crate::Result<OctraAccountSession> {
	octra_accounts::register(username, password, minecraft_nick).await
}

pub async fn login(username: &str, password: &str) -> crate::Result<OctraAccountSession> {
	octra_accounts::login(username, password).await
}

pub async fn logout() -> crate::Result<()> {
	octra_accounts::logout().await
}
