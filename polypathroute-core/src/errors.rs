// Unified error definitions 



pub struct ConfigError;
pub struct CacheError;
pub struct NetworkError;
pub struct DataError;
pub struct GraphError;


pub enum Errors {
    ConfigError,
    CacheError,
    NetworkError,
    DataError,
    GraphError
}