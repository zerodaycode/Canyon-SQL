use canyon_sql::connection::DatabaseConnector;
use canyon_sql::core::Canyon;
use canyon_sql::macros::{CanyonCrud, CanyonMapper, canyon_entity};
use canyon_sql::query::{QueryParameter, querybuilder::SelectQueryBuilder};
use std::error::Error;

#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_hex_arch_ops() {
    let default_db_conn = Canyon::instance()
        .unwrap()
        .get_default_connection()
        .unwrap();
    let league_service = LeagueHexServiceAdapter {
        league_repository: LeagueHexRepositoryAdapter {
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
        LeagueHexRepositoryAdapter::<DatabaseConnector>::count()
            .await
            .unwrap() as usize,
        find_all_result.len()
    );
}

#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_hex_arch_insert_entity_ops() {
    let default_db_conn = Canyon::instance()
        .unwrap()
        .get_default_connection()
        .unwrap();
    let league_service = LeagueHexServiceAdapter {
        league_repository: LeagueHexRepositoryAdapter {
            db_conn: default_db_conn,
        },
    };

    let mut other_league: LeagueHex = LeagueHex {
        id: Default::default(),
        ext_id: Default::default(),
        slug: "leaguehex-slug".to_string(),
        name: "Test LeagueHex on layered".to_string(),
        region: "LeagueHex Region".to_string(),
        image_url: "http://example.com/image.png".to_string(),
    };
    league_service.create(&mut other_league).await.unwrap();

    let find_new_league = league_service.get(&other_league.id).await.unwrap();
    assert!(find_new_league.is_some());
    assert_eq!(
        find_new_league.as_ref().unwrap().name,
        String::from("Test LeagueHex on layered")
    );
}

#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_hex_arch_update_entity_ops() {
    let default_db_conn = Canyon::instance()
        .unwrap()
        .get_default_connection()
        .unwrap();
    let league_service = LeagueHexServiceAdapter {
        league_repository: LeagueHexRepositoryAdapter {
            db_conn: default_db_conn,
        },
    };

    let mut other_league: LeagueHex = LeagueHex {
        id: Default::default(),
        ext_id: Default::default(),
        slug: "leaguehex-slug".to_string(),
        name: "Test LeagueHex on layered".to_string(),
        region: "LeagueHex Region".to_string(),
        image_url: "http://example.com/image.png".to_string(),
    };
    league_service.create(&mut other_league).await.unwrap();

    let find_new_league = league_service.get(&other_league.id).await.unwrap();
    assert!(find_new_league.is_some());
    assert_eq!(
        find_new_league.as_ref().unwrap().name,
        String::from("Test LeagueHex on layered")
    );

    let mut updt = find_new_league.unwrap();
    updt.ext_id = 5;
    let r = LeagueHexRepositoryAdapter::<DatabaseConnector>::update_entity(&updt).await;
    assert!(r.is_ok());

    let updated = league_service.get(&other_league.id).await.unwrap();
    assert_eq!(updated.unwrap().ext_id, 5);
}

#[derive(CanyonMapper, Debug)]
#[canyon_entity]
pub struct LeagueHex {
    // The core model of the 'LeagueHex' domain
    #[primary_key]
    pub id: i32,
    pub ext_id: i64,
    pub slug: String,
    pub name: String,
    pub region: String,
    pub image_url: String,
}

pub trait LeagueHexService {
    async fn find_all(&self) -> Result<Vec<LeagueHex>, Box<dyn Error + Send + Sync>>;
    async fn create<'a>(
        &self,
        league: &'a mut LeagueHex,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>>;

    async fn get<'a, Pk: QueryParameter>(
        &self,
        id: &'a Pk,
    ) -> Result<Option<LeagueHex>, Box<dyn Error + Send + Sync + 'a>>;
} // As a domain boundary for the application side of the hexagon

pub struct LeagueHexServiceAdapter<T: LeagueHexRepository> {
    league_repository: T,
}
impl<T: LeagueHexRepository> LeagueHexService for LeagueHexServiceAdapter<T> {
    async fn find_all(&self) -> Result<Vec<LeagueHex>, Box<dyn Error + Send + Sync>> {
        self.league_repository.find_all().await
    }

    async fn create<'a>(
        &self,
        league: &'a mut LeagueHex,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        self.league_repository.create(league).await
    }

    async fn get<'a, Pk: QueryParameter>(
        &self,
        id: &'a Pk,
    ) -> Result<Option<LeagueHex>, Box<dyn Error + Send + Sync + 'a>> {
        self.league_repository.get(id).await
    }
}

pub trait LeagueHexRepository {
    async fn find_all(&self) -> Result<Vec<LeagueHex>, Box<dyn Error + Send + Sync>>;
    async fn create<'a>(
        &self,
        league: &'a mut LeagueHex,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>>;

    async fn get<'a, Pk: QueryParameter>(
        &self,
        id: &'a Pk,
    ) -> Result<Option<LeagueHex>, Box<dyn Error + Send + Sync + 'a>>;
} // As a domain boundary for the infrastructure side of the hexagon

#[derive(CanyonCrud)]
#[canyon_crud(maps_to=LeagueHex)]
#[canyon_entity(table_name = "league")]
pub struct LeagueHexRepositoryAdapter<T: DbConnection + Send + Sync> {
    db_conn: T,
}
impl<T: DbConnection + Send + Sync> LeagueHexRepository for LeagueHexRepositoryAdapter<T> {
    async fn find_all(&self) -> Result<Vec<LeagueHex>, Box<dyn Error + Send + Sync>> {
        let db_conn = &self.db_conn;
        let select_query =
            SelectQueryBuilder::new("league", db_conn.get_database_type()?).build()?;
        db_conn.query(select_query, &[]).await
    }

    async fn create<'a>(
        &self,
        league: &'a mut LeagueHex,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'a>> {
        Self::insert_entity(league).await
    }

    async fn get<'a, Pk: QueryParameter>(
        &self,
        id: &'a Pk,
    ) -> Result<Option<LeagueHex>, Box<dyn Error + Send + Sync + 'a>> {
        Self::find_by_pk(id).await
    }
}
