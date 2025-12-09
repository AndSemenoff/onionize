[![Clippy Check](https://github.com/andsemenoff/onionize/actions/workflows/clippy.yml/badge.svg)](https://github.com/andsemenoff/onionize/actions/workflows/clippy.yml)
![Rust Edition](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
[![License](https://img.shields.io/badge/License-MIT_or_Apache-blue?style=flat-square)](LICENSE-MIT)
![status](https://img.shields.io/badge/Status-Active-blue)
[![GitHub issues](https://img.shields.io/github/issues/andsemenoff/onionize)](https://github.com/andsemenoff/onionize/issues)

### Идея

Это аналог ngrok, но работающий исключительно через Tor Onion Services.

Проблема: Разработчикам часто нужно показать коллеге или заказчику свой локальный веб-сервер (localhost:8080), но они не хотят возиться с пробросом портов на роутере или платить за сервисы типа ngrok.
Решение: CLI-утилита, которая берет локальный порт и мгновенно создает для него временный .onion адрес.

Стек: arti, clap (для CLI), tokio (для асинхронности), hyper (для проксирования трафика).

### Планы

- добавить документацию.
- cargo i18n check — чтобы не забыть перевести новые фразы.
  
#### Аутентификация (Client Auth)

Пока что ваш сервис публичен для всех, кто знает Onion-адрес. Часто разработчики хотят закрыть доступ к локальному сервису паролем ("Client Authorization" в терминологии Tor).
Идея: Добавить флаг --auth или --secret, который включит механизм basic или stealth авторизации в tor-hsservice. Это сложнее в реализации, но сделает инструмент безопасным для шаринга чувствительных админок.

### Для разработчиков

Запустите локальный веб-сервер для теста (например, с помощью Python):
В отдельном терминале: `python3 -m http.server 3000`

### Похожие проекты

[ephemeral-hidden-service](https://github.com/aurelg/ephemeral-hidden-service) на python

## Лицензия

Этот проект лицензирован **на ваш выбор** (at your option) по одной из следующих лицензий:

* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) или http://www.apache.org/licenses/LICENSE-2.0)
* **MIT license** ([LICENSE-MIT](LICENSE-MIT) или http://opensource.org/licenses/MIT)
