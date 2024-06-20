use std::{collections::HashMap};

#[derive(Debug, Clone)]
pub struct Reads<T> {
    pub data: HashMap<String, T>,
}

impl<T> Default for Reads<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Reads<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::default(),
        }
    }

    pub fn insert(&mut self, key: String, value: T) -> Option<T> {
        self.data.insert(key, value)
    }

    pub fn remove(&mut self, key: &str) -> Option<T> {
        self.data.remove(key)
    }
}

// impl<T: Display + Clone> Display for Reads<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if self.data.len() == 1 {
//             write!(
//                 f,
//                 "{}",
//                 self.data.clone().into_values().collect::<Vec<_>>()[0]
//             )
//         } else {
//             for (key, value) in self.data.clone().into_iter() {
//                 write!(f, "{}: {}", key, value)?;
//             }
//             Ok(())
//         }
//     }
// }
