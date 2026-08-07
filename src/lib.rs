//! Python-модуль для парсинга банковских выписок 1CClientBankExchange.

mod convert;
mod errors;
mod hooks;
mod types;

use odin_palace::parser::ParserBuilder;
use pyo3::prelude::*;

use crate::convert::convert_statement;
use crate::errors::convert_error;
use crate::types::Statement;

/// Извлекает байты из входных данных (bytes или str) без копирования.
fn extract_bytes<'py>(statement: &'py Bound<'py, PyAny>) -> PyResult<&'py [u8]> {
    if let Ok(s) = statement.extract::<&str>() {
        Ok(s.as_bytes())
    } else if let Ok(b) = statement.extract::<&[u8]>() {
        Ok(b)
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "statement must be bytes or str",
        ))
    }
}

/// Парсит выписку 1CClientBankExchange из байтов или строки.
///
/// # Аргументы
///
/// * `statement` - Сырые байты или строка файла выписки (поддерживает кодировки CP1251 и UTF-8)
/// * `hooks` - Опциональный список callable для модификации секций во время парсинга.
///   Сигнатура hook: `(section_type: SectionType, attrs: dict, header: dict) -> None`
///   - `section_type`: тип секции (`SectionType.Document` или `SectionType.Account`)
///   - attrs: словарь атрибутов секции (можно модифицировать)
///   - header: словарь заголовка выписки (только для чтения)
///
/// # Возвращает
///
/// Объект Statement с атрибутами:
/// - encoding: "cp1251" или "utf-8"
/// - header: словарь пар ключ-значение заголовка
/// - accounts: словарь номер счёта -> объект Account
/// - documents: список объектов Document
/// - warnings: список кортежей (lineno, message)
///
/// # Ошибки
///
/// * `NotStatementError` - Если входные данные не являются выпиской 1C
/// * `EmptyInputError` - Если входные данные пусты
/// * `UnfinishedError` - Если выписка не завершена корректно
/// * `SyntaxError` - Если в выписке синтаксическая ошибка (включая ошибки hooks)
/// * `ParseError` - Для прочих ошибок парсинга
#[pyfunction]
#[pyo3(signature = (statement, hooks=None))]
fn parse(
    py: Python<'_>,
    statement: &Bound<'_, PyAny>,
    hooks: Option<Vec<Py<PyAny>>>,
) -> PyResult<Statement> {
    let bytes = extract_bytes(statement)?;
    let hooks = hooks.unwrap_or_default();

    let result = if hooks.is_empty() {
        // Без хуков парсинг не обращается к интерпретатору — отпускаем GIL,
        // чтобы параллельные вызовы parse из потоков не блокировали друг друга.
        py.detach(|| ParserBuilder::new().build().parse(bytes))
    } else {
        let parser = ParserBuilder::new()
            .with_hooks(crate::hooks::build_hooks(hooks))
            .build();
        parser.parse(bytes)
    };

    let statement = result.map_err(|e| convert_error(py, e))?;
    convert_statement(py, statement)
}

#[pymodule]
fn odin_palace_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Классы данных
    m.add_class::<types::Interval>()?;
    m.add_class::<types::Account>()?;
    m.add_class::<types::Document>()?;
    m.add_class::<types::Statement>()?;

    // Enum типа секции для hooks
    m.add_class::<types::SectionType>()?;

    // Классы деталей синтаксических ошибок
    m.add_class::<errors::UnexpectedSection>()?;
    m.add_class::<errors::UnexpectedAttribute>()?;
    m.add_class::<errors::UnrecognizedLine>()?;
    m.add_class::<errors::MissingField>()?;
    m.add_class::<errors::AccountParseError>()?;
    m.add_class::<errors::DocumentParseError>()?;
    m.add_class::<errors::HookError>()?;

    // Функция парсинга
    m.add_function(wrap_pyfunction!(parse, m)?)?;

    // Исключения
    m.add("ParseError", m.py().get_type::<errors::ParseError>())?;
    m.add(
        "NotStatementError",
        m.py().get_type::<errors::NotStatementError>(),
    )?;
    m.add(
        "EmptyInputError",
        m.py().get_type::<errors::EmptyInputError>(),
    )?;
    m.add(
        "UnfinishedError",
        m.py().get_type::<errors::UnfinishedError>(),
    )?;
    m.add("SyntaxError", m.py().get_type::<errors::SyntaxError>())?;
    Ok(())
}
