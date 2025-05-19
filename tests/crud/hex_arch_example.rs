use canyon_sql::core::Canyon;
use canyon_sql::macros::{CanyonCrud, CanyonMapper, canyon_entity};
use canyon_sql::query::querybuilder::SelectQueryBuilder;
use std::error::Error;

#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_hex_arch_find_all() {
    let binding = Canyon::instance()
        .unwrap()
        .get_default_connection()
        .unwrap()
        .lock()
        .await;
    let league_service = LeagueServiceAdapter {
        league_repository: LeagueRepositoryAdapter {
            db_conn: binding.postgres_connection(),
        },
    };
    let find_all_result = league_service.find_all().await;

    // Connection doesn't return an error
    assert!(find_all_result.is_ok());
    assert!(!find_all_result.unwrap().is_empty());
}

#[derive(CanyonMapper)]
#[canyon_entity]
pub struct League {
    // The core model of the 'League' domain
    #[primary_key]
    pub id: i32,
}

pub trait LeagueService {
    async fn find_all(&self) -> Result<Vec<League>, Box<dyn Error + Send + Sync>>;
} // As a domain boundary for the application side of the hexagon

pub struct LeagueServiceAdapter<T: LeagueRepository> {
    league_repository: T,
}
impl<T: LeagueRepository> LeagueService for LeagueServiceAdapter<T> {
    async fn find_all(&self) -> Result<Vec<League>, Box<dyn Error + Send + Sync>> {
        self.league_repository.find_all().await
    }
}

pub trait LeagueRepository {
    async fn find_all(&self) -> Result<Vec<League>, Box<dyn Error + Send + Sync>>;
} // As a domain boundary for the infrastructure side of the hexagon

#[derive(CanyonCrud)]
#[canyon_crud(maps_to=League)]
pub struct LeagueRepositoryAdapter<'b, T: DbConnection + Send + Sync> {
    db_conn: &'b T,
}
impl<T: DbConnection + Send + Sync> LeagueRepository for LeagueRepositoryAdapter<'_, T> {
    async fn find_all(&self) -> Result<Vec<League>, Box<dyn Error + Send + Sync>> {
        let select_query =
            SelectQueryBuilder::new("league", self.db_conn.get_database_type()?)?.build()?;
        self.db_conn.query(select_query, &[]).await
    }
}
