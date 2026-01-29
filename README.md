# odin_palace_py

Python-библиотека для парсинга банковских выписок в формате 1CClientBankExchange.

Ядро написано на Rust ([odin_palace](https://github.com/tochka-public/odin_palace)),
Python-обёртка построена с помощью [PyO3](https://pyo3.rs) и [Maturin](https://www.maturin.rs).

## Установка

```bash
pip install odin_palace_py
```

## Быстрый старт

```python
from odin_palace_py import Statement, parse

# Чтение файла выписки (CP1251 или UTF-8)
with open("statement.txt", "rb") as f:
    data: bytes = f.read()

statement: Statement = parse(data)

print(f"Кодировка: {statement.encoding}")
print(f"Заголовок: {statement.header}")
print(f"Документов: {len(statement.documents)}")
print(f"Счетов: {len(statement.accounts)}")

for doc in statement.documents:
    print(f"  #{doc.doc_number} от {doc.doc_date} на сумму {doc.amount}")
    print(f"    Назначение: {doc.purpose}")
    print(f"    Плательщик: {doc.payee_inn} ({doc.payee})")
    print(f"    Контрагент: {doc.counterparty_inn} ({doc.counterparty})")
```

## Работа со счетами

```python
for number, account in statement.accounts.items():
    print(f"Счёт: {number}")
    for interval in account.intervals:
        print(f"  Период: {interval.date_start} - {interval.date_end}")
        print(f"  Начальный остаток: {interval.start_amount}")
        print(f"  Конечный остаток: {interval.end_amount}")
```

## Обработка ошибок

Библиотека предоставляет иерархию исключений для различных ситуаций:

```python
from odin_palace_py import (
    parse,
    ParseError,
    NotStatementError,
    EmptyInputError,
    UnfinishedError,
    SyntaxError,
)

try:
    statement = parse(data)
except EmptyInputError:
    print("Файл пуст")
except NotStatementError:
    print("Файл не является выпиской 1C")
except UnfinishedError:
    print("Выписка не завершена корректно")
except SyntaxError as e:
    detail = e.detail
    print(f"Синтаксическая ошибка на строке {detail.lineno}: {e}")
except ParseError as e:
    print(f"Ошибка парсинга: {e}")
```

Объект `SyntaxError.detail` содержит типизированную информацию об ошибке:

| Тип                  | Поля                         | Описание                        |
|----------------------|------------------------------|---------------------------------|
| `UnexpectedSection`  | `lineno`, `found`, `context` | Неожиданная секция              |
| `UnexpectedAttribute`| `lineno`, `key`, `value`     | Неожиданный атрибут             |
| `UnrecognizedLine`   | `lineno`, `line`             | Нераспознанная строка           |
| `MissingField`       | `lineno`, `field`, `context` | Отсутствует обязательное поле   |
| `AccountParseError`  | `lineno`, `message`          | Ошибка парсинга счёта           |
| `DocumentParseError` | `lineno`, `message`          | Ошибка парсинга документа       |
| `HookError`          | `lineno`, `message`          | Ошибка в hook-функции           |

## Hooks

Hooks позволяют модифицировать атрибуты секций перед тем, как парсер создаст
из них объекты `Document` или `Account`.

Hook вызывается **после того, как секция полностью прочитана** (парсер встретил
маркер `КонецДокумента` или `КонецРасчСчет`), но **до того, как атрибуты
обработаны** и добавлены в итоговый `Statement`. Это значит, что hook получает
все накопленные ключ-значение пары секции целиком и может их изменить до
финальной обработки.

Порядок вызова:

1. Парсер читает строки секции, накапливая атрибуты в словарь.
2. Парсер встречает конец секции (`КонецДокумента` / `КонецРасчСчет`).
3. **Вызываются все hooks** по порядку, каждый получает собранный словарь атрибутов.
4. Парсер создаёт `Document` или `Account` из (возможно изменённых) атрибутов.

```python
from odin_palace_py import parse, SectionType

def my_hook(
    section_type: SectionType,
    attrs: dict[str, str],
    header: dict[str, str],
) -> None:
    if section_type == SectionType.Document:
        # Подменить назначение платежа
        if "НазначениеПлатежа" in attrs:
            attrs["НазначениеПлатежа"] = attrs["НазначениеПлатежа"].upper()

statement = parse(data, hooks=[my_hook])
```

Если hook выбросит исключение, парсинг прервётся с `SyntaxError`,
а `detail` будет содержать объект `HookError`.

## Поддерживаемые версии Python

- CPython 3.10+
- PyPy 3.10+

## Лицензия

MIT
