//! Конвертация Rust-структур ядра в Python-объекты.

use odin_palace::parser::{Document as RustDocument, Encoding, Statement as RustStatement};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::types::{Account, Document, Interval, Statement};

pub fn encoding_to_str(encoding: Encoding) -> &'static str {
    match encoding {
        Encoding::Cp1251 => "cp1251",
        Encoding::Utf8 => "utf-8",
    }
}

pub fn convert_statement(py: Python<'_>, statement: RustStatement) -> PyResult<Statement> {
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
    let documents_list = PyList::new(py, documents)?;

    // Конвертируем warnings в PyList
    let warnings_len = statement.warnings.len();
    let warnings_list = PyList::new(py, statement.warnings)?;

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
        counterparty_2: doc.counterparty_2,
        counterparty_3: doc.counterparty_3,
        counterparty_4: doc.counterparty_4,
        counterparty_bank2: doc.counterparty_bank2,
        payee_1: doc.payee_1,
        payee_2: doc.payee_2,
        payee_3: doc.payee_3,
        payee_4: doc.payee_4,
        payee_bank2: doc.payee_bank2,
        payment_kind: doc.payment_kind,
        payment_deadline: doc.payment_deadline,
        uin: doc.uin,
        purpose_code: doc.purpose_code,
        purpose_1: doc.purpose_1,
        purpose_2: doc.purpose_2,
        purpose_3: doc.purpose_3,
        purpose_4: doc.purpose_4,
        purpose_5: doc.purpose_5,
        purpose_6: doc.purpose_6,
        compiler_status: doc.compiler_status,
        kbk: doc.kbk,
        okato: doc.okato,
        tax_basis: doc.tax_basis,
        tax_period: doc.tax_period,
        tax_number: doc.tax_number,
        tax_date: doc.tax_date,
        tax_type: doc.tax_type,
    }
}
