"""Бенчмарк parse() через FFI-границу.

Запуск:
    uv run maturin develop --release
    uv run python benchmarks/bench_parse.py [--docs N] [--iterations N] [--threads N]

Генерирует синтетическую выписку из N документов и меряет время одного
вызова parse(). С --threads > 1 параллельные вызовы демонстрируют эффект
отпускания GIL на время парсинга.
"""

from __future__ import annotations

import argparse
import statistics
import time
from concurrent.futures import ThreadPoolExecutor

from odin_palace_py import parse

HEADER = """1CClientBankExchange
ВерсияФормата=1.03
Кодировка=Windows
Отправитель=Банк
ДатаНачала=01.06.2024
ДатаКонца=30.06.2024
РасчСчет=40702810000000000111
СекцияРасчСчет
ДатаНачала=01.06.2024
ДатаКонца=30.06.2024
РасчСчет=40702810000000000111
НачальныйОстаток=1000.00
КонечныйОстаток=1300.00
КонецРасчСчет
"""

DOCUMENT = """СекцияДокумент=Платежное поручение
Номер={n}
Дата=15.06.2024
Сумма=200.00
ПлательщикСчет=40702810000000000111
Плательщик=ООО Ромашка {n}
ПлательщикИНН=7707083893
ПлательщикБИК=044525225
ПлательщикБанк1=ПАО БАНК
ПолучательСчет=40702810000000000222
Получатель=ООО Василек {n}
ПолучательИНН=7727406020
ПолучательБИК=017003983
ПолучательБанк1=ДРУГОЙ БАНК
НазначениеПлатежа=Оплата по договору {n}
ДатаСписано=15.06.2024
КонецДокумента
"""


def build_statement(docs: int) -> bytes:
    parts = [HEADER]
    parts.extend(DOCUMENT.format(n=i) for i in range(1, docs + 1))
    parts.append("КонецФайла\n")
    return "".join(parts).encode("utf-8")


def bench_single(data: bytes, iterations: int) -> list[float]:
    for _ in range(max(iterations // 10, 5)):
        parse(data)
    times = []
    for _ in range(iterations):
        t0 = time.perf_counter()
        parse(data)
        times.append(time.perf_counter() - t0)
    return times


def bench_threads(data: bytes, iterations: int, threads: int) -> float:
    """Возвращает суммарное wall-time N параллельных вызовов parse."""
    with ThreadPoolExecutor(max_workers=threads) as pool:
        t0 = time.perf_counter()
        list(pool.map(lambda _: parse(data), range(iterations)))
        return time.perf_counter() - t0


def main() -> None:
    arg_parser = argparse.ArgumentParser(description=__doc__)
    arg_parser.add_argument("--docs", type=int, default=130)
    arg_parser.add_argument("--iterations", type=int, default=200)
    arg_parser.add_argument("--threads", type=int, default=1)
    args = arg_parser.parse_args()

    data = build_statement(args.docs)
    result = parse(data)
    print(f"statement: {len(data)} bytes, {len(result.documents)} documents")

    times = bench_single(data, args.iterations)
    print(
        f"single-thread: median={statistics.median(times) * 1e3:.3f}ms "
        f"mean={statistics.mean(times) * 1e3:.3f}ms min={min(times) * 1e3:.3f}ms"
    )

    if args.threads > 1:
        wall = bench_threads(data, args.iterations, args.threads)
        per_call = wall / args.iterations * 1e3
        print(f"{args.threads} threads: wall={wall * 1e3:.1f}ms per-call={per_call:.3f}ms")


if __name__ == "__main__":
    main()
