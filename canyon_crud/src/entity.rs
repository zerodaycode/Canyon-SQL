use canyon_core::connection::contracts::DbConnection;
use canyon_core::error::CanyonResult;
use canyon_core::mapper::RowMapper;
use canyon_core::query::bounds::EntityRuntimeInfo;

/// Inserts an entity supplied to a repository adapter.
pub trait EntityInsert: Send {
    type Entity: RowMapper + EntityRuntimeInfo + Sync;

    fn insert_entity<'entity>(
        entity: &'entity mut Self::Entity,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        Self::Entity: 'entity;

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
    type Entity: RowMapper + EntityRuntimeInfo + Sync;

    fn update_entity<'entity>(
        entity: &'entity Self::Entity,
    ) -> impl Future<Output = CanyonResult<u64>>
    where
        Self::Entity: 'entity;

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
    type Entity: RowMapper + EntityRuntimeInfo + Sync;

    fn delete_entity<'entity>(
        entity: &'entity Self::Entity,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        Self::Entity: 'entity;

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
