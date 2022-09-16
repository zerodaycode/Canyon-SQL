use serde::Deserialize;


/// ```
#[test]
fn load_ds_config_from_array() {
    const CONFIG_FILE_MOCK_ALT: &'static str = r#"
        [canyon_sql]
        datasources = [
            {name = 'PostgresDS', properties.db_type = 'postgresql', properties.username = 'username', properties.password = 'random_pass', properties.host = 'localhost', properties.db_name = 'triforce'},
            {name = 'SqlServerDS', properties.db_type = 'sqlserver', properties.username = 'username2', properties.password = 'random_pass2', properties.host = '192.168.0.250.1:3340', properties.db_name = 'triforce2'}
        ]
    "#;

    let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT)
        .expect("A failure happened retrieving the [canyon_sql] section");

        let ds_0 = &config.canyon_sql.datasources[0];
        let ds_1 = &config.canyon_sql.datasources[1];
            
        assert_eq!(ds_0.name, "PostgresDS");
        assert_eq!(ds_0.properties.db_type, "postgresql");
        assert_eq!(ds_0.properties.username, "username");
        assert_eq!(ds_0.properties.password, "random_pass");
        assert_eq!(ds_0.properties.host, "localhost");
        assert_eq!(ds_0.properties.db_name, "triforce");

        assert_eq!(ds_1.name, "SqlServerDS");
        assert_eq!(ds_1.properties.db_type, "sqlserver");
        assert_eq!(ds_1.properties.username, "username2");
        assert_eq!(ds_1.properties.password, "random_pass2");
        assert_eq!(ds_1.properties.host, "192.168.0.250.1:3340");
        assert_eq!(ds_1.properties.db_name, "triforce2");
}

/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct CanyonSqlConfig<'a> {
    #[serde(borrow)]
    pub canyon_sql: Datasources<'a>
}
#[derive(Deserialize, Debug, Clone)]
pub struct Datasources<'a> {
    #[serde(borrow)]
    pub datasources: Vec<DatasourceConfig<'a>>
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DatasourceConfig<'a> {
    #[serde(borrow)]
    pub name: &'a str, 
    pub properties: DatasourceProperties<'a>
} 

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DatasourceProperties<'a> {
    pub db_type: &'a str,  
    pub username: &'a str, 
    pub password: &'a str,
    pub host: &'a str,
    pub port: u16,
    pub db_name: &'a str,
}