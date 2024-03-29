LKNPD
=====

Тулза для пробивания чеков самозанятого.
Позволяет использовать шаблоны для создания чек.

Шаблоны
-------

Синтаксис простой, всё что находится между '{{ }}' интерпретируется либо как _переменная_
либо как _функция_. Перед генерацией чека будет запрошен пользовательский ввод для
всех _переменных_ а в момент генерации будут вычислены _функции_ и результаты будут подставлены в шаблон.

_Переменная_ обязательно должна начинаться с английской буквы и может содержать
английские буквы и цифры. После _переменной_, через `:`, можно указать _значение по-умолчанию_, которое будет предложено пользователю перед вводом значения для данной _переменной_.
_Значение по-умолчанию_ может быть как строка, так и _функция_.

На данный момент доступны следующие _функции_:
- `now` вернёт текущую дату в формате (RFC3339)[https://datatracker.ietf.org/doc/html/rfc3339]. Пример: `2024-03-08T16:23:37+07:00`.
- `year` вернёт текущий год, 4 цифры. Пример: `2024`.
- `month` вернёт название текущего месяца. Пример: `Февраль`.
