//! Repository traits for database access
//!
//! This module defines traits for database repositories that can be implemented
//! by different database backends. This allows the connectify_db crate to be
//! completely agnostic of the specific database implementation.

use std::error::Error;
use std::fmt::Debug;

/// A trait for database repositories
///
/// This trait defines the basic operations that all database repositories
/// should support. It is generic over the entity type and the error type.
pub trait Repository<T, E>
where
    T: Clone + Debug,
    E: Error + Debug,
{
    /// Create a new entity in the repository
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to create
    ///
    /// # Returns
    ///
    /// The created entity, or an error if the operation failed
    fn create(&self, entity: T) -> impl std::future::Future<Output = Result<T, E>> + Send;

    /// Read an entity from the repository by ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the entity to read
    ///
    /// # Returns
    ///
    /// The entity if found, or None if not found, or an error if the operation failed
    fn read<I>(&self, id: I) -> impl std::future::Future<Output = Result<Option<T>, E>> + Send
    where
        I: Debug + Send + Sync;

    /// Update an entity in the repository
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to update
    ///
    /// # Returns
    ///
    /// The updated entity, or an error if the operation failed
    fn update(&self, entity: T) -> impl std::future::Future<Output = Result<T, E>> + Send;

    /// Delete an entity from the repository by ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the entity to delete
    ///
    /// # Returns
    ///
    /// `true` if the entity was deleted, `false` if it was not found,
    /// or an error if the operation failed
    fn delete<I>(&self, id: I) -> impl std::future::Future<Output = Result<bool, E>> + Send
    where
        I: Debug + Send + Sync;
}

/// A trait for database repository factories
///
/// This trait defines a factory for creating repository instances.
/// It is generic over the repository type and the configuration type.
pub trait RepositoryFactory<R, C> {
    /// Create a new repository instance
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the repository
    ///
    /// # Returns
    ///
    /// A new repository instance
    fn create_repository(&self, config: C) -> R;
}
