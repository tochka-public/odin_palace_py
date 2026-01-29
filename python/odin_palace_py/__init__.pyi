from collections.abc import Callable
from enum import Enum
from typing import Literal, TypeAlias

class Interval:
    """Интервал счёта с информацией об остатках за период."""

    date_start: str
    date_end: str | None
    start_amount: str
    end_amount: str | None
    total_income: str | None
    total_expense: str | None

class Account:
    """Банковский счёт с интервалами."""

    number: str
    intervals: list[Interval]

class Document:
    """Распарсенный банковский документ/транзакция."""

    doc_number: str
    doc_type: str
    doc_date: str
    amount: str
    purpose: str
    payment_type: str | None
    ordering: str | None
    counterparty_inn: str
    counterparty_kpp: str | None
    counterparty_bic: str
    counterparty_bank1: str
    counterparty_account: str
    counterparty: str | None
    counterparty_1: str | None
    counterparty_ras_account: str | None
    counterparty_cor_account: str | None
    outcome_date: str | None
    payee_inn: str
    payee: str | None
    payee_account: str
    payee_kpp: str | None
    payee_bic: str
    payee_bank1: str
    payee_ras_account: str | None
    payee_cor_account: str | None
    income_date: str | None

class Statement:
    """Распарсенная выписка 1CClientBankExchange."""

    encoding: Literal["cp1251", "utf-8"]
    header: dict[str, str]
    accounts: dict[str, Account]
    documents: list[Document]
    warnings: list[tuple[int, str]]

# ============================================================================
# Классы деталей SyntaxError
# ============================================================================

class UnexpectedSection:
    """Детали SyntaxError при обнаружении неожиданной секции."""

    lineno: int
    found: str
    context: Literal["header", "document", "account", "finished", "init", "read_next_section"]

class UnexpectedAttribute:
    """Детали SyntaxError при обнаружении неожиданного атрибута."""

    lineno: int
    key: str
    value: str

class UnrecognizedLine:
    """Детали SyntaxError когда строка не распознана."""

    lineno: int
    line: str

class MissingField:
    """Детали SyntaxError когда отсутствует обязательное поле."""

    lineno: int
    field: str
    context: Literal["header", "document", "account", "finished", "init", "read_next_section"]

class AccountParseError:
    """Детали SyntaxError при ошибке парсинга счёта."""

    lineno: int
    message: str

class DocumentParseError:
    """Детали SyntaxError при ошибке парсинга документа."""

    lineno: int
    message: str

class HookError:
    """Детали SyntaxError при ошибке в hook."""

    lineno: int
    message: str

SyntaxErrorDetail: TypeAlias = (
    UnexpectedSection
    | UnexpectedAttribute
    | UnrecognizedLine
    | MissingField
    | AccountParseError
    | DocumentParseError
    | HookError
)

# ============================================================================
# Hooks
# ============================================================================

class SectionType(Enum):
    """Тип секции выписки для использования в hooks."""

    Document = ...
    Account = ...

# ============================================================================
# Исключения
# ============================================================================

class ParseError(Exception):
    """Базовое исключение для всех ошибок парсинга."""

    ...

class NotStatementError(ParseError):
    """Входные данные не являются выпиской 1C."""

    ...

class EmptyInputError(ParseError):
    """Входные данные пусты."""

    ...

class UnfinishedError(ParseError):
    """Выписка не завершена корректно."""

    ...

class SyntaxError(ParseError):
    """Синтаксическая ошибка в выписке.

    Атрибут `detail` содержит типизированный объект с информацией об ошибке.
    """

    detail: SyntaxErrorDetail

SectionHook: TypeAlias = Callable[[SectionType, dict[str, str], dict[str, str]], None]

def parse(
    statement: bytes | str,
    hooks: list[SectionHook] | None = None,
) -> Statement:
    """Парсит выписку 1CClientBankExchange из байтов или строки.

    Args:
        statement: Сырые байты или строка файла выписки
            (поддерживает кодировки CP1251 и UTF-8 для байтов)
        hooks: Опциональный список callable для модификации секций во время парсинга.
            Сигнатура: (section_type: SectionType, attrs: dict, header: dict) -> None
            - section_type: тип секции (SectionType.Document или SectionType.Account)
            - attrs: словарь атрибутов секции (можно модифицировать)
            - header: словарь заголовка выписки (только для чтения)

    Returns:
        Объект Statement с распарсенными данными.

    Raises:
        NotStatementError: Если входные данные не являются выпиской 1C
        EmptyInputError: Если входные данные пусты
        UnfinishedError: Если выписка не завершена корректно
        SyntaxError: Если в выписке синтаксическая ошибка (включая ошибки hooks)
        ParseError: Для прочих ошибок парсинга
        TypeError: Если входные данные не bytes и не str
    """
    ...
