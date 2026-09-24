use canyon_core::connection::contracts::DbConnection;
use canyon_core::error::CanyonResult;
use canyon_core::mapper::RowMapper;
use canyon_core::query::bounds::EntityRuntimeInfo;

/// Inserts an entity supplied to a repository adapter.
pub trait EntityInsert: Send {
    /// Entity accepted by this adapter.
    type Entity: RowMapper + EntityRuntimeInfo + Sync;

    /// Inserts an entity through the default datasource.
    fn insert_entity<'entity>(
        entity: &'entity mut Self::Entity,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        Self::Entity: 'entity;

    /// Inserts an entity through the supplied connection or datasource.
    fn insert_entity_with<'entity, I>(
        entity: &'entity mut Self::Entity,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        Self::Entity: 'entity,
        I: DbConnection + Send + 'entity;
}

/// Updates an entity supplied to a repository adapter.
pub trait EntityUpdate: Send {
    /// Entity accepted by this adapter.
    type Entity: RowMapper + EntityRuntimeInfo + Sync;

    /// Updates an entity and returns the number of affected rows.
    fn update_entity<'entity>(
        entity: &'entity Self::Entity,
    ) -> impl Future<Output = CanyonResult<u64>>
    where
        Self::Entity: 'entity;

    /// Updates an entity through the supplied connection and returns the affected-row count.
    fn update_entity_with<'entity, I>(
        entity: &'entity Self::Entity,
        input: I,
    ) -> impl Future<Output = CanyonResult<u64>>
    where
        Self::Entity: 'entity,
        I: DbConnection + Send + 'entity;
}

/// Deletes an entity supplied to a repository adapter.
pub trait EntityDelete: Send {
    /// Entity accepted by this adapter.
    type Entity: RowMapper + EntityRuntimeInfo + Sync;

    /// Deletes an entity through the default datasource.
    fn delete_entity<'entity>(
        entity: &'entity Self::Entity,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        Self::Entity: 'entity;

    /// Deletes an entity through the supplied connection or datasource.
    fn delete_entity_with<'entity, I>(
        entity: &'entity Self::Entity,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        Self::Entity: 'entity,
        I: DbConnection + Send + 'entity;
}

/// Complete runtime entity CRUD contract.
///
/// This trait is implemented automatically for adapters that implement
/// [`EntityInsert`], [`EntityUpdate`] and [`EntityDelete`] for the same entity.
pub trait EntityCrud:
    EntityInsert<Entity = <Self as EntityCrud>::Entity>
    + EntityUpdate<Entity = <Self as EntityCrud>::Entity>
    + EntityDelete<Entity = <Self as EntityCrud>::Entity>
{
    /// Entity accepted by every operation in the composite contract.
    type Entity: RowMapper + EntityRuntimeInfo + Sync;
}

impl<T> EntityCrud for T
where
    T: EntityInsert,
    T: EntityUpdate<Entity = <T as EntityInsert>::Entity>,
    T: EntityDelete<Entity = <T as EntityInsert>::Entity>,
{
    type Entity = <T as EntityInsert>::Entity;
}
