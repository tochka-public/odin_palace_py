import json
import pickle
from pathlib import Path
from typing import Any

import pytest
from odin_palace_py import (
    Account,
    AccountParseError,
    Document,
    DocumentParseError,
    HookError,
    Interval,
    MissingField,
    ParseError,
    SectionType,
    Statement,
    SyntaxError,
    UnexpectedAttribute,
    UnexpectedSection,
    UnrecognizedLine,
    parse,
)

SUITES_DIR = Path(__file__).parent / "suites"

ALL_SUITES = [
    "binary",
    "bom",
    "empty",
    "invalid",
    "no_documents",
    "no_end_of_account",
    "no_end_of_document",
    "no_end_of_file",
    "one_document",
    "one_document_utf",
    "statement-131911",
    "statement-137442",
    "tax_fields_1_03",
    "taxdev_4645",
    "taxdev_4645_utf",
    "trailing_space",
    "two_accounts",
    "two_documents",
    "unknown_direction",
    "version_1_03",
]


def get_input_file(suite_dir: Path) -> Path:
    """Получить файл с входными данными из папки suite."""
    input_txt = suite_dir / "input.txt"
    if input_txt.exists():
        return input_txt
    input_bin = suite_dir / "input.bin"
    if input_bin.exists():
        return input_bin
    raise FileNotFoundError(f"No input file found in {suite_dir}")


def get_snapshot_path(suite_dir: Path) -> Path:
    """Получить путь к файлу снапшота."""
    return suite_dir / "snapshot.json"


def to_dict(obj: Any) -> Any:
    """Рекурсивно конвертирует объекты в dict для JSON-сериализации."""
    if isinstance(obj, Statement):
        return {
            "accounts": {k: to_dict(v) for k, v in obj.accounts.items()},
            "documents": [to_dict(d) for d in obj.documents],
            "encoding": obj.encoding,
            "header": dict(obj.header),
            "warnings": [list(w) for w in obj.warnings],
        }
    elif isinstance(obj, Account):
        return {
            "intervals": [to_dict(i) for i in obj.intervals],
            "number": obj.number,
        }
    elif isinstance(obj, Interval):
        return {
            "date_end": obj.date_end,
            "date_start": obj.date_start,
            "end_amount": obj.end_amount,
            "start_amount": obj.start_amount,
            "total_expense": obj.total_expense,
            "total_income": obj.total_income,
        }
    elif isinstance(obj, Document):
        return {
            "amount": obj.amount,
            "compiler_status": obj.compiler_status,
            "counterparty": obj.counterparty,
            "counterparty_1": obj.counterparty_1,
            "counterparty_2": obj.counterparty_2,
            "counterparty_3": obj.counterparty_3,
            "counterparty_4": obj.counterparty_4,
            "counterparty_account": obj.counterparty_account,
            "counterparty_bank1": obj.counterparty_bank1,
            "counterparty_bank2": obj.counterparty_bank2,
            "counterparty_bic": obj.counterparty_bic,
            "counterparty_cor_account": obj.counterparty_cor_account,
            "counterparty_inn": obj.counterparty_inn,
            "counterparty_kpp": obj.counterparty_kpp,
            "counterparty_ras_account": obj.counterparty_ras_account,
            "doc_date": obj.doc_date,
            "doc_number": obj.doc_number,
            "doc_type": obj.doc_type,
            "income_date": obj.income_date,
            "kbk": obj.kbk,
            "okato": obj.okato,
            "ordering": obj.ordering,
            "outcome_date": obj.outcome_date,
            "payee": obj.payee,
            "payee_1": obj.payee_1,
            "payee_2": obj.payee_2,
            "payee_3": obj.payee_3,
            "payee_4": obj.payee_4,
            "payee_account": obj.payee_account,
            "payee_bank1": obj.payee_bank1,
            "payee_bank2": obj.payee_bank2,
            "payee_bic": obj.payee_bic,
            "payee_cor_account": obj.payee_cor_account,
            "payee_inn": obj.payee_inn,
            "payee_kpp": obj.payee_kpp,
            "payee_ras_account": obj.payee_ras_account,
            "payment_deadline": obj.payment_deadline,
            "payment_kind": obj.payment_kind,
            "payment_type": obj.payment_type,
            "purpose": obj.purpose,
            "purpose_1": obj.purpose_1,
            "purpose_2": obj.purpose_2,
            "purpose_3": obj.purpose_3,
            "purpose_4": obj.purpose_4,
            "purpose_5": obj.purpose_5,
            "purpose_6": obj.purpose_6,
            "purpose_code": obj.purpose_code,
            "tax_basis": obj.tax_basis,
            "tax_date": obj.tax_date,
            "tax_number": obj.tax_number,
            "tax_period": obj.tax_period,
            "tax_type": obj.tax_type,
            "uin": obj.uin,
        }
    elif isinstance(obj, dict):
        return {k: to_dict(v) for k, v in obj.items()}
    elif isinstance(obj, list):
        return [to_dict(item) for item in obj]
    else:
        return obj


def serialize_result(result: Statement | dict) -> str:
    """Сериализовать результат парсинга в pretty JSON."""
    data = to_dict(result)
    return json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True)


def serialize_error(exc: ParseError) -> dict:
    """Сериализовать ошибку парсинга в dict с типом и атрибутами."""
    result: dict[str, Any] = {
        "error_type": type(exc).__name__,
        "message": str(exc),
    }

    # Для SyntaxError извлекаем атрибуты из detail
    if hasattr(exc, "detail"):
        detail = exc.detail
        result["lineno"] = detail.lineno

        if isinstance(detail, UnexpectedSection):
            result["kind"] = "unexpected_section"
            result["found"] = detail.found
            result["context"] = detail.context
        elif isinstance(detail, UnexpectedAttribute):
            result["kind"] = "unexpected_attribute"
            result["key"] = detail.key
            result["value"] = detail.value
        elif isinstance(detail, UnrecognizedLine):
            result["kind"] = "unrecognized_line"
            result["line"] = detail.line
        elif isinstance(detail, MissingField):
            result["kind"] = "missing_field"
            result["field"] = detail.field
            result["context"] = detail.context
        elif isinstance(detail, AccountParseError):
            result["kind"] = "account_parse_error"
            result["detail_message"] = detail.message
        elif isinstance(detail, DocumentParseError):
            result["kind"] = "document_parse_error"
            result["detail_message"] = detail.message
        elif isinstance(detail, HookError):
            result["kind"] = "hook_error"
            result["detail_message"] = detail.message

    return result


@pytest.mark.parametrize("suite_name", ALL_SUITES)
def test_parse(suite_name: str):
    """Тест парсинга с проверкой через снапшот."""
    suite_dir = SUITES_DIR / suite_name
    input_file = get_input_file(suite_dir)
    snapshot_path = get_snapshot_path(suite_dir)

    data = input_file.read_bytes()

    try:
        result = parse(data)
        actual_json = serialize_result(result)
    except ParseError as e:
        actual_json = serialize_result(serialize_error(e))

    if not snapshot_path.exists():
        snapshot_path.write_text(actual_json, encoding="utf-8")
        pytest.fail(f"Snapshot created: {snapshot_path}. Re-run test to verify.")

    expected_json = snapshot_path.read_text(encoding="utf-8")
    assert actual_json == expected_json, (
        f"Snapshot mismatch for {suite_name}. Delete {snapshot_path} and re-run to update."
    )


ERROR_SUITES = [
    "binary",
    "empty",
    "invalid",
    "no_end_of_account",
    "no_end_of_document",
    "no_end_of_file",
    "statement-131911",
]


@pytest.mark.parametrize("suite_name", ERROR_SUITES)
def test_error_pickle(suite_name: str):
    """Тест сериализации ошибок через pickle."""
    suite_dir = SUITES_DIR / suite_name
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    with pytest.raises(ParseError) as exc_info:
        parse(data)

    original = exc_info.value
    restored = pickle.loads(pickle.dumps(original))

    assert type(restored) is type(original)
    assert str(restored) == str(original)


# UTF-8 suites that can be safely decoded to str
UTF8_SUITES = [
    "one_document_utf",
    "taxdev_4645_utf",
]


@pytest.mark.parametrize("suite_name", UTF8_SUITES)
def test_parse_str_input(suite_name: str):
    """Тест парсинга строки (str) вместо байтов."""
    suite_dir = SUITES_DIR / suite_name
    input_file = get_input_file(suite_dir)
    snapshot_path = get_snapshot_path(suite_dir)

    # Читаем как строку UTF-8
    data_str = input_file.read_text(encoding="utf-8")
    result = parse(data_str)

    actual_json = serialize_result(result)
    expected_json = snapshot_path.read_text(encoding="utf-8")

    assert actual_json == expected_json


def test_parse_str_vs_bytes_same_result():
    """Тест что парсинг str и bytes даёт одинаковый результат."""
    suite_dir = SUITES_DIR / "one_document_utf"
    input_file = get_input_file(suite_dir)

    data_bytes = input_file.read_bytes()
    data_str = input_file.read_text(encoding="utf-8")

    result_bytes = parse(data_bytes)
    result_str = parse(data_str)

    assert serialize_result(result_bytes) == serialize_result(result_str)


def test_parse_invalid_type():
    """Тест что передача неверного типа вызывает TypeError."""
    with pytest.raises(TypeError, match="statement must be bytes or str"):
        parse(12345)

    with pytest.raises(TypeError, match="statement must be bytes or str"):
        parse(["list", "of", "strings"])

    with pytest.raises(TypeError, match="statement must be bytes or str"):
        parse(None)


def test_syntax_error_detail_types():
    """Тест что SyntaxError.detail имеет правильный тип."""
    # Используем no_end_of_account, который вызывает SyntaxError с UnexpectedSection
    suite_dir = SUITES_DIR / "no_end_of_account"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    with pytest.raises(SyntaxError) as exc_info:
        parse(data)

    err = exc_info.value
    assert hasattr(err, "detail")
    detail = err.detail

    # Проверяем что detail - один из ожидаемых типов
    assert isinstance(
        detail,
        (
            UnexpectedSection,
            UnexpectedAttribute,
            UnrecognizedLine,
            MissingField,
            AccountParseError,
            DocumentParseError,
            HookError,
        ),
    )

    # Проверяем что lineno доступен
    assert isinstance(detail.lineno, int)
    assert detail.lineno > 0


# Тесты на конкретные типы SyntaxError
SYNTAX_ERROR_SUITES = [
    ("no_end_of_account", UnexpectedSection, {"found", "context"}),
    ("no_end_of_document", UnexpectedSection, {"found", "context"}),
    ("statement-131911", UnrecognizedLine, {"line"}),
]


@pytest.mark.parametrize(("suite_name", "expected_type", "expected_attrs"), SYNTAX_ERROR_SUITES)
def test_syntax_error_specific_types(
    suite_name: str,
    expected_type: type,
    expected_attrs: set[str],
):
    """Тест что конкретные suites вызывают правильные типы SyntaxError."""
    suite_dir = SUITES_DIR / suite_name
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    with pytest.raises(SyntaxError) as exc_info:
        parse(data)

    err = exc_info.value
    assert hasattr(err, "detail")
    detail = err.detail

    # Проверяем тип detail
    assert isinstance(detail, expected_type), (
        f"Ожидался {expected_type.__name__}, получен {type(detail).__name__}"
    )

    # Проверяем наличие всех ожидаемых атрибутов
    for attr in expected_attrs:
        assert hasattr(detail, attr), f"Отсутствует атрибут {attr}"
        assert getattr(detail, attr) is not None, f"Атрибут {attr} равен None"

    # Проверяем lineno
    assert isinstance(detail.lineno, int)
    assert detail.lineno > 0


def test_unexpected_section_attributes():
    """Тест атрибутов UnexpectedSection."""
    suite_dir = SUITES_DIR / "no_end_of_account"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    with pytest.raises(SyntaxError) as exc_info:
        parse(data)

    detail = exc_info.value.detail
    assert isinstance(detail, UnexpectedSection)

    # Проверяем типы атрибутов
    assert isinstance(detail.lineno, int)
    assert isinstance(detail.found, str)
    assert isinstance(detail.context, str)

    # Проверяем что found содержит название секции
    assert "СекцияДокумент" in detail.found

    # Проверяем что context - допустимое значение
    assert detail.context in {
        "header",
        "document",
        "account",
        "finished",
        "init",
        "read_next_section",
    }


def test_unrecognized_line_attributes():
    """Тест атрибутов UnrecognizedLine."""
    suite_dir = SUITES_DIR / "statement-131911"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    with pytest.raises(SyntaxError) as exc_info:
        parse(data)

    detail = exc_info.value.detail
    assert isinstance(detail, UnrecognizedLine)

    # Проверяем типы атрибутов
    assert isinstance(detail.lineno, int)
    assert isinstance(detail.line, str)

    # Проверяем номер строки
    assert detail.lineno == 11


def test_syntax_error_detail_pickle():
    """Тест сериализации detail-объектов SyntaxError через pickle."""
    suite_dir = SUITES_DIR / "no_end_of_account"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    with pytest.raises(SyntaxError) as exc_info:
        parse(data)

    original_detail = exc_info.value.detail
    restored_detail = pickle.loads(pickle.dumps(original_detail))

    assert type(restored_detail) is type(original_detail)
    assert restored_detail.lineno == original_detail.lineno

    if isinstance(original_detail, UnexpectedSection):
        assert restored_detail.found == original_detail.found
        assert restored_detail.context == original_detail.context


def test_syntax_error_detail_repr():
    """Тест что detail-объекты имеют читаемый repr."""
    suite_dir = SUITES_DIR / "no_end_of_account"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    with pytest.raises(SyntaxError) as exc_info:
        parse(data)

    detail = exc_info.value.detail
    repr_str = repr(detail)

    # Проверяем что repr содержит имя класса
    assert "UnexpectedSection" in repr_str

    # Проверяем что repr содержит lineno
    assert "lineno=" in repr_str


# ============================================================================
# Тесты для hooks
# ============================================================================


def test_parse_with_empty_hooks():
    """Тест что parse с пустым списком hooks работает."""
    suite_dir = SUITES_DIR / "one_document"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    result = parse(data, hooks=[])

    assert isinstance(result, Statement)
    assert len(result.documents) == 1


def test_parse_with_hooks_same_result_as_without():
    """Тест что parse с пустыми hooks даёт тот же результат, что и без них."""
    suite_dir = SUITES_DIR / "one_document"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    result_without_hooks = parse(data)
    result_with_empty_hooks = parse(data, hooks=[])

    assert serialize_result(result_without_hooks) == serialize_result(result_with_empty_hooks)


def test_section_type_enum():
    """Тест enum SectionType."""
    assert SectionType.Document != SectionType.Account
    assert repr(SectionType.Document) == "SectionType.Document"
    assert repr(SectionType.Account) == "SectionType.Account"


def test_hook_receives_correct_arguments():
    """Тест что hook получает правильные аргументы."""
    suite_dir = SUITES_DIR / "one_document"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    received_calls: list[tuple[SectionType, dict, dict]] = []

    def recording_hook(section_type, attrs, header):
        # Сохраняем копию, т.к. attrs может измениться
        received_calls.append((section_type, dict(attrs), dict(header)))

    parse(data, hooks=[recording_hook])

    # Должен быть вызван минимум 2 раза - для Account и Document
    assert len(received_calls) >= 2

    # Проверяем типы аргументов
    for section_type, attrs, header in received_calls:
        assert isinstance(section_type, SectionType)
        assert isinstance(attrs, dict)
        assert isinstance(header, dict)

    # Проверяем что header содержит данные
    for _, _, header in received_calls:
        assert "ВерсияФормата" in header


def test_hook_modifies_attrs():
    """Тест что hook может модифицировать атрибуты."""
    suite_dir = SUITES_DIR / "one_document"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    def modify_hook(section_type, attrs, header):
        if section_type == SectionType.Document and "НазначениеПлатежа" in attrs:
            attrs["НазначениеПлатежа"] = "MODIFIED_PURPOSE"

    result = parse(data, hooks=[modify_hook])

    # Проверяем что purpose был изменён
    assert len(result.documents) == 1
    assert result.documents[0].purpose == "MODIFIED_PURPOSE"


def test_hook_error_raises_syntax_error():
    """Тест что исключение в hook вызывает SyntaxError с HookError detail."""
    suite_dir = SUITES_DIR / "one_document"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    def failing_hook(section_type, attrs, header):
        raise ValueError("Test error from hook")

    with pytest.raises(SyntaxError) as exc_info:
        parse(data, hooks=[failing_hook])

    err = exc_info.value
    assert hasattr(err, "detail")
    assert isinstance(err.detail, HookError)
    assert "Test error from hook" in err.detail.message


def test_multiple_hooks_called_in_order():
    """Тест что несколько hooks вызываются по порядку."""
    suite_dir = SUITES_DIR / "one_document"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    call_order: list[int] = []

    def hook1(section_type, attrs, header):
        call_order.append(1)

    def hook2(section_type, attrs, header):
        call_order.append(2)

    parse(data, hooks=[hook1, hook2])

    # Проверяем что hooks вызываются в правильном порядке
    # Каждый hook вызывается несколько раз (для каждой секции)
    # Порядок должен быть 1, 2, 1, 2, ...
    for i in range(0, len(call_order) - 1, 2):
        if i + 1 < len(call_order):
            assert call_order[i] == 1
            assert call_order[i + 1] == 2


def test_hook_with_account_section():
    """Тест что hook вызывается для секции Account."""
    suite_dir = SUITES_DIR / "one_document"
    input_file = get_input_file(suite_dir)
    data = input_file.read_bytes()

    account_calls = []

    def account_hook(section_type, attrs, header):
        if section_type == SectionType.Account:
            account_calls.append(dict(attrs))

    parse(data, hooks=[account_hook])

    # Должен быть вызван для секции Account
    assert len(account_calls) >= 1
    # Проверяем что attrs содержит данные счёта
    assert any("РасsчСчет" in call or "НачальныйОстаток" in call for call in account_calls)
