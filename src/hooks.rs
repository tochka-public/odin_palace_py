//! Обёртки Python-callable в хуки секций парсера.

use odin_palace::parser::hooks::{HookError as RustHookError, SectionHook};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::types::SectionType;

/// Оборачивает Python-callable в Rust-хуки секций.
///
/// Каждый хук на время вызова присоединяется к интерпретатору, отдаёт
/// callable изменяемую копию атрибутов секции и заголовок только для чтения,
/// после чего переносит изменения обратно.
pub fn build_hooks(hooks: Vec<Py<PyAny>>) -> Vec<Box<SectionHook>> {
    hooks
        .into_iter()
        .map(|py_hook| {
            let hook: Box<SectionHook> = Box::new(move |section_type, attrs, statement| {
                Python::attach(|py| {
                    let py_section_type = SectionType::from(section_type);

                    let py_attrs = PyDict::new(py);
                    for (k, v) in attrs.iter() {
                        py_attrs
                            .set_item(k, v)
                            .map_err(|e| RustHookError::Error(e.to_string()))?;
                    }

                    let py_header = PyDict::new(py);
                    for (k, v) in &statement.header {
                        py_header
                            .set_item(k, v)
                            .map_err(|e| RustHookError::Error(e.to_string()))?;
                    }

                    let result = py_hook.call(
                        py,
                        (py_section_type, py_attrs.as_any(), py_header.as_any()),
                        None,
                    );

                    match result {
                        Ok(_) => {
                            attrs.clear();
                            for (key, value) in py_attrs.iter() {
                                let k: String = key
                                    .extract::<String>()
                                    .map_err(|e| RustHookError::Error(e.to_string()))?;
                                let v: String = value
                                    .extract::<String>()
                                    .map_err(|e| RustHookError::Error(e.to_string()))?;
                                attrs.insert(k, v);
                            }
                            Ok(())
                        }
                        Err(e) => Err(RustHookError::Error(e.to_string())),
                    }
                })
            });
            hook
        })
        .collect()
}
