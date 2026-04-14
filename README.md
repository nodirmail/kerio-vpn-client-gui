# Kerio Control VPN Client GUI (Unofficial)

[![Tauri](https://img.shields.io/badge/built%20with-Tauri-blue.svg)](https://tauri.app/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **⚠️ Disclaimer:** This is an unofficial, open-source community GUI wrapper for the official Kerio Control VPN Client. This project is not affiliated with, endorsed, or sponsored by GFI Software or Kerio. All trademarks, logos and brand names are the property of their respective owners.

Современный графический интерфейс для Kerio Control VPN Client на Ubuntu 24.04+, вдохновленный классическим дизайном Kerio и современными стандартами UX.

![App Screenshot](https://raw.githubusercontent.com/nodirmail/kerio-vpn-client-gui/main/src-tauri/icons/128x128.png)

## Основные особенности

- **Нативный интерфейс:** Чистый дизайн на основе HTML/JS/Rust.
- **Умное управление профилями:** Создание, сохранение и быстрое переключение между серверами.
- **Бесшовная авторизация:** Забудьте о многократном вводе пароля при каждом действии (на базе оптимизированного Polkit).
- **Работа в фоновом режиме:** Приложение сворачивается в системный трей и продолжает держать соединение.
- **Синхронизация:** При запуске приложение автоматически определяет активное соединение и подтягивает нужный профиль.
- **Динамический трей:** Управляйте VPN (подключение/отключение/переключение) прямо из системного меню.

## Установка

### 1. Предварительные требования
Убедитесь, что у вас установлен `kerio-kvc`. Если нет, загрузите и установите его с официального сайта Kerio.

### 2. Клонирование и запуск (для разработчиков)
```bash
git clone https://github.com/nodirmail/kerio-vpn-client.git
cd kerio-vpn-client
npm install
npm run tauri dev
```

### 3. Сборка пакета (Release)
Чтобы создать установочный `.deb` пакет или AppImage:
```bash
npm run tauri build
```
Готовые файлы появятся в `src-tauri/target/release/bundle/`.

### 4. Оптимизация авторизации (рекомендуется)
Чтобы приложение не запрашивало пароль каждый раз при подключении:
```bash
bash installer/setup-polkit.sh
```

## Как пользоваться

1. **Запуск:** Приложение запускается в фоновом режиме (иконка в трее).
2. **Настройка:** Кликните правой кнопкой по иконке в трее -> **Settings** или левой кнопкой по иконке для открытия окна.
3. **Соединение:** Введите данные сервера в поле "Connection". При первом нажатии на **Connect** профиль будет сохранен автоматически.
4. **Удаление:** Чтобы удалить профиль, выберите его в списке и нажмите на кнопку `[X]` рядом.
5. **Выход:** Для полного закрытия приложения со сворачиванием туннеля используйте кнопку **Quit** в меню трея.

## Технологии

- **Frontend:** Vite, JavaScript, Vanilla CSS.
- **Backend:** Rust, Tauri v2.
- **System:** Systemd integration, Polkit policy management.

## Лицензия
MIT
