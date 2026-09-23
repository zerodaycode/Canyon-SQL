use canyon_core::connection::contracts::DbConnection;
use canyon_core::error::CanyonResult;
use canyon_core::mapper::RowMapper;
use canyon_core::query::bounds::EntityRuntimeInfo;

/// CRUD operations over an entity supplied to the operation.
///
/// It is intended for repository adapters and layered architectures where the persistence type is
/// not the entity being persisted.
pub trait EntityCrud: Send {
    fn insert_entity<'entity, T>(entity: &'entity mut T) -> impl Future<Output = CanyonResult<()>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity;

    fn insert_entity_with<'entity, T, I>(
        entity: &'entity mut T,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity,
        I: DbConnection + Send + 'entity;

    fn update_entity<'entity, T>(entity: &'entity T) -> impl Future<Output = CanyonResult<()>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity;

    fn update_entity_with<'entity, T, I>(
        entity: &'entity T,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity,
        I: DbConnection + Send + 'entity;

    fn delete_entity<'entity, T>(entity: &'entity T) -> impl Future<Output = CanyonResult<()>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity;

    fn delete_entity_with<'entity, T, I>(
        entity: &'entity T,
        input: I,
    ) -> impl Future<Output = CanyonResult<()>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity,
        I: DbConnection + Send + 'entity;
}
