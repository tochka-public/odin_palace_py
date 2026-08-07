//! Исключения парсинга и типизированные detail-классы для `SyntaxError`.

use odin_palace::parser::{Error, ParserError, ParserErrorKind, SectionContext};
use pyo3::create_exception;
use pyo3::prelude::*;

// Базовое исключение для всех ошибок парсинга
create_exception!(odin_palace_py, ParseError, pyo3::exceptions::PyException);

// Специфичные типы ошибок.
// Имя SyntaxError перекрывает встроенный builtins.SyntaxError — менять нельзя,
// это опубликованный API; см. предупреждение в README.
create_exception!(odin_palace_py, NotStatementError, ParseError);
create_exception!(odin_palace_py, EmptyInputError, ParseError);
create_exception!(odin_palace_py, UnfinishedError, ParseError);
create_exception!(odin_palace_py, SyntaxError, ParseError);

/// Генерирует detail-класс с полями `lineno` и `message`.
macro_rules! detail_lineno_message {
    ($name:ident) => {
        #[pyclass(frozen, get_all, skip_from_py_object, module = "odin_palace_py")]
        #[derive(Clone)]
        pub struct $name {
            pub lineno: usize,
            pub message: String,
        }

        #[pymethods]
        impl $name {
            #[new]
            fn new(lineno: usize, message: String) -> Self {
                Self { lineno, message }
            }

            fn __getnewargs__(&self) -> (usize, String) {
                (self.lineno, self.message.clone())
            }

            fn __repr__(&self) -> String {
                format!(
                    "{}(lineno={}, message={:?})",
                    stringify!($name),
                    self.lineno,
                    self.message
                )
            }
        }
    };
}

/// Генерирует detail-класс с полями `lineno` и двумя именованными String-полями.
macro_rules! detail_lineno_two_strings {
    ($name:ident, $field1:ident, $field2:ident) => {
        #[pyclass(frozen, get_all, skip_from_py_object, module = "odin_palace_py")]
        #[derive(Clone)]
        pub struct $name {
            pub lineno: usize,
            pub $field1: String,
            pub $field2: String,
        }

        #[pymethods]
        impl $name {
            #[new]
            fn new(lineno: usize, $field1: String, $field2: String) -> Self {
                Self {
                    lineno,
                    $field1,
                    $field2,
                }
            }

            fn __getnewargs__(&self) -> (usize, String, String) {
                (self.lineno, self.$field1.clone(), self.$field2.clone())
            }

            fn __repr__(&self) -> String {
                format!(
                    concat!(
                        stringify!($name),
                        "(lineno={}, ",
                        stringify!($field1),
                        "={:?}, ",
                        stringify!($field2),
                        "={:?})"
                    ),
                    self.lineno, self.$field1, self.$field2
                )
            }
        }
    };
}

/// Генерирует detail-класс с полями `lineno` и одним именованным String-полем.
macro_rules! detail_lineno_one_string {
    ($name:ident, $field:ident) => {
        #[pyclass(frozen, get_all, skip_from_py_object, module = "odin_palace_py")]
        #[derive(Clone)]
        pub struct $name {
            pub lineno: usize,
            pub $field: String,
        }

        #[pymethods]
        impl $name {
            #[new]
            fn new(lineno: usize, $field: String) -> Self {
                Self { lineno, $field }
            }

            fn __getnewargs__(&self) -> (usize, String) {
                (self.lineno, self.$field.clone())
            }

            fn __repr__(&self) -> String {
                format!(
                    concat!(
                        stringify!($name),
                        "(lineno={}, ",
                        stringify!($field),
                        "={:?})"
                    ),
                    self.lineno, self.$field
                )
            }
        }
    };
}

detail_lineno_two_strings!(UnexpectedSection, found, context);
detail_lineno_two_strings!(UnexpectedAttribute, key, value);
detail_lineno_one_string!(UnrecognizedLine, line);
detail_lineno_two_strings!(MissingField, field, context);
detail_lineno_message!(AccountParseError);
detail_lineno_message!(DocumentParseError);
detail_lineno_message!(HookError);

/// Конвертирует ошибку парсера ядра в Python-исключение.
pub fn convert_error(py: Python<'_>, error: Error) -> PyErr {
    match error {
        Error::Not1CStatement => NotStatementError::new_err("Input is not a 1C statement"),
        Error::Empty => EmptyInputError::new_err("Input is empty"),
        Error::Unfinished => UnfinishedError::new_err("Statement is not finished"),
        Error::InvalidDocument => ParseError::new_err("Invalid document"),
        Error::Syntax(parser_error) => create_syntax_error(py, parser_error),
    }
}

fn section_context_to_str(context: &SectionContext) -> &'static str {
    match context {
        SectionContext::Header => "header",
        SectionContext::Document => "document",
        SectionContext::Account => "account",
        SectionContext::Finished => "finished",
        SectionContext::Init => "init",
        SectionContext::ReadNextSection => "read_next_section",
    }
}

fn syntax_error_with_detail<'a>(
    py: Python<'a>,
    msg: String,
    detail: impl IntoPyObject<'a>,
) -> PyErr {
    let err = SyntaxError::new_err(msg);
    if let Err(set_err) = err.value(py).setattr("detail", detail) {
        // Не прячем внутреннюю проблему: если detail не проставился,
        // возвращаем именно её.
        return set_err;
    }
    err
}

fn create_syntax_error(py: Python<'_>, parser_error: ParserError) -> PyErr {
    let lineno = parser_error.lineno;

    match parser_error.kind {
        ParserErrorKind::UnexpectedSection { found, context } => {
            let context_str = section_context_to_str(&context);
            let msg = format!("Unexpected section '{found}' in context '{context_str}'");
            let detail = UnexpectedSection {
                lineno,
                found,
                context: context_str.to_string(),
            };
            syntax_error_with_detail(py, msg, detail)
        }
        ParserErrorKind::UnexpectedAttribute { key, value } => {
            let msg = format!("Unexpected attribute '{key}' = '{value}'");
            syntax_error_with_detail(py, msg, UnexpectedAttribute { lineno, key, value })
        }
        ParserErrorKind::UnrecognizedLine { line } => {
            let msg = format!("Unrecognized line: '{line}'");
            syntax_error_with_detail(py, msg, UnrecognizedLine { lineno, line })
        }
        ParserErrorKind::MissingField { field, context } => {
            let context_str = section_context_to_str(&context);
            let msg = format!("Missing field '{field}' in context '{context_str}'");
            let detail = MissingField {
                lineno,
                field,
                context: context_str.to_string(),
            };
            syntax_error_with_detail(py, msg, detail)
        }
        ParserErrorKind::AccountParseError(message) => {
            let msg = format!("Account parse error: {message}");
            syntax_error_with_detail(py, msg, AccountParseError { lineno, message })
        }
        ParserErrorKind::DocumentParseError(message) => {
            let msg = format!("Document parse error: {message}");
            syntax_error_with_detail(py, msg, DocumentParseError { lineno, message })
        }
        ParserErrorKind::HookError(message) => {
            let msg = format!("Hook error: {message}");
            syntax_error_with_detail(py, msg, HookError { lineno, message })
        }
    }
}
