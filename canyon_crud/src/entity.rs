use canyon_core::connection::contracts::DbConnection;
use canyon_core::mapper::RowMapper;
use canyon_core::query::bounds::EntityRuntimeInfo;
use std::error::Error;

/// CRUD operations over an entity supplied to the operation.
///
/// This contract is separate from [`CrudOperations`]. It is intended for
/// repository adapters and layered architectures where the persistence type is
/// not the entity being persisted.
pub trait EntityCrudOperations: Send {
    fn insert_entity<'entity, 'error, T>(
        entity: &'entity mut T,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'error>>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity;

    fn insert_entity_with<'entity, 'error, T, I>(
        entity: &'entity mut T,
        input: I,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'error>>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity,
        I: DbConnection + Send + 'entity;

    fn update_entity<'entity, 'error, T>(
        entity: &'entity T,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'error>>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity;

    fn update_entity_with<'entity, 'error, T, I>(
        entity: &'entity T,
        input: I,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'error>>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity,
        I: DbConnection + Send + 'entity;

    fn delete_entity<'entity, 'error, T>(
        entity: &'entity T,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'error>>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity;

    fn delete_entity_with<'entity, 'error, T, I>(
        entity: &'entity T,
        input: I,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'error>>>
    where
        T: RowMapper + EntityRuntimeInfo + Sync + 'entity,
        I: DbConnection + Send + 'entity;
}
