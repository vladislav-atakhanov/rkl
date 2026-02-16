# RKL

RKL - это CLI-инструмент для описания раскладок клавиатуры и компиляции их в
- [x] [Kanata](https://github.com/jtroo/kanata)
- [x] [Vial](https://get.vial.today/)
- [ ] [ZMK](https://zmk.dev/)
- [ ] [RMK](https://rmk.rs/)

## Установка

```bash
cargo build --release
```

## Использование

```bash
raskl <file> [--kanata <output>] [--vial]
```

| Флаг              | Описание                                                          |
|-------------------|-------------------------------------------------------------------|
| `--kanata <file>` | Сгенерировать конфиг Kanata (используйте `-` для вывода в stdout) |
| `--vial`          | Прошить раскладку в подключенную Vial-клавиатуру                  |

```bash
raskl layout.rkl --kanata config.kbd   # записать в файл
raskl layout.rkl --kanata -            # вывести в stdout
raskl layout.rkl --vial                # прошить в устройство
```

## Синтаксис

Язык основан на синтаксисе Kanata. Все директивы записываются как `(имя ...)`.

### Клавиши

| Синтаксис                       | Значение                                            |
|---------------------------------|-----------------------------------------------------|
| `q`, `esc`, `spc`, `tab`, `bks` | Обычные клавиши                                     |
| `ctl`, `sft`, `alt`, `meta`     | Модификаторы (сокращения: `C`, `S`, `A`, `M`)       |
| `X`                             | Нет действия                                        |
| `_`                             | Прозрачная (действие берется из родительского слоя) |
| `@имя`                          | Ссылка на алиас                                     |
| `.символ`                       | Юникод-символ (`.!`, `.@`, `.#`)                    |
| `lb`, `rb`                      | Скобки `(` и `)`                                    |
| `A-i`                           | Комбинация модификаторов (Alt+I)                    |

Все клавиши описаны [здесь](https://github.com/vladislav-atakhanov/rkl/blob/9be2368e9ed05dd4ebae419e97add7e299d64dec/crates/keys/src/keys.rs#L84)

### Директивы

#### `keyboard` - встроенная клавиатура

Используется для задания матрицы клавиатуры в Vial. В будущем будет использоваться
для графического определения матрицы

```lisp
(keyboard imperial44)
```

#### `defsrc` - физическая раскладка

Определяет порядок клавиш на клавиатуре:

```lisp
(defsrc
    esc q w e r t       y u i o p bks
    tab a s d f g       h j k l ; enter
)
```

#### `deflayer` - определить слой

Каждая позиция соответствует клавише из `defsrc`:

```lisp
(deflayer default
    esc q w e r t       y u i o p bks
    tab a s d f g       h j k l ; enter
)
```

Имя `default` наследуется от `src`. Остальные слои наследуются от `default`, но можно явно
указать родительский слой (deflayer (new-layer parent-layer) ...)

#### `deflayermap` - частичное обновление слоя

Переопределяет конкретные клавиши, не затрагивая остальные:

```lisp
(deflayermap default
    a (tap-hold a M)
    s (tap-hold s A)
    d (tap-hold d C)
    f (tap-hold f S)
)
```

#### `defalias` - алиасы

```lisp
(defalias
    nav (layer-while-held nav)
    sym (layer-while-held sym)
)

;; использование: @nav, @sym
```

#### `deftemplate` - шаблоны

Параметры начинаются с `$`. Последний параметр может принимать произвольное количество аргументов:

```lisp
(deftemplate app ($x) (multi meta $x))

(defalias
    a0 (app 0)   ;; раскрывается в (multi meta 0)
    a1 (app 1)
)
```

#### `defoverride` - переопределение клавиш с модификаторами на определенном слое

```lisp
(defoverride layer-name
    A-i o    ;; Alt+I -> O
    A-m ]    ;; Alt+M -> ]
)
```

#### `defkeymap` - привязка раскладки языка к слою

```lisp
(defkeymap default en S-A-8)
```

### Действия

```lisp
(tap-hold a sft)                ;; при нажатии -> a, при зажатии -> Shift
(layer-while-held nav)          ;; активировать слой пока удерживается
(layer-switch game)             ;; переключиться на слой
(multi meta a)                  ;; нажать несколько клавиш одновременно
```

## Пример конфигурации

```lisp
(keyboard imperial44)

(defsrc
    esc q w e r t                    y u i o p bks
    tab a s d f g                    h j k l ; rmeta
    sft z x c v b [   up    pgup   ] n m , . / rsft
              C 1 spc dn    pgdn ent 2 A
)

;; Шаблон для быстрого запуска приложений через Meta+цифра
(deftemplate app ($x) (multi meta $x))
(defalias
    a0 (app 0) a1 (app 1) a2 (app 2) a3 (app 3) a4 (app 4)
    a5 (app 5) a6 (app 6) a7 (app 7) a8 (app 8) a9 (app 9)
    num (layer-while-held num)
    sym (layer-while-held sym)
)

;; Основной слой
(deflayer default
    X   q   w   e   r   t                       y    u   i   o   p   bks
    tab a   s   d   f   g                       h    j   k   l   ;   X
    X   z   x   c   v   b    X mwup   vol+ mute n    m   X   X   X   X
                    esc @num _ mwdn   vol- _    @sym bks
)

;; Home row mods
(deflayermap default
    a (tap-hold a M)    ; (tap-hold ; M)
    s (tap-hold s A)    l (tap-hold l A)
    d (tap-hold d C)    k (tap-hold k C)
    f (tap-hold f S)    j (tap-hold j S)
)

;; Цифры и навигация
(deflayer num
    _ 1   2   3   4   5                     6  7    8    9  0 _
    _ @a1 @a2 @a3 @a4 @a5                   lt dn   up   rt M _
    _ @a6 @a7 X   X   X   _ vol+      _   _ X  pgdn pgup X  X _
                  X   _   X vol-      _ del _  _
)

;; Символы
(deflayer sym
     _ .! .@ .# .$ .%             .^ .& .* .| .~ _
     _ .; .[ .{ lb .=             .- rb .} .] .: _
    .\ .` .' ." .< ._ _ _     _ _ .+ .> ., .. .? ./
                 _ _ _ _       _ X _  X
)
```
