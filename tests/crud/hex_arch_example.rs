use canyon_sql::connection::DatabaseConnection;
use canyon_sql::core::Canyon;
use canyon_sql::macros::{CanyonCrud, CanyonMapper, canyon_entity};
use canyon_sql::query::querybuilder::SelectQueryBuilder;
use canyon_sql::runtime::tokio::sync::Mutex;
use std::error::Error;
use std::sync::Arc;

#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_hex_arch_find_all() {
    let default_db_conn = Canyon::instance()
        .unwrap()
        .get_default_connection()
        .unwrap();
    let league_service = LeagueServiceAdapter {
        league_repository: LeagueRepositoryAdapter {
            db_conn: default_db_conn,
        },
    };

    let find_all_result = league_service.find_all().await;

    // Connection doesn't return an error
    assert!(find_all_result.is_ok());
    let find_all_result = find_all_result.unwrap();
    assert!(!find_all_result.is_empty());
    // If we try to do a call using the adapter, count will use the default datasource, which is locked at this point,
    // since we passed the same connection that it will be using here to the repository!
    assert_eq!(
        LeagueRepositoryAdapter::<DatabaseConnection>::count()
            .await
            .unwrap() as usize,
        find_all_result.len()
    );
    // assert_eq!(LeagueRepositoryAdapter::<DatabaseConnection>::count_with(binding.deref_mut()).await.unwrap() as usize, find_all_result.len());
    // The line above works, because we're using binding, but in a better ideal world, our repository would hold an Arc<Mutex<...>> with the connection,
    // so the user acquire the lock on every query, just cloning the Arc, which if you remember, just increases in one unit the number of active
    // references pointing to the resource behind the atomic smart pointer
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
    async fn create<'a>(
        &self,
        league: &'a mut League,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>>;
} // As a domain boundary for the application side of the hexagon

pub struct LeagueServiceAdapter<T: LeagueRepository> {
    league_repository: T,
}
impl<T: LeagueRepository> LeagueService for LeagueServiceAdapter<T> {
    async fn find_all(&self) -> Result<Vec<League>, Box<dyn Error + Send + Sync>> {
        self.league_repository.find_all().await
    }

    async fn create<'a>(
        &self,
        league: &'a mut League,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        self.league_repository.create(league).await
    }
}

pub trait LeagueRepository {
    async fn find_all(&self) -> Result<Vec<League>, Box<dyn Error + Send + Sync>>;
    async fn create<'a>(
        &self,
        league: &'a mut League,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>>;
} // As a domain boundary for the infrastructure side of the hexagon

#[derive(CanyonCrud)]
#[canyon_crud(maps_to=League)]
pub struct LeagueRepositoryAdapter<T: DbConnection + Send + Sync> {
    // db_conn: &'b T,
    db_conn: Arc<Mutex<T>>,
}
impl<T: DbConnection + Send + Sync> LeagueRepository for LeagueRepositoryAdapter<T> {
    async fn find_all(&self) -> Result<Vec<League>, Box<dyn Error + Send + Sync>> {
        let db_conn = self.db_conn.lock().await;
        let select_query =
            SelectQueryBuilder::new("league", db_conn.get_database_type()?)?.build()?;
        db_conn.query(select_query, &[]).await
    }

    async fn create<'a>(
        &self,
        league: &'a mut League,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        Self::insert_entity(league).await
    }
}
