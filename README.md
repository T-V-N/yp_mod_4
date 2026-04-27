# image_processor

CLI-приложение для обработки PNG-изображений через динамически подключаемые плагины (FFI).

## Структура проекта

```
yp_mod_4/
├── Cargo.toml              # Workspace
├── image_processor/        # Основное CLI-приложение
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── error.rs
│       └── plugin_loader.rs
├── blur_plugin/            # Плагин размытия
│   └── src/lib.rs
└── mirror_plugin/          # Плагин зеркального отражения
    └── src/lib.rs
```


## Сборка

```bash
cargo build --workspace
```

После сборки плагины окажутся в `target/debug/`:

| ОС      | Файлы плагинов                                 |
|---------|------------------------------------------------|
| Linux   | `libblur_plugin.so`, `libmirror_plugin.so`     |
| macOS   | `libblur_plugin.dylib`, `libmirror_plugin.dylib` |
| Windows | `blur_plugin.dll`, `mirror_plugin.dll`         |

## Запуск

```bash
cargo run -p image_processor -- \
  --input <путь к PNG> \
  --output <путь для результата> \
  --plugin <имя плагина> \
  --params <путь к файлу параметров> \
  [--plugin-path <директория плагинов>]  # по умолчанию: target/debug
```

> **Linux/macOS:** передавать имя плагина с префиксом `lib` - `libblur_plugin`.  
> **Windows:** без префикса - `blur_plugin`.

### Пример: размытие

Создаем конфиг `params.json`:
```json
{"radius": 3, "iterations": 2}
```

```bash
# Linux
cargo run -p image_processor -- \
  --input photo.png \
  --output blurred.png \
  --plugin libblur_plugin \
  --params params.json

# Windows
cargo run -p image_processor -- \
  --input photo.png \
  --output blurred.png \
  --plugin blur_plugin \
  --params params.json
```

### Пример: мироринг

Конфиг `params.json`:
```json
{"horizontal": true, "vertical": false}
```

```bash
# Linux
cargo run -p image_processor -- \
  --input photo.png \
  --output mirrored.png \
  --plugin libmirror_plugin \
  --params params.json
```

### Справка по аргументам

```bash
cargo run -p image_processor -- --help
```

## Аргументы командной строки

| Аргумент        | Описание                                              | Обязателен |
|-----------------|-------------------------------------------------------|------------|
| `--input`       | Путь к исходному PNG-изображению                      | Да         |
| `--output`      | Путь для сохранения результата                        | Да         |
| `--plugin`      | Имя файла плагина без расширения                      | Да         |
| `--params`      | Путь к JSON-файлу с параметрами обработки             | Да         |
| `--plugin-path` | Директория с плагинами (по умолчанию: `target/debug`) | Нет        |

## Формат параметров плагинов

Параметры передаются в виде JSON-файла.

### blur_plugin — размытие

```json
{
  "radius": 2,
  "iterations": 1
}
```

| Поле         | Тип  | Описание                    |
|--------------|------|-----------------------------|
| `radius`     | u32  | Радиус размытия в пикселях  |
| `iterations` | u32  | Количество проходов         |

Алгоритм: взвешенное среднее соседних пикселей, вес = `1 / расстояние` (центральный пиксель имеет вес 1).

### mirror_plugin — зеркальное отражение

```json
{
  "horizontal": true,
  "vertical": false
}
```

| Поле         | Тип  | Описание                          |
|--------------|------|-----------------------------------|
| `horizontal` | bool | Отразить по горизонтали (лево↔право) |
| `vertical`   | bool | Отразить по вертикали (верх↔низ)    |

## API плагина

Любой плагин должен экспортировать функцию с сигнатурой на языке C:

```c
void process_image(
    uint32_t height,
    uint32_t width,
    uint8_t* rgba_data,
    const char* params
);
```

- `height`, `width` — размеры изображения в пикселях
- `rgba_data` — указатель на плоский массив байт в формате RGBA (4 байта на пиксель), `height * width * 4` байт
- `params` — null-terminated строка с параметрами (JSON или любой текстовый формат)

Плагин модифицирует `rgba_data` на месте.

## Тесты

```bash
cargo test --workspace
```

Тесты плагинов используют PNG-файлы из `<crate>/tests/`: `input.png` и `reference.png`.
