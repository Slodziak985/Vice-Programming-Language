# Vice 0.2.0 — dokumentacja dla AI
Linia 001: Ten plik jest kanonicznym przewodnikiem dla modeli AI pracujących z projektem Vice.
Linia 002: AI ma traktować ten dokument jako kontrakt zgodny ze stanem repozytorium po przebudowie.
Linia 003: Vice jest językiem kompilowanym do własnego bajtkodu i uruchamianym przez VM w Rust.
Linia 004: Vice nie ma obecnie silnika przeglądarkowego jako części runtime języka.
Linia 005: Vice posiada natywne API sieciowe http, C-FFI ffi, moduł macierzowy matrix, kryptografię crypto, bazę danych sqlite, silnik wyrażeń regularnych regex oraz menedżer pakietów pkg.
Linia 005a: Polecenie `vice schema` generuje kompletny maszynowy JSON schema wszystkich dostępnych natywnych modułów i ich sygnatur.
Linia 005b: Struktury `data` oraz tablice `array` w Vice posiadają semantykę wartości (pass-by-value / immutable updates); metody modyfikujące tablice takie jak `array.push(arr, elem)` zwracają zaktualizowaną tablicę: `arr = array.push(arr, elem);`.
Linia 006: AI nie może obiecywać funkcji usuniętych albo niepodłączonych do kompilatora.
Linia 007: AI ma preferować opis tego, co faktycznie działa w CLI, parserze, semantyce, VM, stdlib i Web.
Linia 008: Repozytorium zawiera język, CLI, bibliotekę Rust, przykłady, testy, VS Code extension candidate i stronę Web.
Linia 009: Głównym celem Vice jest czytelny przepływ danych i jawne punkty decyzji.
Linia 010: Styl Vice ma być prosty, techniczny, wizualny i oparty o trzy jawne strzałki: `=>`, `<-` i `->`.

## 1. Zasady nadrzędne dla AI
Linia 011: Najpierw sprawdzaj pliki projektu, potem formułuj wnioski.
Linia 012: Nie generuj dokumentacji sprzecznej z kodem.
Linia 013: Nie zakładaj, że funkcja istnieje, jeżeli nie ma jej w parserze, semantyce, kompilatorze, VM lub stdlib.
Linia 014: Nie używaj starego README jako źródła prawdy, bo dokumentacje zostały skonsolidowane.
Linia 015: Kanoniczne dokumenty w workspace to trzy pliki: studencki, developerski i AI.
Linia 016: `AGENTS.md` może istnieć jako instrukcja robocza, ale nie jest dokumentacją użytkownika języka.
Linia 017: AI ma chronić zmiany użytkownika w repozytorium.
Linia 018: AI ma sprawdzać `git status --short` przed większymi edycjami.
Linia 019: AI ma unikać dotykania plików niezwiązanych z zadaniem.
Linia 020: AI ma szczególnie uważać na lokalne zmiany w `AGENTS.md`.
Linia 021: AI ma używać `rg` do szukania tekstu i plików.
Linia 022: AI ma używać `apply_patch` do edycji plików.
Linia 023: AI nie ma wykonywać destrukcyjnych poleceń git bez wyraźnej prośby użytkownika.
Linia 024: AI nie ma używać `git reset --hard`.
Linia 025: AI nie ma usuwać zmian użytkownika przez checkout.
Linia 026: AI ma robić commity logiczne i opisowe.
Linia 027: AI ma uruchamiać testy proporcjonalnie do zakresu zmian.
Linia 028: Dla zmian Rust uruchamiaj `cargo test --all-targets`.
Linia 029: Dla zmian Rust uruchamiaj `cargo clippy --all-targets -- -D warnings`, jeżeli zmiana dotyczy kodu.
Linia 030: Dla zmian Web uruchamiaj testy npm dostępne w `Web/package.json`.
Linia 031: Dla zmian reguł Firestore uruchamiaj testy reguł, jeżeli środowisko ma emulator.
Linia 032: Dla zmian dokumentacji sprawdzaj liczbę linii i zgodność nazw plików.
Linia 033: AI ma pisać o Vice w tonie konkretnym i bez marketingowej przesady.
Linia 034: AI ma używać polskich nazw w polskiej dokumentacji.
Linia 035: AI może używać angielskich nazw technicznych, gdy są nazwami API.
Linia 036: AI ma tłumaczyć błędy przez przyczynę i działanie naprawcze.
Linia 037: AI nie ma tworzyć fałszywych przykładów CLI.
Linia 038: AI nie ma dodawać API `net.*`, `ffi.*` ani starego engine browser.
Linia 039: AI ma odróżniać Web społecznościowy od runtime języka.
Linia 040: AI ma odróżniać `Web` od hipotetycznego katalogu `web`.

## 2. Mentalny model Vice
Linia 041: Program Vice jest tekstem parsowanym do AST.
Linia 042: AST jest sprawdzane semantycznie przed kompilacją do bajtkodu.
Linia 043: Bajtkod jest wykonywany przez VM.
Linia 044: VM obsługuje stos wartości i ramki wywołań.
Linia 045: Wartości runtime obejmują liczby, bool, stringi, tablice, mapy, dane, enumy i void.
Linia 046: Vice preferuje jawne wiązania wartości.
Linia 047: Podstawowy zapis deklaracji to `let nazwa => wyrazenie;`.
Linia 048: Zmienna mutowalna używa `mut nazwa => wyrazenie;`.
Linia 049: Ponowne przypisanie do istniejącej zmiennej używa `nazwa = wyrazenie;`.
Linia 050: `input -> nazwa;` czyta wejście tekstowe do zmiennej.
Linia 051: `emit <- wyrazenie;` jest preferowaną składnią wysyłania wartości do wyjścia programu.
Linia 052: `@entry` oznacza jawny punkt startowy programu.
Linia 053: Funkcja `main` może pełnić rolę wejścia, jeśli nie ma `@entry`.
Linia 054: `@test` oznacza test wykonywany przez `vice test`.
Linia 055: Biblioteka może przejść `vice check` bez funkcji wejściowej.
Linia 056: Aplikacja uruchamiana przez `vice run` potrzebuje wejścia.
Linia 057: Test w Vice kończy się błędem przez `fail <- "opis";`.
Linia 058: Test nie używa osobnej składni `assert`.
Linia 059: Komentarze `#` są obsługiwane.
Linia 060: Komentarze `//` są obsługiwane.
Linia 061: Formatter zachowuje komentarze.
Linia 062: Formatter nie powinien niszczyć URL-i w stringach.
Linia 063: Bloki są ograniczane klamrami.
Linia 064: Instrukcje kończą się średnikiem tam, gdzie wymaga tego składnia.
Linia 065: Warunek `if` używa nawiasów.
Linia 066: Pętla `while` używa nawiasów.
Linia 067: Zakres `start..end` w pętli `for` ma koniec wyłączny.
Linia 068: `match` używa ramion z `=>`.
Linia 069: W `match` `_` jest wzorcem domyślnym.
Linia 070: Wzorzec domyślny powinien być ostatni.

## 3. Składnia funkcji
Linia 071: Funkcja zaczyna się od `fn`.
Linia 072: Nazwa funkcji powinna być czytelna i stabilna.
Linia 073: Parametr funkcji zapisuj jako `nazwa: typ`.
Linia 074: Wynik funkcji zapisuj jako `-> typ`.
Linia 075: Funkcja bez wyniku może używać `void`.
Linia 076: Ciało funkcji jest blokiem.
Linia 077: `return` zwraca wartość z funkcji.
Linia 078: `return;` pasuje do `void`.
Linia 079: Wejście programu powinno być małe i delegować pracę do nazwanych funkcji.
Linia 080: AI powinno generować funkcje o jednej odpowiedzialności.
Linia 081: AI powinno unikać ogromnych funkcji bez nazwanych kroków.
Linia 082: AI powinno używać typów dokładnie zgodnych z aktualnym systemem typów.
Linia 083: AI powinno pisać `let nazwa => wyrazenie;` dla niemutowalnych deklaracji.
Linia 084: AI nie powinno pisać `var`, jeżeli nie jest to składnia Vice.
Linia 085: AI nie powinno pisać importów nieobsługiwanych przez projekt.
Linia 086: AI ma używać `use "plik.vice";` dla nowych importów.
Linia 087: AI ma traktować `@mount "plik.vice";` jako legacy alias migracyjny.
Linia 088: AI nie powinno wymyślać dekoratorów bez implementacji parsera.
Linia 089: AI nie powinno mieszać składni Vice ze składnią Rust, JavaScript ani Python.
Linia 090: AI powinno pokazywać przykład Vice jako Vice, a nie pseudokod.

## 4. Typy
Linia 091: Typ `int` oznacza liczby całkowite.
Linia 092: Typ `float` oznacza liczby zmiennoprzecinkowe.
Linia 093: Typ `bool` oznacza prawdę albo fałsz.
Linia 094: Typ `string` oznacza tekst.
Linia 095: Typ `void` oznacza brak wartości użytkowej.
Linia 096: Tablica ma zapis `T[]`.
Linia 097: Mapa ma zapis `map[K,V]`.
Linia 098: Referencja niemutowalna ma zapis `&T`.
Linia 099: Referencja mutowalna ma zapis `&mut T`.
Linia 100: Dane użytkownika są definiowane przez `data`.
Linia 101: Enumy użytkownika są definiowane przez `enum`.
Linia 102: Traity użytkownika są definiowane przez `trait`.
Linia 103: Implementacje traitów są definiowane przez `impl`.
Linia 104: Typ wewnętrzny `unknown` nie powinien być promowany w dokumentacji użytkownika.
Linia 105: AI ma unikać niejawnego `null`.
Linia 106: Vice nie powinien być opisywany jako język z `null`.
Linia 107: Wartość braku wyniku to `void`.
Linia 108: AI ma nie dodawać przykładowych typów generycznych bez sprawdzenia parsera.
Linia 109: AI ma rozróżniać tablice i mapy.
Linia 110: AI ma nie zakładać klas, dziedziczenia klas ani konstruktorów klas.

## 5. Dane, enumy i dopasowanie
Linia 111: `data` opisuje rekord z nazwanymi polami.
Linia 112: Pole danych ma nazwę i typ.
Linia 113: Pole może mieć wartość domyślną przez `=>`.
Linia 114: Konstrukcja danych powinna używać nazwanych pól.
Linia 115: AI powinno preferować jawne nazwy pól nad pozycjami.
Linia 116: `enum` opisuje zamknięty zestaw wariantów.
Linia 117: Wariant enuma zapisuj przez `Typ.Wariant`.
Linia 118: `match` powinien obsługiwać warianty enuma czytelnie.
Linia 119: `match` powinien mieć przypadek domyślny, gdy nie pokrywa wszystkich wartości.
Linia 120: AI nie ma zakładać sum algebraicznych z dowolną składnią.
Linia 121: AI ma sprawdzić obsługiwane wzorce przed dodaniem przykładów zaawansowanych.
Linia 122: Wzorce mogą obejmować literały.
Linia 123: Wzorce mogą obejmować zakresy, jeżeli parser i semantyka je obsługują w aktualnym pliku.
Linia 124: Wzorzec `_` oznacza resztę przypadków.
Linia 125: Kolejność ramion `match` ma znaczenie.
Linia 126: Kod generowany dla `match` powinien być krótki i kompletny.
Linia 127: AI powinno nie zostawiać pustych ramion.
Linia 128: AI powinno używać komunikatów błędów dla niemożliwych stanów.
Linia 129: W dokumentacji unikaj twierdzenia, że pattern matching jest pełny jak w ML.
Linia 130: Dokumentuj tylko zakres zaimplementowany w repozytorium.

## 6. Traity i implementacje
Linia 131: Trait określa kontrakt zachowania.
Linia 132: Pierwszy parametr metody traita powinien reprezentować `self`.
Linia 133: Implementacja traita musi odpowiadać podpisom traita.
Linia 134: Semantyka sprawdza zgodność implementacji.
Linia 135: AI ma generować trait tylko wtedy, gdy kontrakt powtarza się dla wielu typów.
Linia 136: AI nie ma używać traitów jako zamiennika modułów.
Linia 137: AI ma nie zakładać dynamic dispatch, jeśli nie wynika to z kodu.
Linia 138: AI ma pisać podpisy traitów precyzyjnie.
Linia 139: AI ma nie ignorować typu `self`.
Linia 140: AI ma dodawać testy semantyczne dla nowych reguł traitów.
Linia 141: Jeżeli zmieniasz trait w AST, sprawdź parser, semantykę i kompilator.
Linia 142: Jeżeli zmieniasz trait w runtime, sprawdź VM.
Linia 143: Błąd traita powinien wskazywać nazwę traita i metody.
Linia 144: Dokumentacja traita powinna mieć przykład poprawny i przykład błędny.
Linia 145: AI powinno utrzymywać styl komunikatów diagnostycznych.
Linia 146: AI ma unikać pojęcia klasy bazowej.
Linia 147: Trait w Vice jest kontraktem, nie klasą.
Linia 148: Implementacja jest jawna.
Linia 149: Trait ma pomagać czytelności projektu.
Linia 150: Trait nie powinien ukrywać przepływu danych.

## 7. Sterowanie przepływem
Linia 151: `if` służy do wyboru ścieżki.
Linia 152: `else` jest alternatywą.
Linia 153: Warunek `if` musi być logiczny.
Linia 154: `while` służy do pętli zależnej od warunku.
Linia 155: `for name in start..end` służy do zakresu liczbowego.
Linia 156: Koniec zakresu `..` jest wyłączny.
Linia 157: `break` kończy pętlę.
Linia 158: `continue` przechodzi do następnej iteracji.
Linia 159: `return` kończy funkcję.
Linia 160: AI ma nie generować pętli nieskończonych bez jasnego warunku przerwania.
Linia 161: AI ma unikać ukrytych efektów ubocznych w warunkach.
Linia 162: AI ma preferować nazwane zmienne pośrednie dla złożonych warunków.
Linia 163: AI ma nie mieszać `match` i `if` bez potrzeby.
Linia 164: AI ma wybierać `match`, gdy kod rozróżnia warianty enuma.
Linia 165: AI ma wybierać `if`, gdy kod sprawdza prosty warunek logiczny.
Linia 166: AI ma pisać warunki w nawiasach.
Linia 167: AI ma sprawdzić formatter po zmianach składni sterowania.
Linia 168: Testy sterowania przepływem powinny obejmować gałęzie pozytywne i negatywne.
Linia 169: Błędy sterowania przepływem powinny wskazywać brakujący typ albo nieosiągalny stan.
Linia 170: Dokumentacja powinna pokazywać minimalne przykłady gałęzi.

## 8. Biblioteka standardowa
Linia 171: `time.now_ms()` zwraca czas w milisekundach.
Linia 172: `math.abs()` obsługuje wartość bezwzględną.
Linia 173: `math.sin()` obsługuje sinus.
Linia 174: `math.cos()` obsługuje cosinus.
Linia 175: `math.tan()` obsługuje tangens.
Linia 176: `math.sqrt()` obsługuje pierwiastek.
Linia 177: `math.floor()` obsługuje zaokrąglenie w dół.
Linia 178: `math.ceil()` obsługuje zaokrąglenie w górę.
Linia 179: `math.round()` obsługuje zaokrąglenie do najbliższej wartości.
Linia 180: `math.pow()` obsługuje potęgowanie.
Linia 181: `math.min()` zwraca mniejszą wartość.
Linia 182: `math.max()` zwraca większą wartość.
Linia 183: `math.random()` zwraca losową wartość zgodnie z kontraktem implementacji.
Linia 184: `random.int()` generuje liczbę całkowitą.
Linia 185: `random.float()` generuje liczbę zmiennoprzecinkową.
Linia 186: `string.contains()` sprawdza fragment tekstu.
Linia 187: `string.starts_with()` sprawdza prefiks.
Linia 188: `string.ends_with()` sprawdza sufiks.
Linia 189: `string.len()` zwraca długość tekstu.
Linia 190: `string.split()` dzieli tekst.
Linia 191: `string.trim()` usuwa białe znaki z brzegów.
Linia 192: `string.lower()` zmienia tekst na małe litery.
Linia 193: `string.upper()` zmienia tekst na wielkie litery.
Linia 194: `array.len()` zwraca długość tablicy.
Linia 195: `array.join()` łączy elementy tablicy w tekst.
Linia 196: `array.first()` zwraca pierwszy element według kontraktu runtime.
Linia 197: `array.last()` zwraca ostatni element według kontraktu runtime.
Linia 198: `fs.read_text()` czyta plik tekstowy.
Linia 199: `fs.write_text()` zapisuje plik tekstowy.
Linia 200: `fs.append_text()` dopisuje tekst do pliku.
Linia 201: `fs.read_json()` czyta JSON.
Linia 202: `fs.write_json()` zapisuje JSON w czytelnej formie.
Linia 203: `fs.list_dir()` zwraca posortowane nazwy wpisów katalogu.
Linia 204: `fs.create_dir()` tworzy katalog.
Linia 205: `path.join()` łączy fragmenty ścieżki.
Linia 206: `path.dirname()` zwraca katalog nadrzędny.
Linia 207: `path.basename()` zwraca nazwę końcową ścieżki.
Linia 208: `path.extension()` zwraca rozszerzenie.
Linia 209: `path.exists()` sprawdza istnienie ścieżki.
Linia 210: `path.is_file()` sprawdza, czy ścieżka jest plikiem.
Linia 211: `path.is_dir()` sprawdza, czy ścieżka jest katalogiem.
Linia 212: `env.get()` czyta zmienną środowiskową.
Linia 213: `env.set()` ustawia zmienną środowiskową procesu.
Linia 214: `process.args()` zwraca argumenty programu.
Linia 215: `process.cwd()` zwraca katalog roboczy programu.
Linia 216: `json.parse()` parsuje JSON.
Linia 217: `json.stringify()` serializuje JSON.
Linia 218: `ui.create()` inicjuje kontekst UI zgodnie z aktualnym runtime.
Linia 219: `ui.open_window()` otwiera okno.
Linia 220: `ui.rect()` rysuje prostokąt.
Linia 221: `ui.label()` rysuje tekst.
Linia 222: `ui.image()` rysuje obraz.
Linia 223: `ui.sprite()` rysuje sprite.
Linia 224: `ui.begin_frame()` rozpoczyna ramkę UI.
Linia 225: `ui.button()` obsługuje przycisk.
Linia 226: `ui.checkbox()` obsługuje checkbox.
Linia 227: `ui.is_key_pressed()` sprawdza klawisz.
Linia 228: `ui.clear()` czyści kontekst UI.
Linia 229: `engine.load_scene()` ładuje scenę według aktualnego kontraktu.
Linia 230: `audio.play_sound()` odtwarza dźwięk.
Linia 231: `audio.play_music()` odtwarza muzykę.
Linia 232: AI nie ma dokumentować `ui.slider()`, jeżeli nie istnieje w rejestrze natywnym.
Linia 233: AI nie ma dokumentować `engine.run()`, jeżeli nie istnieje w rejestrze natywnym.
Linia 234: AI nie ma dokumentować `audio.stop_all()`, jeżeli nie istnieje w rejestrze natywnym.
Linia 235: AI ma sprawdzać `src/stdlib.rs` przed dodaniem nowej funkcji standardowej.
Linia 236: AI ma aktualizować testy po zmianie stdlib.
Linia 237: AI ma aktualizować dokumentację po zmianie stdlib.
Linia 238: AI ma pamiętać, że błędy plików powinny zawierać ścieżkę.
Linia 239: AI ma pamiętać, że `fs.write_json()` zapisuje pretty JSON z końcowym znakiem nowej linii.
Linia 240: AI ma pamiętać, że `fs.list_dir()` sortuje nazwy.

## 9. CLI
Linia 241: `vice run <path>` uruchamia plik albo projekt.
Linia 242: `vice run <path> -- arg1 arg2` przekazuje argumenty do programu.
Linia 243: `vice check <path>` sprawdza program bez uruchamiania.
Linia 244: `vice check --json <path>` zwraca diagnostykę dla narzędzi.
Linia 245: `vice test <path>` uruchamia funkcje `@test`.
Linia 246: `vice test --json <path>` zwraca wynik testów jako JSON.
Linia 247: `vice new <name>` tworzy projekt.
Linia 248: `vice fmt <path>` formatuje kod.
Linia 249: `vice fmt --check <path>` sprawdza format.
Linia 250: `vice fmt --stdin` formatuje źródło z wejścia standardowego.
Linia 251: `vice doctor` diagnozuje środowisko.
Linia 252: `vice ui-check` sprawdza zależności UI.
Linia 253: `vice --version` pokazuje wersję.
Linia 254: AI ma nie pisać opcji CLI bez sprawdzenia parsera CLI.
Linia 255: AI ma aktualizować testy CLI przy zmianie kontraktu CLI.
Linia 256: AI ma rozróżniać komunikat dla człowieka i JSON dla narzędzi.
Linia 257: JSON CLI powinien być stabilny dla rozszerzenia VS Code.
Linia 258: `vice test` kompiluje i wykonuje funkcje testowe.
Linia 259: Testy `@test` powinny być izolowane i deterministyczne.
Linia 260: Argumenty procesu są lokalne dla uruchomienia programu.
Linia 261: Katalog roboczy procesu jest widoczny przez `process.cwd()`.
Linia 262: AI powinno traktować CLI jako podstawowe doświadczenie użytkownika.
Linia 263: AI ma unikać funkcji CLI wymagających konfiguracji globalnej bez potrzeby.
Linia 264: AI ma dbać o Windows, bo workspace użytkownika jest na Windows.
Linia 265: AI ma dbać o przenośność ścieżek.
Linia 266: AI ma używać `PathBuf` i `Path` w Rust, gdzie to możliwe.
Linia 267: AI ma testować ścieżki z katalogami tymczasowymi.
Linia 268: AI ma pilnować, aby błędy CLI kończyły się kodem niepowodzenia.
Linia 269: AI ma pilnować, aby sukces CLI kończył się kodem sukcesu.
Linia 270: AI ma pilnować, aby `--json` nie mieszał losowych tekstów z JSON.

## 10. VS Code extension candidate
Linia 271: Katalog `vscode/candidate/vice-language-0.2.0` zawiera oficjalny kandydat rozszerzenia v0.2.
Linia 272: Rozszerzenie używa `vice check --json` do diagnostyki.
Linia 273: Rozszerzenie używa `vice fmt --stdin` do formatowania.
Linia 274: Rozszerzenie pozwala uruchomić bieżący plik przez CLI.
Linia 275: Rozszerzenie ma ustawienie ścieżki do `vice`.
Linia 276: Rozszerzenie ma ustawienie sprawdzania przy zapisie.
Linia 277: Diagnostyka powinna czyścić stare błędy po zmianie pliku.
Linia 278: Diagnostyka powinna ignorować spóźnione wyniki starszych uruchomień.
Linia 279: Uruchomienie pliku w rozszerzeniu nie powinno używać powłoki, jeśli da się użyć procesu.
Linia 280: Wyjście procesu powinno mieć limit.
Linia 281: Timeout procesu powinien bronić edytor przed zawieszeniem.
Linia 282: Grammar składni powinna obejmować komentarze `#` i `//`.
Linia 283: Grammar składni powinna obejmować `mut`.
Linia 284: Grammar składni powinna obejmować `input -> nazwa;`.
Linia 285: Grammar składni powinna obejmować `error`.
Linia 286: Grammar składni nie powinna promować `null`.
Linia 287: Snippety powinny używać nawiasów w `if`.
Linia 288: Snippety powinny używać nawiasów w `while`.
Linia 289: Snippety powinny mieć przykład `@test`.
Linia 290: Snippety powinny mieć przykład `process.args()`.

## 11. Web i forum
Linia 291: `Web` jest stroną społecznościową i dokumentacyjną, nie runtime języka.
Linia 292: Strona ma służyć do pobierania, nauki, forum, profili, logowania i panelu operacyjnego.
Linia 293: Forum używa Firebase Auth.
Linia 294: Forum używa Firestore.
Linia 295: Forum używa Storage dla zasobów takich jak awatary.
Linia 296: Konfiguracja Firebase jest w `Web/js/firebase-config.js`.
Linia 297: Logika bezpieczeństwa klienta jest pomocnicza.
Linia 298: Autorytatywne bezpieczeństwo jest w regułach Firestore i Storage.
Linia 299: Klient nie może być jedyną granicą uprawnień.
Linia 300: CSP ma ograniczać źródła skryptów i zasobów.
Linia 301: Dane użytkownika wyświetlane w DOM muszą być escapowane albo ustawiane przez `textContent`.
Linia 302: `innerHTML` wymaga ścisłej kontroli.
Linia 303: Formularze muszą walidować długości pól.
Linia 304: Formularze muszą walidować dozwolone znaki tam, gdzie ma to sens.
Linia 305: Nazwy użytkowników muszą być rezerwowane przed konfliktem.
Linia 306: Raport posta powinien mieć deterministyczny identyfikator `postId_uid`.
Linia 307: Użytkownik nie powinien móc spamować raportu tego samego posta.
Linia 308: Admin nie powinien móc zbanować ownera.
Linia 309: Admin nie powinien móc zbanować admina, jeśli reguły tak chronią hierarchię.
Linia 310: Avatar powinien być ograniczony do właściciela profilu.
Linia 311: Reguły Storage powinny sprawdzać ścieżkę i typ zasobu.
Linia 312: Reguły Firestore powinny sprawdzać role i pola.
Linia 313: Moderator może mieć inne uprawnienia niż admin.
Linia 314: Panel operacyjny powinien być ukryty przed zwykłym użytkownikiem.
Linia 315: Ukrycie panelu w UI nie jest zabezpieczeniem.
Linia 316: Reguły backendowe decydują o prawie zapisu.
Linia 317: Web powinien działać bez build step, jeśli taka jest architektura katalogu.
Linia 318: Testy Web powinny sprawdzać walidację i helpery bezpieczeństwa.
Linia 319: Testy reguł powinny sprawdzać role i odmowy.
Linia 320: AI ma nie przenosić sekretów Firebase do publicznego kodu.
Linia 321: Publiczny config Firebase nie jest sekretem, ale reguły muszą być poprawne.
Linia 322: Web nie powinien odsyłać do usuniętych README.
Linia 323: Web powinien wskazywać trzy kanoniczne dokumentacje.
Linia 324: Dokumentacja na stronie powinna być czytelna, skanowalna i zgodna z CLI.
Linia 325: Strona docs powinna odróżniać ścieżkę nauki od referencji developera.
Linia 326: Strona docs powinna mieć sekcję dla AI i automatyzacji.
Linia 327: Forum powinno mieć zgłaszanie, sortowanie, filtrowanie i stany pustej listy.
Linia 328: Profil powinien chronić pola użytkownika.
Linia 329: Logowanie powinno mieć komunikaty błędów bez ujawniania zbyt wielu szczegółów.
Linia 330: Rejestracja powinna wymagać akceptowalnej nazwy użytkownika.

## 12. Bezpieczeństwo projektu
Linia 331: Najważniejsza zasada bezpieczeństwa to minimalny zakres zaufania.
Linia 332: Kod klienta Web jest niezaufany.
Linia 333: Dane z Firestore są niezaufane przed walidacją wyświetlania.
Linia 334: Dane z formularzy są niezaufane przed walidacją.
Linia 335: Pliki użytkownika są niezaufane przed walidacją typu i rozmiaru.
Linia 336: Komunikaty błędów nie powinny ujawniać sekretów.
Linia 337: CLI nie powinno wykonywać niejawnych poleceń systemowych na podstawie kodu Vice.
Linia 338: Runtime bez FFI zmniejsza powierzchnię ataku.
Linia 339: Brak sieci w runtime zmniejsza powierzchnię ataku.
Linia 340: Operacje plikowe nadal wymagają ostrożnych komunikatów i testów.
Linia 341: `env.set()` ma efekt uboczny w procesie.
Linia 342: `fs.write_text()` i `fs.write_json()` modyfikują system plików.
Linia 343: Testy operacji plikowych powinny używać katalogów tymczasowych.
Linia 344: AI nie powinno używać katalogu domowego użytkownika do testów destrukcyjnych.
Linia 345: AI nie powinno kasować plików poza zakresem testu.
Linia 346: Web powinien mieć nagłówki bezpieczeństwa w `firebase.json`.
Linia 347: Web powinien mieć CSP zgodne z używanymi CDN.
Linia 348: Web powinien mieć `X-Content-Type-Options`.
Linia 349: Web powinien mieć `Referrer-Policy`.
Linia 350: Web powinien mieć ograniczenia ramek, jeżeli nie ma potrzeby osadzania.
Linia 351: Web powinien mieć testy XSS dla helperów renderowania.
Linia 352: Reguły Firestore powinny odmawiać nieznanych pól.
Linia 353: Reguły Firestore powinny odmawiać eskalacji roli przez klienta.
Linia 354: Reguły Firestore powinny odmawiać modyfikacji cudzych profili.
Linia 355: Reguły Firestore powinny pozwalać ownerowi na kontrolowane operacje administracyjne.
Linia 356: Reguły Storage powinny odmawiać cudzych uploadów.
Linia 357: Reguły Storage powinny ograniczać rozmiar i typ pliku.
Linia 358: AI ma nie wrzucać kluczy prywatnych do repozytorium.
Linia 359: AI ma sprawdzać `.gitignore` przed dodaniem plików środowiskowych.
Linia 360: AI ma nie commitować logów emulatora, jeśli są generowane lokalnie.

## 13. Procedura zmian języka
Linia 361: Zmiana składni zaczyna się od parsera i AST.
Linia 362: Po parserze aktualizuj semantykę.
Linia 363: Po semantyce aktualizuj kompilator.
Linia 364: Po kompilatorze aktualizuj VM.
Linia 365: Po VM aktualizuj stdlib, jeśli składnia dotyczy funkcji natywnych.
Linia 366: Po kodzie aktualizuj formatter.
Linia 367: Po formatterze aktualizuj CLI.
Linia 368: Po CLI aktualizuj rozszerzenie VS Code.
Linia 369: Po narzędziach aktualizuj dokumentację.
Linia 370: Po dokumentacji aktualizuj przykłady.
Linia 371: Po przykładach uruchom testy.
Linia 372: Nie zostawiaj funkcji tylko w parserze.
Linia 373: Nie zostawiaj funkcji tylko w stdlib bez dokumentacji.
Linia 374: Nie zostawiaj funkcji tylko w dokumentacji bez implementacji.
Linia 375: Każdy nowy typ błędu powinien mieć czytelny komunikat.
Linia 376: Każda nowa funkcja CLI powinna mieć test.
Linia 377: Każda nowa funkcja stdlib powinna mieć test pozytywny.
Linia 378: Funkcja stdlib z błędami I/O powinna mieć test błędu.
Linia 379: Nowa składnia powinna mieć test parsera.
Linia 380: Nowa reguła typów powinna mieć test semantyki.
Linia 381: Nowa instrukcja bajtkodu powinna mieć test VM.
Linia 382: Nowa funkcja Web powinna mieć test helperów albo reguł.
Linia 383: Nowa funkcja dokumentacji powinna mieć link z `learn.html`.
Linia 384: Zmiana stylu strony powinna nie łamać dostępności klawiatury.
Linia 385: Zmiana formularza powinna zachować etykiety i komunikaty błędów.
Linia 386: Zmiana nawigacji powinna zachować `aria-current`.
Linia 387: Zmiana panelu admina powinna mieć sprawdzenie roli.
Linia 388: Zmiana profilu powinna mieć sprawdzenie właściciela.
Linia 389: Zmiana uploadu powinna mieć reguły Storage.
Linia 390: Zmiana Firestore powinna mieć test odmowy.

## 14. Diagnostyka
Linia 391: Diagnostyka powinna mieć lokalizację w pliku.
Linia 392: Diagnostyka powinna mieć komunikat zrozumiały dla osoby uczącej się.
Linia 393: Diagnostyka dla narzędzi powinna być serializowalna do JSON.
Linia 394: Diagnostyka nie powinna zasypywać użytkownika skutkami jednego błędu.
Linia 395: Diagnostyka powinna proponować naprawę, gdy jest oczywista.
Linia 396: Błąd typu powinien pokazywać oczekiwany i otrzymany typ.
Linia 397: Błąd funkcji powinien pokazywać nazwę funkcji.
Linia 398: Błąd argumentów powinien pokazywać oczekiwaną liczbę argumentów.
Linia 399: Błąd pola danych powinien pokazywać nazwę pola.
Linia 400: Błąd enuma powinien pokazywać nazwę wariantu.
Linia 401: Błąd traita powinien pokazywać nazwę metody.
Linia 402: Błąd pliku powinien pokazywać ścieżkę.
Linia 403: Błąd JSON powinien rozróżniać parse i stringify.
Linia 404: Błąd runtime powinien być stabilny w testach.
Linia 405: Błąd CLI powinien mieć kod wyjścia.
Linia 406: Błąd Web powinien być bezpieczny dla użytkownika.
Linia 407: Błąd Web nie powinien ujawniać tokenów.
Linia 408: Błąd formularza powinien wskazywać konkretne pole.
Linia 409: Błąd reguł powinien być sprawdzany testami emulatora.
Linia 410: AI ma nie poprawiać diagnostyki kosztem utraty informacji technicznej.

## 15. Przykłady generowane przez AI
Linia 411: Przykład powinien być kompletny.
Linia 412: Przykład powinien mieć minimalną liczbę założeń.
Linia 413: Przykład dla CLI powinien używać prawdziwych komend.
Linia 414: Przykład dla stdlib powinien używać prawdziwych nazw funkcji.
Linia 415: Przykład testu powinien używać `@test`.
Linia 416: Przykład błędu testu powinien używać `fail <- "opis";`.
Linia 417: Przykład programu powinien mieć `@entry` albo `main`.
Linia 418: Przykład biblioteki może nie mieć wejścia, jeśli służy do `vice check`.
Linia 419: Przykład plikowy powinien działać w katalogu projektu.
Linia 420: Przykład z argumentami powinien używać `process.args()`.
Linia 421: Przykład z katalogiem powinien używać `process.cwd()`.
Linia 422: Przykład JSON powinien używać `json.parse()` i `json.stringify()`.
Linia 423: Przykład ścieżek powinien używać `path.join()`.
Linia 424: Przykład listy plików powinien używać `fs.list_dir()`.
Linia 425: Przykład tworzenia katalogu powinien używać `fs.create_dir()`.
Linia 426: Przykład UI powinien nie obiecywać pełnego frameworka aplikacji.
Linia 427: Przykład audio powinien nie obiecywać miksera.
Linia 428: Przykład engine powinien nie obiecywać starego silnika przeglądarkowego.
Linia 429: Przykład Web powinien nie używać sekretów.
Linia 430: Przykład Firebase powinien mówić o regułach.
Linia 431: AI powinno unikać przykładów z nieistniejącymi importami.
Linia 432: AI powinno unikać przykładów z nieistniejącą składnią lambd.
Linia 433: AI powinno unikać przykładów z nieistniejącymi klasami.
Linia 434: AI powinno unikać przykładów z nieistniejącymi wyjątkami.
Linia 435: AI powinno nie pisać `try/catch`, jeśli nie jest w języku.
Linia 436: AI powinno nie pisać `async/await`, jeśli nie jest w języku.
Linia 437: AI powinno nie pisać `Promise`, jeśli nie jest w języku.
Linia 438: AI powinno nie pisać makr, jeśli nie są zaimplementowane.
Linia 439: AI powinno nie pisać modułów pakietowych bez sprawdzenia manifestu.
Linia 440: AI powinno nie pisać systemu paczek, jeśli nie istnieje.

## 16. Intencje użytkownika i odpowiedzi AI
Linia 441: Gdy użytkownik prosi o naukę Vice, pokaż ścieżkę od CLI do prostych programów.
Linia 442: Gdy użytkownik prosi o przykład, daj kod i komendę uruchomienia.
Linia 443: Gdy użytkownik prosi o błąd, najpierw poproś o diagnostykę albo sprawdź lokalnie.
Linia 444: Gdy użytkownik prosi o dodanie składni, sprawdź pełny pipeline.
Linia 445: Gdy użytkownik prosi o stdlib, sprawdź rejestr natywny.
Linia 446: Gdy użytkownik prosi o Web, sprawdź HTML, CSS, JS, reguły i testy.
Linia 447: Gdy użytkownik prosi o bezpieczeństwo, zacznij od granicy zaufania.
Linia 448: Gdy użytkownik prosi o dokumentację, usuń rozbieżności między docs i kodem.
Linia 449: Gdy użytkownik prosi o commit, nie stageuj `AGENTS.md`, jeśli jest lokalnie zmieniony i nie wchodzi w zakres.
Linia 450: Gdy użytkownik prosi o kontynuację, kontynuuj najbliższy niedomknięty etap.
Linia 451: Gdy użytkownik mówi o folderze `web`, sprawdź, czy w repo jest `Web`.
Linia 452: Gdy użytkownik mówi o wyglądzie strony, oceniaj layout, responsywność i stan pusty.
Linia 453: Gdy użytkownik mówi o forum, oceniaj moderację, raporty i profile.
Linia 454: Gdy użytkownik mówi o dokumentacji ogromnej, sprawdź liczby linii.
Linia 455: Gdy użytkownik mówi o AI docs, pisz reguły jednoznaczne i operacyjne.
Linia 456: Gdy użytkownik mówi o developera docs, pisz decyzje architektoniczne i inventory.
Linia 457: Gdy użytkownik mówi o studentach, pisz lekcje, przykłady i drogę nauki.
Linia 458: Gdy użytkownik chce usunąć dokumentację, usuń stare pliki docs i README z zakresu dokumentacji.
Linia 459: Gdy usuwasz docs, zostaw nie-dokumentacyjne pliki konfiguracyjne.
Linia 460: Gdy zmieniasz docs page, upewnij się, że nie linkuje do usuniętego README.

## 17. Pliki i miejsca do sprawdzania
Linia 461: `src/lib.rs` pokazuje moduły biblioteki.
Linia 462: `src/parser.rs` pokazuje obsługiwaną składnię.
Linia 463: `src/ast.rs` pokazuje strukturę AST.
Linia 464: `src/semantic.rs` pokazuje reguły semantyczne.
Linia 465: `src/compiler.rs` pokazuje generowanie bajtkodu.
Linia 466: `src/vm.rs` pokazuje wykonanie bajtkodu.
Linia 467: `src/value.rs` pokazuje wartości runtime.
Linia 468: `src/stdlib.rs` pokazuje funkcje natywne.
Linia 469: `src/diagnostics.rs` pokazuje format diagnostyki.
Linia 470: `src/formatter.rs` pokazuje formatowanie Vice.
Linia 471: `src/main.rs` pokazuje CLI.
Linia 472: `tests` pokazuje kontrakty funkcjonalne.
Linia 473: `examples` pokazuje przykłady użytkowe.
Linia 474: `vscode` pokazuje integrację edytora.
Linia 475: `Web` pokazuje stronę i forum.
Linia 476: `Web/firestore.rules` pokazuje reguły danych.
Linia 477: `Web/storage.rules` pokazuje reguły plików.
Linia 478: `Web/firebase.json` pokazuje hosting i nagłówki.
Linia 479: `Web/js/security.js` pokazuje helpery bezpieczeństwa klienta.
Linia 480: `Web/tests` pokazuje testy strony i reguł.

## 18. Decyzje, których AI ma bronić
Linia 481: Usunięcie niepodłączonego silnika przeglądarkowego jest świadomym uproszczeniem.
Linia 482: Usunięcie niepodłączonych stubów network i interop jest świadomym ograniczeniem powierzchni.
Linia 483: Rozbudowa CLI jest ważniejsza niż obietnice graficznego edytora.
Linia 484: Rozbudowa stdlib plików jest ważna dla realnych projektów.
Linia 485: `vice test` jest ważne dla użyteczności języka.
Linia 486: `process.args()` jest ważne dla narzędzi CLI pisanych w Vice.
Linia 487: `process.cwd()` jest ważne dla projektów plikowych.
Linia 488: `fs.write_json()` jest ważne dla konfiguracji i danych.
Linia 489: `fs.list_dir()` jest ważne dla skryptów projektowych.
Linia 490: `path.is_file()` i `path.is_dir()` są ważne dla bezpiecznych workflow.
Linia 491: Dokumentacja ma być skonsolidowana, aby uniknąć rozjazdu.
Linia 492: Web docs page ma być mapą po trzech kanonicznych dokumentach.
Linia 493: VS Code extension ma być cienką warstwą nad CLI.
Linia 494: Firestore rules mają być prawdziwą granicą uprawnień.
Linia 495: Storage rules mają chronić uploady.
Linia 496: CSP ma ograniczać szkody po błędzie frontendu.
Linia 497: Formatter ma zachować komentarze, bo komentarze są częścią nauki.
Linia 498: Komentarze `#` pasują do stylu języka przyjaznego nauce.
Linia 499: Brak `null` zmniejsza niejawne stany.
Linia 500: Brak FFI w tym etapie upraszcza bezpieczeństwo.

## 19. Checklist przed odpowiedzią AI
Linia 501: Sprawdź, czy odpowiedź dotyczy aktualnego stanu repozytorium.
Linia 502: Sprawdź, czy nazwy komend są prawdziwe.
Linia 503: Sprawdź, czy nazwy funkcji stdlib są prawdziwe.
Linia 504: Sprawdź, czy przykład ma poprawną składnię.
Linia 505: Sprawdź, czy przykład ma `@entry` albo `main`, jeśli ma być uruchomiony.
Linia 506: Sprawdź, czy test używa `@test`.
Linia 507: Sprawdź, czy porada Web nie omija reguł bezpieczeństwa.
Linia 508: Sprawdź, czy porada Firebase nie traktuje klienta jako zaufanego.
Linia 509: Sprawdź, czy dokumentacja nie linkuje do usuniętych README.
Linia 510: Sprawdź, czy usunięte stuby nie są opisane jako gotowe funkcje.
Linia 511: Sprawdź, czy `AGENTS.md` nie został przypadkowo dodany do commita.
Linia 512: Sprawdź, czy testy przeszły albo jasno opisz, czego nie uruchomiono.
Linia 513: Sprawdź, czy zmiany są w jednym logicznym zakresie.
Linia 514: Sprawdź, czy finalna odpowiedź zawiera ścieżki plików, gdy to pomaga.
Linia 515: Sprawdź, czy finalna odpowiedź nie jest przesadnie długa.
Linia 516: Sprawdź, czy użytkownik dostał konkretny status.
Linia 517: Sprawdź, czy commit ma sensowny tytuł.
Linia 518: Sprawdź, czy nie ma niepotrzebnych plików tymczasowych.
Linia 519: Sprawdź, czy Web nie ma references do README.
Linia 520: Sprawdź, czy trzy dokumentacje mają ponad 700 linii.

## 20. Reguły generowania kodu Vice dla AI
Linia 521: Reguła 001: Używaj `let nazwa => wyrazenie;` do niemutowalnych deklaracji.
Linia 522: Reguła 002: Używaj `mut nazwa => wyrazenie;`, gdy wartość ma się zmieniać.
Linia 523: Reguła 003: Używaj `nazwa: typ` w parametrach funkcji.
Linia 524: Reguła 004: Używaj `-> typ` w wyniku funkcji.
Linia 525: Reguła 005: Używaj `@entry` dla przykładu aplikacyjnego.
Linia 526: Reguła 006: Używaj `@test` dla przykładu testowego.
Linia 527: Reguła 007: Używaj `fail <- "opis";` do nieudanego testu.
Linia 528: Reguła 008: Używaj `process.args()` dla argumentów CLI.
Linia 529: Reguła 009: Używaj `process.cwd()` dla katalogu roboczego.
Linia 530: Reguła 010: Używaj `path.join()` dla ścieżek przenośnych.
Linia 531: Reguła 011: Używaj `fs.read_text()` do czytania tekstu.
Linia 532: Reguła 012: Używaj `fs.write_text()` do zapisu tekstu.
Linia 533: Reguła 013: Używaj `fs.append_text()` do dopisywania tekstu.
Linia 534: Reguła 014: Używaj `fs.read_json()` do czytania JSON.
Linia 535: Reguła 015: Używaj `fs.write_json()` do zapisu JSON.
Linia 536: Reguła 016: Używaj `fs.list_dir()` do listowania katalogu.
Linia 537: Reguła 017: Używaj `fs.create_dir()` do tworzenia katalogu.
Linia 538: Reguła 018: Używaj `path.exists()` przed zależnością od pliku.
Linia 539: Reguła 019: Używaj `path.is_file()` dla pliku.
Linia 540: Reguła 020: Używaj `path.is_dir()` dla katalogu.
Linia 541: Reguła 021: Używaj `string.trim()` dla wejścia tekstowego.
Linia 542: Reguła 022: Używaj `string.lower()` dla porównań bez wielkości liter.
Linia 543: Reguła 023: Używaj `string.upper()` tylko gdy wynik ma być prezentacyjny.
Linia 544: Reguła 024: Używaj `string.contains()` dla fragmentu.
Linia 545: Reguła 025: Używaj `string.starts_with()` dla prefiksu.
Linia 546: Reguła 026: Używaj `string.ends_with()` dla sufiksu.
Linia 547: Reguła 027: Używaj `array.len()` do długości tablicy.
Linia 548: Reguła 028: Używaj `array.first()` tylko po rozważeniu pustej tablicy.
Linia 549: Reguła 029: Używaj `array.last()` tylko po rozważeniu pustej tablicy.
Linia 550: Reguła 030: Używaj `array.join()` dla czytelnego tekstu.
Linia 551: Reguła 031: Używaj `json.parse()` tylko na danych zaufanych albo obsłuż błąd.
Linia 552: Reguła 032: Używaj `json.stringify()` do serializacji danych.
Linia 553: Reguła 033: Używaj `env.get()` oszczędnie.
Linia 554: Reguła 034: Używaj `env.set()` świadomie, bo zmienia proces.
Linia 555: Reguła 035: Używaj `math.min()` i `math.max()` zamiast ręcznych gałęzi dla prostych limitów.
Linia 556: Reguła 036: Używaj `math.sqrt()` tylko dla sensownego zakresu wartości.
Linia 557: Reguła 037: Używaj `random.int()` dla liczby całkowitej.
Linia 558: Reguła 038: Używaj `random.float()` dla liczby zmiennoprzecinkowej.
Linia 559: Reguła 039: Używaj `time.now_ms()` do prostego pomiaru czasu.
Linia 560: Reguła 040: Nie używaj nieistniejącego `net.fetch()`.
Linia 561: Reguła 041: Nie używaj nieistniejącego `ffi.call()`.
Linia 562: Reguła 042: Nie używaj nieistniejącego `ui.slider()`.
Linia 563: Reguła 043: Nie używaj nieistniejącego `engine.run()`.
Linia 564: Reguła 044: Nie używaj nieistniejącego `audio.stop_all()`.
Linia 565: Reguła 045: Nie używaj `null`.
Linia 566: Reguła 046: Nie używaj `try/catch`.
Linia 567: Reguła 047: Nie używaj `async/await`.
Linia 568: Reguła 048: Nie używaj klas.
Linia 569: Reguła 049: Nie używaj dziedziczenia klas.
Linia 570: Reguła 050: Nie używaj importów bez sprawdzenia `use`.
Linia 571: Reguła 051: Nie używaj package managera Vice, jeśli nie istnieje.
Linia 572: Reguła 052: Nie używaj ukrytych globali.
Linia 573: Reguła 053: Nie używaj efektów ubocznych w przykładach początkujących bez komentarza.
Linia 574: Reguła 054: Nie używaj nazw jednoliterowych poza licznikami.
Linia 575: Reguła 055: Nie używaj wielkich bloków bez podziału.
Linia 576: Reguła 056: Nie używaj magicznych stringów bez stałej opisowej.
Linia 577: Reguła 057: Nie używaj złożonego `match`, gdy wystarczy proste `if`.
Linia 578: Reguła 058: Nie używaj `if`, gdy enum wymaga pełnego `match`.
Linia 579: Reguła 059: Nie ignoruj wyniku operacji plikowej.
Linia 580: Reguła 060: Nie ignoruj błędów JSON.
Linia 581: Reguła 061: Dokumentuj komendę `vice run` przy przykładzie aplikacji.
Linia 582: Reguła 062: Dokumentuj komendę `vice test` przy przykładzie testu.
Linia 583: Reguła 063: Dokumentuj `vice check` przy przykładzie biblioteki.
Linia 584: Reguła 064: Dokumentuj `vice fmt` przy stylu kodu.
Linia 585: Reguła 065: Dokumentuj `vice doctor` przy problemach środowiska.
Linia 586: Reguła 066: Dokumentuj `vice ui-check` przy UI.
Linia 587: Reguła 067: Utrzymuj zgodność przykładów z formatterem.
Linia 588: Reguła 068: Utrzymuj zgodność nazw z testami.
Linia 589: Reguła 069: Utrzymuj zgodność docs ze stdlib.
Linia 590: Reguła 070: Utrzymuj zgodność Web docs z trzema plikami.
Linia 591: Reguła 071: Sprawdzaj `src/stdlib.rs` przed odpowiedzią o funkcji natywnej.
Linia 592: Reguła 072: Sprawdzaj `src/main.rs` przed odpowiedzią o CLI.
Linia 593: Reguła 073: Sprawdzaj `src/parser.rs` przed odpowiedzią o składni.
Linia 594: Reguła 074: Sprawdzaj `src/semantic.rs` przed odpowiedzią o typach.
Linia 595: Reguła 075: Sprawdzaj `src/vm.rs` przed odpowiedzią o runtime.
Linia 596: Reguła 076: Sprawdzaj `Web/firestore.rules` przed odpowiedzią o rolach.
Linia 597: Reguła 077: Sprawdzaj `Web/storage.rules` przed odpowiedzią o uploadach.
Linia 598: Reguła 078: Sprawdzaj `Web/firebase.json` przed odpowiedzią o nagłówkach.
Linia 599: Reguła 079: Sprawdzaj `Web/js/security.js` przed odpowiedzią o sanitizacji.
Linia 600: Reguła 080: Sprawdzaj `vscode/candidate` przed odpowiedzią o rozszerzeniu.
Linia 601: Reguła 081: Jeżeli zmieniasz parser, zmień testy parsera.
Linia 602: Reguła 082: Jeżeli zmieniasz semantykę, zmień testy semantyki.
Linia 603: Reguła 083: Jeżeli zmieniasz VM, zmień testy VM.
Linia 604: Reguła 084: Jeżeli zmieniasz stdlib, zmień testy stdlib.
Linia 605: Reguła 085: Jeżeli zmieniasz CLI, zmień testy CLI.
Linia 606: Reguła 086: Jeżeli zmieniasz Web, zmień testy Web.
Linia 607: Reguła 087: Jeżeli zmieniasz reguły, zmień testy reguł.
Linia 608: Reguła 088: Jeżeli zmieniasz dokumentację, sprawdź liczby linii.
Linia 609: Reguła 089: Jeżeli zmieniasz przykłady, uruchom `vice check` lub testy.
Linia 610: Reguła 090: Jeżeli zmieniasz VS Code, sprawdź JSON package i składnię JS.
Linia 611: Reguła 091: Preferuj małe funkcje w przykładach.
Linia 612: Reguła 092: Preferuj jawne nazwy pól.
Linia 613: Reguła 093: Preferuj testy nad opisem ręcznym.
Linia 614: Reguła 094: Preferuj stabilne formaty JSON dla narzędzi.
Linia 615: Reguła 095: Preferuj przenośne ścieżki.
Linia 616: Reguła 096: Preferuj komunikaty błędów z kontekstem.
Linia 617: Reguła 097: Preferuj brak efektów ubocznych w dokumentacji początkującej.
Linia 618: Reguła 098: Preferuj explicit `@entry` w tutorialach.
Linia 619: Reguła 099: Preferuj `@test` w pokazach jakości.
Linia 620: Reguła 100: Preferuj opis ograniczeń zamiast obietnic.

## 21. Reguły odpowiedzi końcowej dla AI
Linia 621: Odpowiedź końcowa ma zaczynać się od wyniku.
Linia 622: Odpowiedź końcowa ma wymienić najważniejsze pliki.
Linia 623: Odpowiedź końcowa ma wymienić testy.
Linia 624: Odpowiedź końcowa ma wymienić commit, jeśli został utworzony.
Linia 625: Odpowiedź końcowa ma wskazać ograniczenia, jeśli coś nie zostało zrobione.
Linia 626: Odpowiedź końcowa nie ma powtarzać całej dokumentacji.
Linia 627: Odpowiedź końcowa nie ma ukrywać błędów testów.
Linia 628: Odpowiedź końcowa nie ma twierdzić, że push wykonano, jeśli nie wykonano.
Linia 629: Odpowiedź końcowa nie ma twierdzić, że PR utworzono, jeśli nie utworzono.
Linia 630: Odpowiedź końcowa nie ma wymagać czytania komentarzy pośrednich.
Linia 631: Jeśli commit powstał, AI ma użyć właściwej dyrektywy aplikacji.
Linia 632: Jeśli pliki zostały staged, AI ma użyć właściwej dyrektywy aplikacji.
Linia 633: Jeśli branch powstał, AI ma użyć właściwej dyrektywy aplikacji.
Linia 634: Jeśli push powstał, AI ma użyć właściwej dyrektywy aplikacji.
Linia 635: Jeśli PR powstał, AI ma użyć właściwej dyrektywy aplikacji.
Linia 636: Nie emituj dyrektywy dla akcji, której nie wykonano.
Linia 637: Nie emituj ścieżek względnych w linkach do plików, gdy aplikacja oczekuje absolutnych.
Linia 638: Link do pliku powinien mieć postać markdown z absolutną ścieżką.
Linia 639: Nie linkuj do nieistniejących plików.
Linia 640: Nie ukrywaj, że `AGENTS.md` został pominięty, jeśli użytkownik pyta o status git.

## 22. Mapowanie ryzyka
Linia 641: Ryzyko parsera: składnia przyjęta bez semantyki.
Linia 642: Ryzyko semantyki: typ uznany błędnie za zgodny.
Linia 643: Ryzyko kompilatora: AST skompilowane do złego bajtkodu.
Linia 644: Ryzyko VM: instrukcja wykonana z niepoprawnym stosem.
Linia 645: Ryzyko stdlib: funkcja natywna przyjmuje złe typy.
Linia 646: Ryzyko CLI: narzędzie zwraca niestabilny output.
Linia 647: Ryzyko formattera: komentarze albo URL-e są niszczone.
Linia 648: Ryzyko VS Code: diagnostyka jest opóźniona i myląca.
Linia 649: Ryzyko Web: UI ufa klientowi.
Linia 650: Ryzyko Firestore: reguły pozwalają na eskalację.
Linia 651: Ryzyko Storage: upload cudzych plików.
Linia 652: Ryzyko CSP: zbyt szerokie źródła skryptów.
Linia 653: Ryzyko docs: dokumentacja obiecuje nieistniejące API.
Linia 654: Ryzyko examples: przykład nie przechodzi `vice check`.
Linia 655: Ryzyko testów: testy są zbyt płytkie.
Linia 656: Ryzyko commitów: stage obejmuje pliki użytkownika.
Linia 657: Ryzyko usuwania: kasowanie dokumentacji technicznej bez zastąpienia treścią.
Linia 658: Ryzyko migracji: strona linkuje do usuniętego pliku.
Linia 659: Ryzyko bezpieczeństwa: klient ukrywa przycisk, ale reguły pozwalają na zapis.
Linia 660: Ryzyko runtime: nowa funkcja ma efekt uboczny bez testu.

## 23. Minimalny zestaw użyteczności języka
Linia 661: Vice jest użyteczny, gdy można utworzyć projekt.
Linia 662: Vice jest użyteczny, gdy można uruchomić projekt.
Linia 663: Vice jest użyteczny, gdy można sprawdzić projekt bez uruchamiania.
Linia 664: Vice jest użyteczny, gdy można formatować kod.
Linia 665: Vice jest użyteczny, gdy można pisać testy natywne w języku.
Linia 666: Vice jest użyteczny, gdy można czytać argumenty CLI.
Linia 667: Vice jest użyteczny, gdy można czytać katalog roboczy.
Linia 668: Vice jest użyteczny, gdy można czytać i zapisywać pliki tekstowe.
Linia 669: Vice jest użyteczny, gdy można czytać i zapisywać JSON.
Linia 670: Vice jest użyteczny, gdy można listować katalogi.
Linia 671: Vice jest użyteczny, gdy można tworzyć katalogi.
Linia 672: Vice jest użyteczny, gdy można pracować ze ścieżkami przenośnie.
Linia 673: Vice jest użyteczny, gdy można testować funkcje bez zewnętrznego harnessu.
Linia 674: Vice jest użyteczny, gdy edytor pokazuje diagnostykę.
Linia 675: Vice jest użyteczny, gdy formatter działa także dla niezapisanych plików.
Linia 676: Vice jest użyteczny, gdy dokumentacja uczy realnego workflow.
Linia 677: Vice jest użyteczny, gdy strona prowadzi do instalacji i nauki.
Linia 678: Vice jest użyteczny, gdy forum ma podstawy moderacji.
Linia 679: Vice jest użyteczny, gdy zabezpieczenia nie są tylko w UI.
Linia 680: Vice jest użyteczny, gdy ograniczenia są jawnie opisane.

## 24. Zakazane halucynacje
Linia 681: Nie twierdź, że Vice ma kompilację do WebAssembly, jeśli nie została dodana.
Linia 682: Nie twierdź, że Vice ma backend HTTP, jeśli nie został dodany.
Linia 683: Nie twierdź, że Vice ma package registry, jeśli nie został dodany.
Linia 684: Nie twierdź, że Vice ma LSP server, jeśli nie został dodany.
Linia 685: Nie twierdź, że Vice ma debugger, jeśli nie został dodany.
Linia 686: Nie twierdź, że Vice ma REPL, jeśli nie został dodany.
Linia 687: Nie twierdź, że Vice ma browser engine, bo został usunięty.
Linia 688: Nie twierdź, że Vice ma sieć w runtime, bo usunięto niepodłączone stuby.
Linia 689: Nie twierdź, że Vice ma FFI w runtime, bo usunięto niepodłączone stuby.
Linia 690: Nie twierdź, że Web admin panel sam zabezpiecza dane.
Linia 691: Nie twierdź, że CSP rozwiązuje wszystkie problemy XSS.
Linia 692: Nie twierdź, że Firebase config jest sekretem.
Linia 693: Nie twierdź, że publiczny frontend może ukryć uprawnienia.
Linia 694: Nie twierdź, że dokumentacja zastępuje testy.
Linia 695: Nie twierdź, że test ręczny zastępuje test automatyczny dla reguł.
Linia 696: Nie twierdź, że `cargo check` zastępuje testy runtime.
Linia 697: Nie twierdź, że `cargo fmt` sprawdza zachowanie.
Linia 698: Nie twierdź, że usunięcie README oznacza brak dokumentacji.
Linia 699: Nie twierdź, że `AGENTS.md` jest publiczną dokumentacją Vice.
Linia 700: Nie twierdź, że AI może zmieniać zakres zadania bez zgody użytkownika.

## 25. Końcowy protokół kontroli
Linia 701: Krok 01: Uruchom `git status --short`.
Linia 702: Krok 02: Sprawdź, czy stare dokumentacje są usunięte.
Linia 703: Krok 03: Sprawdź, czy istnieje dokumentacja studencka.
Linia 704: Krok 04: Sprawdź, czy istnieje dokumentacja developerska.
Linia 705: Krok 05: Sprawdź, czy istnieje dokumentacja AI.
Linia 706: Krok 06: Sprawdź, czy każdy z trzech plików ma ponad 700 linii.
Linia 707: Krok 07: Sprawdź, czy Web docs page nie odsyła do README.
Linia 708: Krok 08: Sprawdź, czy Web docs page opisuje trzy ścieżki czytania.
Linia 709: Krok 09: Sprawdź, czy Web docs page ma sekcję CLI.
Linia 710: Krok 10: Sprawdź, czy Web docs page ma sekcję bezpieczeństwa.
Linia 711: Krok 11: Sprawdź, czy Web docs page ma sekcję dla AI.
Linia 712: Krok 12: Sprawdź, czy konfiguracja strony nie wyklucza nowych zasobów.
Linia 713: Krok 13: Uruchom testy Rust, jeśli kod Rust zmieniono.
Linia 714: Krok 14: Uruchom testy Web, jeśli Web zmieniono.
Linia 715: Krok 15: Uruchom testy reguł, jeśli reguły zmieniono.
Linia 716: Krok 16: Nie stageuj `AGENTS.md`.
Linia 717: Krok 17: Stageuj tylko pliki z zakresu.
Linia 718: Krok 18: Zrób commit o jasnym tytule.
Linia 719: Krok 19: Podaj hash commita w odpowiedzi.
Linia 720: Krok 20: Podaj testy w odpowiedzi.
Linia 721: Krok 21: Podaj ewentualne ograniczenia w odpowiedzi.
Linia 722: Krok 22: Jeżeli test nie działa przez brak emulatora, napisz to wprost.
Linia 723: Krok 23: Jeżeli test nie działa przez brak zależności, napisz to wprost.
Linia 724: Krok 24: Jeżeli wszystko przeszło, napisz to krótko.
Linia 725: Krok 25: Nie rozpisuj się ponad potrzebę.

## 26. Szybka matryca odpowiedzialności
Linia 726: Parser odpowiada za rozpoznanie formy tekstu.
Linia 727: AST odpowiada za strukturę programu.
Linia 728: Semantyka odpowiada za sens programu.
Linia 729: Kompilator odpowiada za bajtkod.
Linia 730: VM odpowiada za wykonanie.
Linia 731: Value odpowiada za reprezentację danych.
Linia 732: Stdlib odpowiada za funkcje natywne.
Linia 733: CLI odpowiada za doświadczenie terminala.
Linia 734: Formatter odpowiada za styl kodu.
Linia 735: Diagnostyka odpowiada za komunikację błędów.
Linia 736: VS Code extension odpowiada za integrację edytora.
Linia 737: Web odpowiada za stronę i społeczność.
Linia 738: Firestore rules odpowiadają za dostęp do danych.
Linia 739: Storage rules odpowiadają za dostęp do plików.
Linia 740: Firebase hosting odpowiada za nagłówki i routing.
Linia 741: Testy odpowiadają za utrzymanie kontraktów.
Linia 742: Dokumentacja odpowiada za prawdę użytkową.
Linia 743: AI odpowiada za spójność zmian.
Linia 744: Użytkownik decyduje o kierunku produktu.
Linia 745: Repozytorium jest źródłem prawdy technicznej.
Linia 746: Dokumentacja jest źródłem prawdy operacyjnej po zgodności z kodem.
Linia 747: Strona jest źródłem prawdy publicznej dla nowych użytkowników.
Linia 748: Forum jest źródłem kontaktu społecznościowego.
Linia 749: Bezpieczeństwo jest warstwą reguł, kodu i procesu.
Linia 750: Vice 0.2.0 jest projektem rozwijanym konsekwentnie, bez obiecywania nieistniejących warstw.

