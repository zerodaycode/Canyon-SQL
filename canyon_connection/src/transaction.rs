/* pub trait DatabaseTransaction {
    /// The type of rows returned by the database.
    type Rows<T>;

    /// The type of query parameters.
    type QueryParam<'a>: QueryParameter<'a>;

    /// Perform a query against the database.
    async fn query<'a, S, Z, T>(
        stmt: S,
        params: Z,
        datasource_name: &'a str,
    ) -> Result<Self::Rows<T>, Box<dyn std::error::Error + Sync + Send + 'static>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a Self::QueryParam<'a>]> + Sync + Send + 'a,
        T: Sized;
}
*/
