use canyon_core::connection::contracts::DbConnection;
use canyon_core::error::CanyonResult;
use canyon_core::mapper::RowMapper;
use canyon_core::query::bounds::EntityRuntimeInfo;

/// CRUD operations over an entity supplied to the operation.
///
/// It is intended for repository adapters and layered architectures where the persistence type is
/// not the entity being persisted.
pub trait EntityCrud: Send {
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

    fn update_entity<'entity>(
        entity: &'entity Self::Entity,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        Self::Entity: 'entity;

    fn update_entity_with<'entity, I>(
        entity: &'entity Self::Entity,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        Self::Entity: 'entity,
        I: DbConnection + Send + 'entity;

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
