//! Python-классы данных: `Statement`, `Document`, `Account`, `Interval`, `SectionType`.

use odin_palace::parser::hooks::SectionType as RustSectionType;
use pyo3::prelude::*;

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
    // Поля спецификации 1.01-1.03, добавленные в odin_palace позже
    // основного набора (см. документацию ядра).
    pub counterparty_2: Option<String>,
    pub counterparty_3: Option<String>,
    pub counterparty_4: Option<String>,
    pub counterparty_bank2: Option<String>,
    pub payee_1: Option<String>,
    pub payee_2: Option<String>,
    pub payee_3: Option<String>,
    pub payee_4: Option<String>,
    pub payee_bank2: Option<String>,
    pub payment_kind: Option<String>,
    pub payment_deadline: Option<String>,
    pub uin: Option<String>,
    pub purpose_code: Option<String>,
    pub purpose_1: Option<String>,
    pub purpose_2: Option<String>,
    pub purpose_3: Option<String>,
    pub purpose_4: Option<String>,
    pub purpose_5: Option<String>,
    pub purpose_6: Option<String>,
    pub compiler_status: Option<String>,
    pub kbk: Option<String>,
    pub okato: Option<String>,
    pub tax_basis: Option<String>,
    pub tax_period: Option<String>,
    pub tax_number: Option<String>,
    pub tax_date: Option<String>,
    pub tax_type: Option<String>,
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
    pub encoding: &'static str,
    pub header: Py<PyAny>,
    pub accounts: Py<PyAny>,
    pub documents: Py<PyAny>,
    pub documents_len: usize,
    pub warnings: Py<PyAny>,
    pub warnings_len: usize,
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
