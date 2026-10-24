# Vice 0.2.0 — podręcznik dla studentów i hobbystów

> Kanoniczna dokumentacja użytkownika.

## Jak czytać ten podręcznik

Ten plik opisuje wyłącznie funkcje obecne w kodzie i testach Vice 0.2.0.
Składnia przykładów odpowiada parserowi, analizatorowi semantycznemu i VM.
Najlepsza kolejność nauki to rozdziały od pierwszego programu do testów.
Każdy przykład zapisuj w pliku z rozszerzeniem `.vice`.
Polecenia uruchamiaj w PowerShell, cmd albo terminalu VS Code.
Nie uruchamiaj niezaufanego kodu Vice, ponieważ moduł `fs` ma prawa procesu.
Vice jest obecnie językiem lokalnym; nie ma działającego modułu sieci ani FFI.
Renderer desktopowy jest opcjonalny; zwykłe programy działają w buildzie core.
Komentarze zaczynają się od `#` albo `//`.
Kanoniczne wcięcie ma cztery spacje.
Instrukcje kończą się średnikiem tam, gdzie pokazują to przykłady.
Strzałka `=>` przypisuje wartość przy deklaracji i nazywa argument.
Strzałka `<-` kieruje wartość do `emit`, komunikat testu do `fail` albo wejście do starszych aliasów.
Strzałka `->` opisuje wynik funkcji albo wejście `input -> nazwa`.

## Start w pięć minut

1. Zbuduj CLI poleceniem `cargo build --bin vice`.
2. Sprawdź instalację poleceniem `vice --version`.
3. Utwórz projekt poleceniem `vice new hello`.
4. Wejdź do katalogu poleceniem `cd hello`.
5. Uruchom projekt poleceniem `vice run .`.
6. Uruchom testy poleceniem `vice test .`.
7. Sprawdź typy poleceniem `vice check .`.
8. Sformatuj kod poleceniem `vice fmt .`.

## Układ projektu utworzonego przez CLI

`vice.toml` zawiera nazwę, wersję, entry i wersję języka.
`src/main.vice` jest domyślnym punktem wejścia.
`tests/main_test.vice` zawiera pierwszy test `@test`.
`.gitignore` ignoruje `target/` i pliki tymczasowe.
CLI akceptuje również pojedynczy plik bez manifestu.
Importy `use` są rozwiązywane względem pliku, który je deklaruje.
Manifest jest bezpiecznie odrzucany, jeśli entry wychodzi poza projekt.
Starszy `vice.json`, `@mount`, `output <-` i `error() <-` są traktowane jako kompatybilność migracyjna.
Nowy kod pisz jako `vice.toml`, `use`, `emit <-` i `fail <-`.

## Polecenia CLI

| Polecenie | Znaczenie |
|---|---|
| `vice run app.vice` | kompiluje i wykonuje plik |
| `vice run .` | znajduje entry projektu |
| `vice run . -- a b` | przekazuje `a` i `b` do `process.args()` |
| `vice check file.vice` | wykonuje pełny pipeline bez uruchomienia |
| `vice check --json .` | zwraca raport maszynowy |
| `vice test .` | odkrywa katalog `tests` projektu |
| `vice test --json .` | zwraca wyniki testów w JSON |
| `vice fmt .` | formatuje pliki rekursywnie |
| `vice fmt --check .` | sprawdza format bez zapisu |
| `vice fmt --stdout file.vice` | wypisuje sformatowany plik |
| `vice fmt --stdin` | formatuje tekst ze standardowego wejścia |
| `vice new name` | tworzy kompletny szkielet projektu |
| `vice init .` | inicjuje projekt w istniejącym katalogu |
| `vice migrate .` | konwertuje manifest i starszą składnię do stylu v0.2 |
| `vice doc .` | generuje prosty opis API projektu |
| `vice lsp` | uruchamia podstawowy punkt integracji LSP |
| `vice doctor --json` | raportuje środowisko narzędzia |
| `vice ui-check` | raportuje dostępne wiązania UI |
| `vice --version` | wypisuje wersję |

## Lekcja: Pierwszy program

W tej lekcji poznajesz punkt wejścia `@entry`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    emit <- "Cześć z Vice";
}
```

Ćwiczenie: Zmień tekst i uruchom program ponownie.
Ważne: Brak `@entry` jest dozwolony dla bibliotek, lecz nie dla uruchamianego programu.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Stałe lokalne

W tej lekcji poznajesz deklarację `let` z inferencją.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let answer => 42;
    emit <- answer;
}
```

Ćwiczenie: Dodaj drugą stałą typu `string`.
Ważne: Stałej zadeklarowanej przez `let` nie można później przypisać ponownie.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Zmienne mutowalne

W tej lekcji poznajesz deklarację `mut`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    mut score => 0;
    score = score + 10;
    emit <- score;
}
```

Ćwiczenie: Zwiększ wynik trzy razy.
Ważne: Używaj mutowalności tylko tam, gdzie stan naprawdę się zmienia.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Typy jawne

W tej lekcji poznajesz adnotacje typów.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let name: string => "Vice";
    let ready: bool => true;
    emit <- name, ready;
}
```

Ćwiczenie: Dodaj wartości `int` i `float`.
Ważne: Tryb wysokopoziomowy nie wykonuje niejawnej utraty części ułamkowej.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Tablice

W tej lekcji poznajesz typ `T[]` oraz indeksowanie.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let values: int[] => [4, 8, 15];
    emit <- values[0], len(values);
}
```

Ćwiczenie: Odczytaj ostatni element.
Ważne: Indeks spoza zakresu kończy wykonanie błędem runtime.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Mapy typowane

W tej lekcji poznajesz typ `map[K, V]`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let ports: map[string, int] => {"web": 80, "tls": 443};
    emit <- ports["tls"];
}
```

Ćwiczenie: Dodaj trzeci wpis mapy.
Ważne: Klucze map zachowują typ; `1` i `"1"` nie są tym samym kluczem.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Funkcje

W tej lekcji poznajesz parametry i typ wyniku.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
fn add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    emit <- add(20, 22);
}
```

Ćwiczenie: Napisz funkcję mnożącą.
Ważne: Każda osiągalna ścieżka funkcji niewoidowej musi zwrócić wartość.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Argumenty nazwane

W tej lekcji poznajesz czytelne wywołania.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
fn greet(name: string) {
    emit <- "Witaj {name}";
}

@entry
fn main() {
    greet(name => "Ada");
}
```

Ćwiczenie: Zmień nazwę parametru i wywołanie.
Ważne: Nazwa argumentu musi odpowiadać nazwie parametru.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Warunki

W tej lekcji poznajesz instrukcję `if`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let temperature => 24;
    if (temperature > 20) {
        emit <- "ciepło";
    } else {
        emit <- "chłodno";
    }
}
```

Ćwiczenie: Dodaj trzeci próg przez zagnieżdżony warunek.
Ważne: Warunek musi mieć typ `bool` i wymaga nawiasów.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Pętla while

W tej lekcji poznajesz kontrolowaną iterację.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    mut index => 0;
    while (index < 3) {
        emit <- index;
        index = index + 1;
    }
}
```

Ćwiczenie: Zatrzymaj pętlę po pięciu krokach.
Ważne: VM ma limit instrukcji dostępny w konfiguracji hosta.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Pętla zakresowa

W tej lekcji poznajesz zakres `start..end`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    for index in 0..3 {
        emit <- index;
    }
}
```

Ćwiczenie: Policz sumę zakresu.
Ważne: Koniec zakresu nie jest włączony.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Pętla foreach

W tej lekcji poznajesz iterację po kolekcji.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let names: string[] => ["Ada", "Linus"];
    for name in names {
        emit <- name;
    }
}
```

Ćwiczenie: Wypisz długość każdego napisu.
Ważne: Nie modyfikuj kolekcji w trakcie iteracji.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Sterowanie pętlą

W tej lekcji poznajesz `break` i `continue`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    for value in 0..10 {
        if (value == 2) { continue; }
        if (value == 5) { break; }
        emit <- value;
    }
}
```

Ćwiczenie: Przesuń oba progi.
Ważne: `break` i `continue` poza pętlą są błędami semantycznymi.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Dane

W tej lekcji poznajesz deklaracje `data`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
data User {
    name: string => "guest";
    points: int => 0;
}

@entry
fn main() {
    let user => User(name => "Ada", points => 10);
    emit <- user.name;
}
```

Ćwiczenie: Dodaj pole `active: bool`.
Ważne: Wartość domyślna pola musi być zgodna z jego typem.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Enumy

W tej lekcji poznajesz zamknięte zbiory wariantów.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
enum State { Idle, Running, Done }

@entry
fn main() {
    let state: State => State.Running;
    emit <- state;
}
```

Ćwiczenie: Dodaj wariant błędu.
Ważne: Nieistniejący wariant jest odrzucany przed uruchomieniem.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Match

W tej lekcji poznajesz dopasowanie wzorca.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let value => 2;
    match value {
        1 => { emit <- "jeden"; }
        2 | 3 => { emit <- "dwa lub trzy"; }
        _ => { emit <- "inne"; }
    }
}
```

Ćwiczenie: Dodaj ramię dla zakresu.
Ważne: Ramię `_` musi być ostatnie.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Traity

W tej lekcji poznajesz jawny kontrakt typu.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
trait Named {
    fn name(self: Self) -> string;
}
```

Ćwiczenie: Zaprojektuj trait z jedną metodą odczytu.
Ważne: Implementacja traitu musi mieć wszystkie i tylko zadeklarowane metody.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Impl

W tej lekcji poznajesz metody danych.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
data Counter { value: int => 0; }

impl Counter {
    fn current(self: Counter) -> int {
        return self.value;
    }
}
```

Ćwiczenie: Dodaj metodę zwracającą podwojoną wartość.
Ważne: `self` ma konkretny typ implementowanego obiektu.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Result

W tej lekcji poznajesz jawny sukces albo błąd.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let result => result.ok(42);
    emit <- result.is_ok;
}
```

Ćwiczenie: Utwórz wartość przez `result.err("problem", 1)`.
Ważne: Operator `?` propaguje błąd zgodnie z typem wyniku.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Option

W tej lekcji poznajesz wartość opcjonalną.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let value => option.some("Vice");
    emit <- value.is_some;
}
```

Ćwiczenie: Utwórz `option.none()` i sprawdź stan.
Ważne: Nie używaj unwrap bez wcześniejszego sprawdzenia.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Błędy Vice

W tej lekcji poznajesz strzałkową instrukcję `error`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let valid => false;
    if (!valid) {
        error(code => 2) <- "dane są niepoprawne";
    }
}
```

Ćwiczenie: Zmień kod i komunikat.
Ważne: `error(...) <- ...;` kończy bieżące wykonanie i tworzy stack trace.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Testy

W tej lekcji poznajesz atrybut `@test`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@test
fn addition_works() {
    if (20 + 22 != 42) {
        fail <- "dodawanie zawiodło";
    }
}
```

Ćwiczenie: Uruchom `vice test .`.
Ważne: Każdy test jest kompilowany i wykonywany osobno.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Importy

W tej lekcji poznajesz lokalne moduły przez `use`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
use "math.vice";

@entry
fn main() {
    emit <- add(2, 3);
}
```

Ćwiczenie: Utwórz plik `math.vice` z funkcją `add`.
Ważne: Importy są rozwiązywane względem pliku importującego, a cykle są błędem.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Wejście i wyjście

W tej lekcji poznajesz strzałki danych.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    input -> name;
    emit <- "Witaj", name;
}
```

Ćwiczenie: Wczytaj dwie wartości.
Ważne: CLI buforuje wyjście VM i wypisuje każdą linię po wykonaniu.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Argumenty procesu

W tej lekcji poznajesz `process.args()`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let args: string[] => process.args();
    emit <- args;
}
```

Ćwiczenie: Uruchom `vice run app.vice -- first second`.
Ważne: Tylko wartości po separatorze `--` trafiają do programu.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Pliki tekstowe

W tej lekcji poznajesz moduł `fs`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    fs.write_text("note.txt", "Vice");
    fs.append_text("note.txt", " works");
    emit <- fs.read_text("note.txt");
}
```

Ćwiczenie: Zapisz log w podkatalogu.
Ważne: Błędy systemu plików są błędami runtime, a nie fałszywym `false`.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: JSON

W tej lekcji poznajesz odczyt i zapis danych.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    fs.write_json("config.json", {"mode": "dev"});
    let config => fs.read_json("config.json");
    emit <- config["mode"];
}
```

Ćwiczenie: Dodaj drugie pole tekstowe.
Ważne: `write_json` zapisuje czytelny JSON zakończony nową linią.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Ścieżki

W tej lekcji poznajesz przenośne operacje `path`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let file => path.join("data", "items.json");
    emit <- path.basename(file), path.extension(file);
}
```

Ćwiczenie: Sprawdź `path.dirname`.
Ważne: Do łączenia segmentów używaj `path.join`, nie ręcznych separatorów.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Katalogi

W tej lekcji poznajesz listowanie i inspekcję.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    fs.create_dir("data/cache");
    emit <- path.is_dir("data"), fs.list_dir("data");
}
```

Ćwiczenie: Utwórz dwa pliki i sprawdź kolejność.
Ważne: `fs.list_dir` zwraca posortowane nazwy elementów.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Łańcuchy

W tej lekcji poznajesz Unicode i operacje tekstowe.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    emit <- string.len("żółw");
    emit <- string.upper("vice");
}
```

Ćwiczenie: Użyj `split`, `trim` i `contains`.
Ważne: `string.len` liczy znaki Unicode, nie bajty UTF-8.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Matematyka

W tej lekcji poznajesz funkcje numeryczne.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    emit <- math.sqrt(9.0), math.floor(3.9);
}
```

Ćwiczenie: Sprawdź `sin`, `cos`, `pow`, `min` i `max`.
Ważne: `floor`, `ceil` i `round` zwracają `int`.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Losowość

W tej lekcji poznajesz moduł `random`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    emit <- random.int(1, 6), random.float();
}
```

Ćwiczenie: Zasymuluj dziesięć rzutów.
Ważne: Generator jest prosty i nie jest przeznaczony do kryptografii.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Tryby liczbowe

W tej lekcji poznajesz atrybuty highlevel i lowlevel.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@highlevel
@entry
fn main() {
    let ratio: float => 5 / 2;
    emit <- ratio;
}
```

Ćwiczenie: Porównaj zachowanie z `@lowlevel`.
Ważne: Highlevel chroni przed niejawną utratą precyzji.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Read-only

W tej lekcji poznajesz atrybut `@read`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@read
fn label(value: int) -> string {
    return string(value);
}
```

Ćwiczenie: Napisz funkcję tylko do odczytu.
Ważne: Funkcja `@read` nie może wykonywać efektów ubocznych ani wywoływać funkcji zapisujących.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Formatowanie

W tej lekcji poznajesz kanoniczny styl kodu.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main(){
# komentarz zostanie zachowany
emit <- "Vice";
}
```

Ćwiczenie: Uruchom `vice fmt --check .`, a potem `vice fmt .`.
Ważne: Formatter zachowuje komentarze `#` i `//`.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Diagnostyka

W tej lekcji poznajesz pipeline `vice check`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let value: int => 42;
    emit <- value;
}
```

Ćwiczenie: Uruchom także wariant `--json`.
Ważne: Kod wyjścia 1 oznacza błąd programu, a 2 błędne użycie CLI.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Projekt

W tej lekcji poznajesz manifest `vice.toml`.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```toml
[package]
name = "sample"
version = "0.2.0"
entry = "src/main.vice"

[paths]
source = "src"
tests = "tests"
examples = "examples"

[features]
desktop = false
audio = false
```

Ćwiczenie: Utwórz projekt przez `vice new sample`.
Ważne: Ścieżka entry nie może być absolutna ani wychodzić przez `..`.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: VS Code

W tej lekcji poznajesz diagnostyki i formatter.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
vice check --json src/main.vice
vice fmt --stdin
```

Ćwiczenie: Ustaw `vice.executablePath`, jeśli CLI nie jest w PATH.
Ważne: Rozszerzenie używa `spawn` bez powłoki i sprawdza pliki po zapisie.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Migracja z 0.1.1 do 0.2.0

W Vice 0.2 preferowany styl jest bardziej spójny z przepływem danych języka.
Uruchom `vice migrate .` w katalogu projektu, a potem `vice check .`.
Migrator tworzy `vice.toml` z `vice.json`, zamienia `@mount` na `use`, `output <-` na `emit <-` oraz `error() <-` na `fail <-`.
Jeżeli projekt używał identyfikatora `fail`, zmień nazwę na przykład na `should_fail`, ponieważ `fail` jest teraz słowem kluczowym instrukcji testowej.
Stare formy pozostają czytelne dla kompatybilności, ale nowa dokumentacja i formatter promują styl v0.2.

## Lekcja: UI desktop

W tej lekcji poznajesz okno i elementy.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let window => ui.create(title => "Vice", size => [800, 600]);
    emit <- window;
}
```

Ćwiczenie: Sprawdź przykład UI bez wymyślania metod spoza rejestru.
Ważne: Funkcje UI wymagają buildu z funkcją `desktop`; core ma bezpieczny stub typów.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Audio

W tej lekcji poznajesz opcjonalną funkcję Cargo.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    audio.play_sound("click.ogg");
}
```

Ćwiczenie: Uruchom build z funkcją `audio`.
Ważne: Bez funkcji `audio` runtime zwraca jawny błąd zamiast udawać sukces.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Lekcja: Bezpieczeństwo

W tej lekcji poznajesz granice zaufania.
Najpierw przepisz przykład bez zmian.
Następnie uruchom `vice check` i dopiero potem `vice run`.

```vice
@entry
fn main() {
    let candidate => process.args();
    emit <- candidate;
}
```

Ćwiczenie: Waliduj argument przed użyciem jako ścieżka.
Ważne: Vice nie izoluje programu od systemu plików; uruchamiaj tylko zaufany kod.
Jeżeli check zgłasza błąd, przeczytaj typ, plik, linię i kolumnę diagnostyki.
Jeżeli runtime zgłasza błąd, przejrzyj stack trace od najbliższej funkcji.
Po zmianie kodu uruchom `vice fmt`, aby utrzymać styl Vice.

## Kompletna ściąga biblioteki standardowej

Sygnatury poniżej pochodzą z rejestru natywnego analizatora semantycznego.
`unknown` oznacza wartość dynamiczną, najczęściej wynik JSON.
Metoda spoza tej listy jest odrzucana jako nieznana.

### `time.now_ms()`
Zwraca: `int`.
Zastosowanie: czas Unix w milisekundach.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.abs(number)`
Zwraca: `number`.
Zastosowanie: wartość bezwzględna.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.sin(float)`
Zwraca: `float`.
Zastosowanie: sinus.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.cos(float)`
Zwraca: `float`.
Zastosowanie: cosinus.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.tan(float)`
Zwraca: `float`.
Zastosowanie: tangens.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.sqrt(float)`
Zwraca: `float`.
Zastosowanie: pierwiastek.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.floor(float)`
Zwraca: `int`.
Zastosowanie: zaokrąglenie w dół.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.ceil(float)`
Zwraca: `int`.
Zastosowanie: zaokrąglenie w górę.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.round(float)`
Zwraca: `int`.
Zastosowanie: zaokrąglenie.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.pow(float, float)`
Zwraca: `float`.
Zastosowanie: potęga.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.min(number, number)`
Zwraca: `number`.
Zastosowanie: minimum.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.max(number, number)`
Zwraca: `number`.
Zastosowanie: maksimum.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `math.random()`
Zwraca: `float`.
Zastosowanie: wartość od 0 do 1.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `random.int(int, int)`
Zwraca: `int`.
Zastosowanie: liczba całkowita z zakresu.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `random.float()`
Zwraca: `float`.
Zastosowanie: wartość od 0 do 1.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `string.len(string)`
Zwraca: `int`.
Zastosowanie: liczba znaków Unicode.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `string.contains(string, string)`
Zwraca: `bool`.
Zastosowanie: test fragmentu.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `string.starts_with(string, string)`
Zwraca: `bool`.
Zastosowanie: test początku.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `string.ends_with(string, string)`
Zwraca: `bool`.
Zastosowanie: test końca.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `string.split(string, string)`
Zwraca: `string[]`.
Zastosowanie: podział tekstu.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `string.trim(string)`
Zwraca: `string`.
Zastosowanie: usunięcie białych znaków.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `string.lower(string)`
Zwraca: `string`.
Zastosowanie: małe litery.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `string.upper(string)`
Zwraca: `string`.
Zastosowanie: wielkie litery.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `array.len(array)`
Zwraca: `int`.
Zastosowanie: liczba elementów.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `array.first(array)`
Zwraca: `T`.
Zastosowanie: pierwszy element.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `array.last(array)`
Zwraca: `T`.
Zastosowanie: ostatni element.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `array.join(array, string)`
Zwraca: `string`.
Zastosowanie: łączenie elementów.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `fs.read_text(string)`
Zwraca: `string`.
Zastosowanie: odczyt UTF-8.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `fs.write_text(string, string)`
Zwraca: `bool`.
Zastosowanie: zapis tekstu.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `fs.append_text(string, string)`
Zwraca: `bool`.
Zastosowanie: dopisywanie tekstu.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `fs.read_json(string)`
Zwraca: `unknown`.
Zastosowanie: odczyt JSON.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `fs.write_json(string, unknown)`
Zwraca: `bool`.
Zastosowanie: zapis JSON.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `fs.list_dir(string)`
Zwraca: `string[]`.
Zastosowanie: posortowane nazwy.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `fs.create_dir(string)`
Zwraca: `bool`.
Zastosowanie: rekursywne tworzenie katalogu.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `path.join(string, string)`
Zwraca: `string`.
Zastosowanie: połączenie segmentów.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `path.dirname(string)`
Zwraca: `string`.
Zastosowanie: katalog nadrzędny.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `path.basename(string)`
Zwraca: `string`.
Zastosowanie: nazwa końcowa.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `path.extension(string)`
Zwraca: `string`.
Zastosowanie: rozszerzenie bez kropki.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `path.exists(string)`
Zwraca: `bool`.
Zastosowanie: istnienie ścieżki.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `path.is_file(string)`
Zwraca: `bool`.
Zastosowanie: test pliku.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `path.is_dir(string)`
Zwraca: `bool`.
Zastosowanie: test katalogu.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `env.get(string)`
Zwraca: `string`.
Zastosowanie: odczyt zmiennej środowiska.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `env.set(string, string)`
Zwraca: `bool`.
Zastosowanie: ustawienie zmiennej procesu.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `process.args()`
Zwraca: `string[]`.
Zastosowanie: argumenty po `--`.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `process.cwd()`
Zwraca: `string`.
Zastosowanie: bieżący katalog.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `json.parse(string)`
Zwraca: `unknown`.
Zastosowanie: parsowanie JSON.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `json.stringify(unknown)`
Zwraca: `string`.
Zastosowanie: kompaktowy JSON.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `engine.load_scene(string)`
Zwraca: `Level`.
Zastosowanie: wczytanie sceny `.vscn`.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `audio.play_sound(string)`
Zwraca: `bool`.
Zastosowanie: odtworzenie efektu.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

### `audio.play_music(string, float)`
Zwraca: `bool`.
Zastosowanie: zapętlona muzyka.
Błędy argumentów lub systemu są raportowane jawnie przez runtime.

## Codzienny workflow

Zacznij zmianę od małego testu `@test`.
Uruchom `vice test .`.
Zaimplementuj minimalną funkcję.
Uruchom `vice check .`.
Uruchom `vice fmt .`.
Uruchom ponownie `vice test .`.
Przed commitem użyj `vice fmt --check .` i `vice check --json .`.
Nie ignoruj ostrzeżeń systemu plików ani błędów importu.
Nie dodawaj wywołań natywnych, których nie ma w ściądze.

## Granice obecnej wersji

Vice 0.2.0 nie ma menedżera pakietów.
Vice 0.2.0 nie ma stabilnego formatu skompilowanego artefaktu.
Vice 0.2.0 nie ma działającego API HTTP ani TCP.
Vice 0.2.0 nie ma działającego FFI do JavaScript, Pythona ani Rusta.
Vice 0.2.0 nie ma garbage collectora; VM używa wartości zarządzanych przez Rusta.
Rozszerzenie VS Code daje diagnostyki po zapisie, a nie pełne podpowiedzi LSP.
UI i audio są funkcjami opcjonalnymi buildu Cargo.
To są świadome granice, nie ukryte obietnice.

## Pytania kontrolne


## Koniec

Po przejściu podręcznika potrafisz tworzyć, sprawdzać, testować i uruchamiać lokalne projekty Vice.
Źródłem prawdy dla implementacji jest podręcznik developera oraz kod objęty testami.

