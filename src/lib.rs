// SPDX-License-Identifier: Apache-2.0
pub mod generator;
pub mod grammar;
pub mod model;
pub mod parser;

// Re-export key components for backward compatibility
pub use model::config_model;
pub use model::model_builder;
pub use model::source_location;
pub use parser::cola;
pub use parser::cola_actions;
