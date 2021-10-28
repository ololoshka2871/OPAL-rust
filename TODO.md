* [v] Минимальная функциональность
    * [v] Собирается
    * [v] Запускается отладка
* [v] Способ встраивания сторонних C-либ
    * [v] Heatshrink
        * [v] Сборка с остальной системой
        * [v] Создать внешнюю либу, которая использует C под капотом
        * [v] Собрать с либой, протестировать
* [v] Начальная конфигурация клоков MCU
    * [v] Найти правильную реализацию HAL
        tm32l4xx-hal, features = ["stm32l4x3", "rt"]
    * [v] Разобраться как взаимодействовать с API
        use stm32l4xx_hal::stm32;

* [v] "Перешагивать" код, где нет отладочных символов
    gdb: skip -rfu ^core::

* [v] USB
    * [v] Настрить минимальную сборку
    * [v] Разобраться с получением stm32::Peripherals::take()
        unsafe {stm32::Peripherals::steal()}
    * [v] Клокинг USB от PLLR (/2)
    * [v] Виртуальный COM-порт работает

* [x] Частота проца в конфиге FreeRTOS
    \- Переехали на RTIC \-

* [v] Разобраться с cargo embed
    Смотри полную справку в обсидиане
    [v] Config: `.embed.toml`
    [v] Прошивка
    [v] Логирование: `rtt-target`
    [v] Отладка

* [v] USB Mass Storage
    https://github.com/cs2dsb/stm32-usb.rs/tree/master/firmware/usbd_mass_storage
    * [v] Выбрать VID/PID => 0x0483/0x5720
    * [v] Пример запускается, но пока с заглушкой вместо чтения-записи
    * [v] Удалось создать Mass Storage + ACM, но работает нестабильно
        * [v] Под FreeRTOS бе части работают, но в винде только COM-порт ито как-то странно
        * [v] Заставить работать в винде
    * [v] Пример c Mass Storage жрет почти все мето в контроллере, почему?
        добавил оптимизацию 1 уровня - размер уменьшился в 2 раза `[profile.dev]/opt-level = 1`
    * [x] Проверить распознает ли винда диск Mass Storage
        Не работает, вероятно проблема в "string descriptor 0 read error: -71"
    * [x] RTIC Запуск задачи вне контекста прерывания
        Все задачи RTIC - это прерывания

* [v] defmt : https://ferrous-systems.com/blog/defmt/
    Множественные изменение в настройках сборки

* [v] Определить, оставляем RTIC или FreeRTOS 
    Выбрана FreeRTOS

* [v] Привезти проект в порядок
    * [v] Убрать варнинги от названий FreeRTOS
    * [v] Вынести сервисный код в модули
    * [v] Привезти в порядок хуки `defmt::panic!()`
    * [v] Настроить зависимость от heatshrink-rust как подкаталог
    * [v] Проверить конфиг FreeRTOS
        * [v] configCPU_CLOCK_HZ <= from config
        * [v] configUSE_MALLOC_FAILED_HOOK = 1
        * [v] configUSE_TICKLESS_IDLE = 1 -> configUSE_LOBARO_TICKLESS_IDLE = 1
        * [x] configPRIO_BITS - ?
    * [v] Прерывания USB будит поток-обработчик
            use FreeRTOS wait-notify

* [v] Определение режима работы в зависимости от того, включен ли USB
    * [v] Определение, включен-ли USB
        * [v] Мониторинг через pwr.sr2.PVMO1, делитель R2-R10 больше не нужен
        * [v] Если не включен - не запускать драйвер
    * [_] Различная настройка клоков для разных режимов
        Медленный режим - 12MHz (HSE => CPU) 
        Быстрый: (HSE = 12 / 3 * 40 / 2 => 80 => CPU), (HSE= 12 / 3 * 24 / 2 => PLLSAI1Q => USB)
        * [v] Частоты установлены
        * [x] Вместо rcc.cfgr.freeze использовать свою реализацию, посколькоу оно не даёт 
            возможности напрямую подать с HSE в CPU
    * [v] FreeRTOS будет настроен на медленный режим (12MHz), в быстром тики будут идти чаще.
    * [v] Трейт для режимов работы
        Поскольку полиморфизм очень странно себя ведет, то виртуальные функции юзать лучше не
        надо, генерики и компайл-тайм полиморфизм!

* [v] Поток мониторинга
    * [v] Краш при попытке malloc() - проверить объем памяти проца
        * [v] Defmt не влияет
        * [v] ломается цепочка блоков аллокатора
        * [v] Прерывание USB не при чем
        * [v] Запись идет в функции vTaskGetInfo() pxTaskStatus->uxBasePriority = 1; pxTaskStatus->ulRunTimeCounter = 0; pxTaskStatus указывает в неправильное место!
        * [v] Проверить размеры структур
            Структура FreeRtosTaskStatusFfi Rust не соответствует структуре xTASK_STATUS С
            [x] Найти и переключиться на правильную версию FreeRTOS
            [v] Форкнута библиотека freertos-rust  пофикшено там, создан issue https://github.com/lobaro/FreeRTOS-rust/issues/15 
    * [v] Отладить вычисление задержки на сон потока мониторинга
        * [v] Спецфункция-врапер
    * [v] Лог и неспящий FreeRTOS только в режиме debug
        * [v] Раздельные конфигурации
        * [v] Раздельные FreeRTOSConfig.h
        * [v] vApplicationIdleHook только для отладочного режима
    * [v] Тест Релиз-сборки
    * [x] Мониторинг в отдельный канал RTT, сериализовать FreeRtosSchedulerState
        Есть спец-либа defmt\-rtt\-target, но она для defmt 0.1, одифицировать слишком сложно
    * [v] Настроить линкер так, чтобы _SEGGER_RTT не прыгала
        Сложно. Будем вычислять на лету
        * [v] Скрипт
    * [v] cargo-make
        * [v] Задачи - Makefile.toml
        * [v] add to Readme
    * [v] Статус FreeRtos - вывод за 1 команду, чтобы не было разделений.
        * [x] Радобраться как печатается структура с вложенными полями.
        * [v] Печать сборкой длинной строки в памяти

* [v] Virtual fat
    * [v] Сборка
        * [v] strlen() missing
            Добавлена реализация на Rust
    * [v] Чтение
    * [x] Запись
        Недопилено в библотеке
    * [v] Работает В Windows
        * [v] SCSI: Rest ready - no answer
        * [v] Форк библиотеки usbd_scsi
            * [v] Встроить субмодулем
        * [v] Команда 0x23 
            * [v] Исправлен парсинг
            * [v] Создан ответ
        * [v] sdc: p1 size 82 extends beyond EOD, enabling native capacity
            https://wi-cat.ru/forums/topic/133/#postid-1602
            * [v] Поле disk_sectors структуры stm32_usb_self_writer::threads::storage.ctx не является размером диска, нужно добавлять значение stm32_usb_self_writer::threads::storage.ctx.priv_.boot_lba
        * [v] Решено: Смотри ZLP
    * [v] BOS Descriptor: Response data (unknown descriptor type 15): 050f0c000107100200000000
        Согласно Universal Serial Bus 3.0 Specification, пункт 9.6.2.1:
            05 - длина заголовка дескриптора в байтах
            0f - BOS Descriptor
            0c00 - обзаяя длина всего десккриптора
            01 - 1 дескриптор в пакете:
                07 - размер этого дескриптора в байтах
                10 - тип дескриптора - "DEVICE COMPABILITY"
                02 - константа "Universal Serial Bus 3.0 Specification"
                00 00 00 00 - бит 1 == 0 - Universal Serial Bus 3.0 Specification не поддерживается
    * [v] Device qualifier: Empty response
        * [v] отсутсвует в либе, делается отлуп (reject)
            Дискриптор должен сообщить хосту как он хочет работать на большей скорости, а поскользу устройство не может - то и дескриптор не нужен:
            https://www.keil.com/pack/doc/mw/USB/html/_u_s_b__device__qualifier__descriptor.html
    * [v] Endpoint descriptor: Interval == 0
        Значение только для изохорного и Interrupt ржимов: с каким периодом опрешивать конечную точку точку в единицах базового интервала USB. Игнорируется для остальных типов конечных точек
    * [v] Почему кард-ридер получает запросы через USB ?.?.2, а ответ посылает через ?.?.1, тогда 
            как наше устройство все через ?.?.1 ?
            Ответ: Кард-ридер юзает точку 0x81 (IN) и 0x02 (OUT), а наш девайс 0x81 (IN) и 0x01 (OUT)
            [v] А так точно можно?
                Да можно, судя по таблице выделения памяти конечным точкам для каждой есть независимый
                Указатель USB_ADDRn_\[RT\]X и размер буфера USB_COUNTn_\[RT\]X
    * [v] USB_COUNTn_RX - таблица
            Это делается в stm32_usbd::endpoint::set_out_buf()/set_in_buf()
    * [v] Почему не работает с 8-битным доступом `EP_MEMORY_ACCESS_2X16 = false`? должно быть
            медленнее, но не ломаться совсем, нет? Не получается запросить даже дискриптор устройства.
            Неверно заполняется таблица указателей на буфера конечных точек: значения записываются как
            будто их адреса в 2 раза больше. На лицо неверная адресация. оставляем `EP_MEMORY_ACCESS_2X16 = true`
    * [v] Для устройств Full Speed можно иметь размер конечных точек типа Bulk только 8, 16, 32 или 64
            байта, поэтому 256 и отваливается в ошибку -EOVERFLOW
    * [v] Проверить, нет ли неопознанных команд
        * [v] парсинг - Нет
        * [v] Обработка - Нет
    * [v] Обнаружено расхождение: в Windows не присылается пакет показывающий успех выполнения чтения
            Из-за этого драйвер не может определтить прошло-ли чтение
        * [v] Определение проблемы - Вместо пакета подтвеждения присылается пакет ZLP 
                только при повторном запросе присылается верный пакет, Linux игнорирует проблему.
        * [v] Закостылить.
            В библиотеку `lib/stm32-usb.rs/firmware/usbd_bulk_only_transport` обавлена фича, запрещающая
            отправку ZLP, фича прокидывается через библиотеку `lib/stm32-usb.rs/firmware/usbd_scsi`.
    * [v] Привезти исходный код в порядок после тестов
        * [v] Убрать лишние изменения из библиотеки stm32-usb.rs
        * [v] Убрать лишние изменения из библиотеки emfat
        * [v] Убрать отладочный код
    
* [v] Composite device: Mass Storage + VCP
    * [v] Разобраться, что происходит, если не создавать композит вообще
        LINUX: все работает
        Windows: Mass Storage работает, VCP - нет
        Если посмотреть на дискрипторы, то:
        ```
        # несущественные поля опущены
        CONFIGURATION DESCRIPTOR # шапка
            wTotalLength: 90 # общая длина в байтах
            bNumInterfaces: 3 # 3 интрфейса
        # это работает
        INTERFACE DESCRIPTOR (0.0): class Mass Storage
            bInterfaceNumber: 0 # номер интерфейса
        # использует 2 конечные точки на вход и выход
        ENDPOINT DESCRIPTOR
            bEndpointAddress: 0x01  OUT  Endpoint:1
        ENDPOINT DESCRIPTOR
            bEndpointAddress: 0x81  IN  Endpoint:1
        # тут не хватает Interface Association Descriptor
        # -- Это не работает --
        # Это интерфейс управления CDC Control
        INTERFACE DESCRIPTOR (1.0): class Communications and CDC Control
            bInterfaceNumber: 1 # номер интерфейса
        COMMUNICATIONS DESCRIPTOR #???
        COMMUNICATIONS DESCRIPTOR #???
        COMMUNICATIONS DESCRIPTOR #???
        COMMUNICATIONS DESCRIPTOR #???
        # 1 конечная точка управления CDC, одна от девайса к хосту, хост 
        # периодически читает её чтобы узнать состояние устройства, 
        # и далее что-то передает или читает из CDC-Data
        ENDPOINT DESCRIPTOR
            bEndpointAddress: 0x82  IN  Endpoint:2
        # Это интерфейс данных CDC-Data
        INTERFACE DESCRIPTOR (2.0): class CDC-Data
            bInterfaceNumber: 2 # номер интерфейса
        # И его конечные точки
        ENDPOINT DESCRIPTOR
            bEndpointAddress: 0x83  IN  Endpoint:3
        ENDPOINT DESCRIPTOR
            bEndpointAddress: 0x03  OUT  Endpoint:3
        ```
        Винда ругается, мол "Указано несуществующее устройство", по видимому не хватает,
        как раз Interface Association Descriptor перед class Communications and CDC Control
    [v] Изучить процесс отправки дескрипторов
        Оказывается, все в либе уже есть, нужно было лишь включить этоу возможность
        `UsbDeviceBuilder::composite_with_iads()` и в `UsbDevice<T>::poll(&mut[...])` 
        сувать список ранее созданных интерфейсов в порядке регистрации.
        После этого появляется заполненый Interface Association Descriptor в нужном месте.

[_] Modbus или protobuf?
    [v] Попытка поспользоваться nanopb
        [_] Генератор
            [v] Код генератора на питоне, необходимо тащить питоновские скрипты с собой
                Упакуем их в бинарь https://stackoverflow.com/a/47889785
            [_] Плагин генератора надо еще собирать - make
