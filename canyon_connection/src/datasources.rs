use serde::Deserialize;

use crate::canyon_database_connector::DatabaseType;

/// ```
#[test]
fn load_ds_config_from_array() {
    #[cfg(feature = "postgres")]
    {
        const CONFIG_FILE_MOCK_ALT_PG: &str = r#"
        [canyon_sql]
        datasources = [
            {name = 'PostgresDS', auth = { postgresql = { basic = { username = "postgres", password = "postgres" } } }, properties.host = 'localhost', properties.db_name = 'triforce', properties.migrations='enabled' },
        ]
        "#;
        let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT_PG)
            .expect("A failure happened retrieving the [canyon_sql] section");

        let ds_0 = &config.canyon_sql.datasources[0];

        assert_eq!(ds_0.name, "PostgresDS");
        assert_eq!(ds_0.get_db_type(), DatabaseType::PostgreSql);
        assert_eq!(
            ds_0.auth,
            Auth::Postgres(PostgresAuth::Basic {
                username: "postgres".to_string(),
                password: "postgres".to_string()
            })
        );
        assert_eq!(ds_0.properties.host, "localhost");
        assert_eq!(ds_0.properties.port, None);
        assert_eq!(ds_0.properties.db_name, "triforce");
        assert_eq!(ds_0.properties.migrations, Some(Migrations::Enabled));
    }

    #[cfg(feature = "mssql")]
    {
        const CONFIG_FILE_MOCK_ALT_MSSQL: &str = r#"
        [canyon_sql]
        datasources = [
            {name = 'SqlServerDS', auth = { sqlserver = { basic = { username = "sa", password = "SqlServer-10" } } }, properties.host = '192.168.0.250.1', properties.port = 3340, properties.db_name = 'triforce2', properties.migrations='disabled' },
            {name = 'SqlServerDS', auth = { sqlserver = { integrated = {} } }, properties.host = '192.168.0.250.1', properties.port = 3340, properties.db_name = 'triforce2', properties.migrations='disabled' }
        ]
        "#;
        let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT_MSSQL)
            .expect("A failure happened retrieving the [canyon_sql] section");

        let ds_1 = &config.canyon_sql.datasources[0];
        let ds_2 = &config.canyon_sql.datasources[1];

        assert_eq!(ds_1.name, "SqlServerDS");
        assert_eq!(ds_1.get_db_type(), DatabaseType::SqlServer);
        assert_eq!(
            ds_1.auth,
            Auth::SqlServer(SqlServerAuth::Basic {
                username: "sa".to_string(),
                password: "SqlServer-10".to_string()
            })
        );
        assert_eq!(ds_1.properties.host, "192.168.0.250.1");
        assert_eq!(ds_1.properties.port, Some(3340));
        assert_eq!(ds_1.properties.db_name, "triforce2");
        assert_eq!(ds_1.properties.migrations, Some(Migrations::Disabled));

        assert_eq!(ds_2.auth, Auth::SqlServer(SqlServerAuth::Integrated));
    }
    #[cfg(feature = "mysql")]
    {
        const CONFIG_FILE_MOCK_ALT_MYSQL: &str = r#"
        [canyon_sql]
        datasources = [
            {name = 'MysqlDS', auth = { mysql = { basic = { username = "root", password = "root" } } }, properties.host = '192.168.0.250.1', properties.port = 3340, properties.db_name = 'triforce2', properties.migrations='disabled' }
        ]
        "#;
        let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT_MYSQL)
            .expect("A failure happened retrieving the [canyon_sql] section");

        let ds_1 = &config.canyon_sql.datasources[0];

        assert_eq!(ds_1.name, "MysqlDS");
        assert_eq!(ds_1.get_db_type(), DatabaseType::MySQL);
        assert_eq!(
            ds_1.auth,
            Auth::MySQL(MySQLAuth::Basic {
                username: "root".to_string(),
                password: "root".to_string()
            })
        );
        assert_eq!(ds_1.properties.host, "192.168.0.250.1");
        assert_eq!(ds_1.properties.port, Some(3340));
        assert_eq!(ds_1.properties.db_name, "triforce2");
        assert_eq!(ds_1.properties.migrations, Some(Migrations::Disabled));
    }
}
///
#[derive(Deserialize, Debug, Clone)]
pub struct CanyonSqlConfig {
    pub canyon_sql: Datasources,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Datasources {
    pub datasources: Vec<DatasourceConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DatasourceConfig {
    pub name: String,
    pub auth: Auth,
    pub properties: DatasourceProperties,
}

impl DatasourceConfig {
    pub fn get_db_type(&self) -> DatabaseType {
        match self.auth {
            #[cfg(feature = "postgres")]
            Auth::Postgres(_) => DatabaseType::PostgreSql,
            #[cfg(feature = "mssql")]
            Auth::SqlServer(_) => DatabaseType::SqlServer,
            #[cfg(feature = "postgres")]
            Auth::MySQL(_) => DatabaseType::MySQL,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum Auth {
    #[serde(alias = "PostgreSQL", alias = "postgresql", alias = "postgres")]
    #[cfg(feature = "postgres")]
    Postgres(PostgresAuth),
    #[serde(alias = "SqlServer", alias = "sqlserver", alias = "mssql")]
    #[cfg(feature = "mssql")]
    SqlServer(SqlServerAuth),
    #[serde(alias = "MYSQL", alias = "mysql", alias = "MySQL")]
    #[cfg(feature = "mysql")]
    MySQL(MySQLAuth),
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[cfg(feature = "postgres")]
pub enum PostgresAuth {
    #[serde(alias = "Basic", alias = "basic")]
    Basic { username: String, password: String },
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[cfg(feature = "mssql")]
pub enum SqlServerAuth {
    #[serde(alias = "Basic", alias = "basic")]
    Basic { username: String, password: String },
    #[serde(alias = "Integrated", alias = "integrated")]
    Integrated,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[cfg(feature = "mysql")]
pub enum MySQLAuth {
    #[serde(alias = "Basic", alias = "basic")]
    Basic { username: String, password: String },
}

#[derive(Deserialize, Debug, Clone)]
pub struct DatasourceProperties {
    pub host: String,
    pub port: Option<u16>,
    pub db_name: String,
    pub migrations: Option<Migrations>,
}

/// Represents the enabled or disabled migrations for a whole datasource
#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Migrations {
    #[serde(alias = "Enabled", alias = "enabled")]
    Enabled,
    #[serde(alias = "Disabled", alias = "disabled")]
    Disabled,
}
