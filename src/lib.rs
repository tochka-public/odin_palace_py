use odin_palace::parser::{
    hooks::{HookError as RustHookError, SectionType as RustSectionType},
    Document as RustDocument, Encoding, Error, ParserBuilder, ParserError, ParserErrorKind,
    SectionContext, Statement as RustStatement,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyDict;

// Базовое исключение для всех ошибок парсинга
create_exception!(odin_palace_py, ParseError, pyo3::exceptions::PyException);

// Специфичные типы ошибок
create_exception!(odin_palace_py, NotStatementError, ParseError);
create_exception!(odin_palace_py, EmptyInputError, ParseError);
create_exception!(odin_palace_py, UnfinishedError, ParseError);
create_exception!(odin_palace_py, SyntaxError, ParseError);

// ============================================================================
// Enum для типа секции (используется в hooks)
// ============================================================================

/// Тип секции выписки для использования в hooks.
#[pyclass(frozen, eq, skip_from_py_object, module = "odin_palace_py")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    /// Секция документа
    Document,
    /// Секция счёта
    Account,
}

#[pymethods]
impl SectionType {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn __repr__(&self) -> &'static str {
        match self {
            Self::Document => "SectionType.Document",
            Self::Account => "SectionType.Account",
        }
    }
}

impl From<RustSectionType> for SectionType {
    fn from(value: RustSectionType) -> Self {
        match value {
            RustSectionType::Document => Self::Document,
            RustSectionType::Account => Self::Account,
        }
    }
}

// ============================================================================
// Классы PyO3
// ============================================================================

#[pyclass(frozen, eq, get_all, skip_from_py_object)]
#[derive(Clone, PartialEq)]
pub struct Interval {
    pub date_start: String,
    pub date_end: Option<String>,
    pub start_amount: String,
    pub end_amount: Option<String>,
    pub total_income: Option<String>,
    pub total_expense: Option<String>,
}

#[pymethods]
impl Interval {
    fn __repr__(&self) -> String {
        format!(
            "Interval(date_start={:?}, date_end={:?}, start_amount={:?}, end_amount={:?})",
            self.date_start, self.date_end, self.start_amount, self.end_amount
        )
    }
}

#[pyclass(frozen, eq, get_all, skip_from_py_object)]
#[derive(Clone, PartialEq)]
pub struct Account {
    pub number: String,
    pub intervals: Vec<Interval>,
}

#[pymethods]
impl Account {
    fn __repr__(&self) -> String {
        format!(
            "Account(number={:?}, intervals=[...{}])",
            self.number,
            self.intervals.len()
        )
    }
}

#[pyclass(frozen, eq, get_all, skip_from_py_object)]
#[derive(Clone, PartialEq)]
pub struct Document {
    pub doc_number: String,
    pub doc_type: String,
    pub doc_date: String,
    pub amount: String,
    pub purpose: String,
    pub payment_type: Option<String>,
    pub ordering: Option<String>,
    pub counterparty_inn: String,
    pub counterparty_kpp: Option<String>,
    pub counterparty_bic: String,
    pub counterparty_bank1: String,
    pub counterparty_account: String,
    pub counterparty: Option<String>,
    pub counterparty_1: Option<String>,
    pub counterparty_ras_account: Option<String>,
    pub counterparty_cor_account: Option<String>,
    pub outcome_date: Option<String>,
    pub payee_inn: String,
    pub payee: Option<String>,
    pub payee_account: String,
    pub payee_kpp: Option<String>,
    pub payee_bic: String,
    pub payee_bank1: String,
    pub payee_ras_account: Option<String>,
    pub payee_cor_account: Option<String>,
    pub income_date: Option<String>,
}

#[pymethods]
impl Document {
    fn __repr__(&self) -> String {
        format!(
            "Document(doc_number={:?}, doc_type={:?}, doc_date={:?}, amount={:?})",
            self.doc_number, self.doc_type, self.doc_date, self.amount
        )
    }
}

#[pyclass(frozen)]
pub struct Statement {
    encoding: &'static str,
    header: Py<PyAny>,
    accounts: Py<PyAny>,
    documents: Py<PyAny>,
    documents_len: usize,
    warnings: Py<PyAny>,
    warnings_len: usize,
}

#[pymethods]
impl Statement {
    #[getter]
    fn encoding(&self) -> &'static str {
        self.encoding
    }

    #[getter]
    fn header(&self, py: Python<'_>) -> Py<PyAny> {
        self.header.clone_ref(py)
    }

    #[getter]
    fn accounts(&self, py: Python<'_>) -> Py<PyAny> {
        self.accounts.clone_ref(py)
    }

    #[getter]
    fn documents(&self, py: Python<'_>) -> Py<PyAny> {
        self.documents.clone_ref(py)
    }

    #[getter]
    fn warnings(&self, py: Python<'_>) -> Py<PyAny> {
        self.warnings.clone_ref(py)
    }

    fn __repr__(&self) -> String {
        format!(
            "Statement(encoding={:?}, documents=[...{}], warnings=[...{}])",
            self.encoding, self.documents_len, self.warnings_len
        )
    }
}

// ============================================================================
// Классы деталей SyntaxError
// ============================================================================

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

// ============================================================================
// Функции конвертации
// ============================================================================

fn encoding_to_str(encoding: Encoding) -> &'static str {
    match encoding {
        Encoding::Cp1251 => "cp1251",
        Encoding::Utf8 => "utf-8",
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
    if let Ok(dict) = err.value(py).getattr("__dict__") {
        let _ = dict.set_item("detail", detail);
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

fn convert_error(py: Python<'_>, error: Error) -> PyErr {
    match error {
        Error::Not1CStatement => NotStatementError::new_err("Input is not a 1C statement"),
        Error::Empty => EmptyInputError::new_err("Input is empty"),
        Error::Unfinished => UnfinishedError::new_err("Statement is not finished"),
        Error::InvalidDocument => ParseError::new_err("Invalid document"),
        Error::Syntax(parser_error) => create_syntax_error(py, parser_error),
    }
}

fn convert_document(doc: RustDocument) -> Document {
    Document {
        doc_number: doc.doc_number,
        doc_type: doc.doc_type,
        doc_date: doc.doc_date.to_string(),
        amount: doc.amount.to_string(),
        purpose: doc.purpose,
        payment_type: doc.payment_type,
        ordering: doc.ordering,
        counterparty_inn: doc.counterparty_inn,
        counterparty_kpp: doc.counterparty_kpp,
        counterparty_bic: doc.counterparty_bic,
        counterparty_bank1: doc.counterparty_bank1,
        counterparty_account: doc.counterparty_account,
        counterparty: doc.counterparty,
        counterparty_1: doc.counterparty_1,
        counterparty_ras_account: doc.counterparty_ras_account,
        counterparty_cor_account: doc.counterparty_cor_account,
        outcome_date: doc.outcome_date.map(|d| d.to_string()),
        payee_inn: doc.payee_inn,
        payee: doc.payee,
        payee_account: doc.payee_account,
        payee_kpp: doc.payee_kpp,
        payee_bic: doc.payee_bic,
        payee_bank1: doc.payee_bank1,
        payee_ras_account: doc.payee_ras_account,
        payee_cor_account: doc.payee_cor_account,
        income_date: doc.income_date.map(|d| d.to_string()),
    }
}

fn convert_statement(py: Python<'_>, statement: RustStatement) -> PyResult<Statement> {
    // Конвертируем заголовок в PyDict (перемещаем строки без клонирования)
    let header_dict = PyDict::new(py);
    for (k, v) in statement.header {
        header_dict.set_item(k, v)?;
    }

    // Конвертируем счета в PyDict с объектами Account
    let accounts_dict = PyDict::new(py);
    for (key, account) in statement.accounts {
        let intervals: Vec<Interval> = account
            .intervals
            .into_iter()
            .map(|interval| Interval {
                date_start: interval.date_start.to_string(),
                date_end: interval.date_end.map(|d| d.to_string()),
                start_amount: interval.start_amount.to_string(),
                end_amount: interval.end_amount.map(|d| d.to_string()),
                total_income: interval.total_income.map(|d| d.to_string()),
                total_expense: interval.total_expense.map(|d| d.to_string()),
            })
            .collect();

        let py_account = Account {
            number: account.number,
            intervals,
        };
        accounts_dict.set_item(key, py_account)?;
    }

    // Конвертируем документы в PyList
    let documents: Vec<Document> = statement
        .documents
        .into_iter()
        .map(convert_document)
        .collect();
    let documents_len = documents.len();
    let documents_list = pyo3::types::PyList::new(py, documents)?;

    // Конвертируем warnings в PyList
    let warnings_len = statement.warnings.len();
    let warnings_list = pyo3::types::PyList::new(py, statement.warnings)?;

    Ok(Statement {
        encoding: encoding_to_str(statement.encoding),
        header: header_dict.unbind().into(),
        accounts: accounts_dict.unbind().into(),
        documents: documents_list.unbind().into(),
        documents_len,
        warnings: warnings_list.unbind().into(),
        warnings_len,
    })
}

/// Извлекает байты из входных данных (bytes или str).
fn extract_bytes(statement: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    if let Ok(s) = statement.extract::<&str>() {
        Ok(s.as_bytes().to_vec())
    } else if let Ok(b) = statement.extract::<&[u8]>() {
        Ok(b.to_vec())
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

    let rust_hooks: Vec<Box<odin_palace::parser::hooks::SectionHook>> = hooks
        .unwrap_or_default()
        .into_iter()
        .map(|py_hook| {
            let hook: Box<odin_palace::parser::hooks::SectionHook> =
                Box::new(move |section_type, attrs, statement| {
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
        .collect();

    let parser = ParserBuilder::new().with_hooks(rust_hooks).build();
    let result = parser.parse(&bytes).map_err(|e| convert_error(py, e))?;
    convert_statement(py, result)
}

/// Python-модуль для парсинга банковских выписок 1CClientBankExchange.
#[pymodule]
fn odin_palace_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Классы данных
    m.add_class::<Interval>()?;
    m.add_class::<Account>()?;
    m.add_class::<Document>()?;
    m.add_class::<Statement>()?;

    // Enum типа секции для hooks
    m.add_class::<SectionType>()?;

    // Классы деталей синтаксических ошибок
    m.add_class::<UnexpectedSection>()?;
    m.add_class::<UnexpectedAttribute>()?;
    m.add_class::<UnrecognizedLine>()?;
    m.add_class::<MissingField>()?;
    m.add_class::<AccountParseError>()?;
    m.add_class::<DocumentParseError>()?;
    m.add_class::<HookError>()?;

    // Функция парсинга
    m.add_function(wrap_pyfunction!(parse, m)?)?;

    // Исключения
    m.add("ParseError", m.py().get_type::<ParseError>())?;
    m.add("NotStatementError", m.py().get_type::<NotStatementError>())?;
    m.add("EmptyInputError", m.py().get_type::<EmptyInputError>())?;
    m.add("UnfinishedError", m.py().get_type::<UnfinishedError>())?;
    m.add("SyntaxError", m.py().get_type::<SyntaxError>())?;
    Ok(())
}
