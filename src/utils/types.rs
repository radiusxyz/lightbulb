use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};

pub type ArcRwLock<T> = Arc<RwLock<T>>;
pub type ArcRwLockHashMap<K, V> = Arc<RwLock<HashMap<K, V>>>;
pub type ArcMutex<T> = Arc<Mutex<T>>;
pub type ArcMutexHashMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
