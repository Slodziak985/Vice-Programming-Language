# Vice 0.2.0 — podręcznik developera

> Kanoniczny opis implementacji, decyzji, testów i bezpieczeństwa.

## Stan i zakres

Crate używa Rust 2021 i ma wersję 0.2.0.
Domyślną binarką oraz default-run jest `vice`.
Domyślny feature to `core`.
Opcjonalne features to `desktop`, `installer-ui` i `audio`.
Desktop używa ab_glyph 0.2.32, bytemuck 1.25.0, image 0.25.10, pollster 0.4.0, wgpu 28.0.0, wgpu_glyph 0.28.0 i winit 0.30.12.
Installer używa directories 6.0.0, eframe 0.34.3 i egui_extras 0.34.3.
Audio używa rodio 0.22.2.
Core używa once_cell 1.21.0, serde_json 1.0.150, libloading 0.8, ureq 2.10, ring 0.17 oraz base64 0.22.
Build dependency winres 0.1.12 osadza zasoby Windows.
Ekosystem języka posiada natywne moduły: `http` (REST/AI), `ffi` (dynamiczne ładowanie bibliotek C-ABI .dll/.so), `matrix` (obliczenia wektorowo-macierzowe w stylu NumPy) oraz `crypto` (SHA256, Base64).
Publiczny runtime jest wyłącznie zweryfikowaną maszyną wirtualną.

## Pipeline

1. Loader czyta entry i rekurencyjne importy `use` oraz legacy `@mount`.
2. Loader wykrywa cykle i duplikaty symboli.
3. Lexer tworzy tokeny ze spanami.
4. Parser buduje AST.
5. SemanticAnalyzer rozwiązuje nazwy, typy, traits, efekty i przepływ.
6. NativeRegistry waliduje builtin calls.
7. Compiler buduje Chunk każdej funkcji.
8. Verifier sprawdza CompiledProgram.
9. VM resetuje stan i wykonuje wybrane entry.
10. ExecutionResult zwraca value i bufor wyjścia `emit`/legacy `output`.
11. CLI renderuje błąd z fragmentem albo JSON.

## Kontrakt publiczny v0.2

Nowy manifest projektu to `vice.toml`.
Loader najpierw szuka `vice.toml`, a dopiero potem legacy `vice.json`.
Minimalny manifest ma sekcje `[package]`, `[paths]` i `[features]`.
Pole `[package].entry` jest sprawdzane tak samo rygorystycznie jak legacy `entry`: nie może być absolutne ani wychodzić poza katalog projektu.
`vice new` i `vice init` generują wyłącznie `vice.toml`.
`vice migrate` konwertuje `vice.json`, `@mount`, `output <-` i `error() <-` do stylu v0.2.

Preferowana składnia v0.2 to:

```vice
use "lib.vice";

@entry
fn main() {
    let value => result.ok("ready");
    emit <- value.is_ok;
}

@test
fn invariant_holds() {
    if (false) {
        fail <- "invariant failed";
    }
}
```

Legacy aliasy są nadal parsowane, żeby projekty 0.1.1 mogły przejść przez migrację.
Nowe testy i przykłady powinny używać `use`, `emit <-`, `fail <-`, `result.ok`, `result.err`, `option.some` i `option.none`.
Słowo `fail` jest słowem kluczowym, więc przykład 0.1.1 z parametrem `fail` został zmigrowany do `should_fail`.

## Struktura repozytorium

### `Cargo.toml`
Odpowiedzialność: manifest crate 0.2.0, features i zależności.
Niezmiennik: core pozostaje domyślny, backendy są opcjonalne.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/lib.rs`
Odpowiedzialność: publiczny pipeline parse, load, check i run.
Niezmiennik: nie przywraca starego interpretera.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/main.rs`
Odpowiedzialność: CLI vice.
Niezmiennik: 0 sukces, 1 błąd programu, 2 błąd użycia.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/ast.rs`
Odpowiedzialność: AST i Span.
Niezmiennik: każde dziecko zachowuje source_id.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/token.rs`
Odpowiedzialność: rodzaje tokenów.
Niezmiennik: keyword wymaga pełnej integracji.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/lexer.rs`
Odpowiedzialność: tokenizacja.
Niezmiennik: lokalizacja ma linię, kolumnę i długość.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/parser.rs`
Odpowiedzialność: budowa AST.
Niezmiennik: parser nie zastępuje analizy typów.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/types.rs`
Odpowiedzialność: typy i kompatybilność.
Niezmiennik: map ma typ klucza i wartości.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/semantic.rs`
Odpowiedzialność: nazwy, typy, przepływ, efekty i traits.
Niezmiennik: błędy statyczne powstają przed VM.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/native_registry.rs`
Odpowiedzialność: sygnatury native.
Niezmiennik: musi odpowiadać runtime jeden do jednego.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/formatter.rs`
Odpowiedzialność: kanoniczny druk AST.
Niezmiennik: komentarze są zachowane, wynik idempotentny.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/error.rs`
Odpowiedzialność: ViceError i StackFrame.
Niezmiennik: diagnostyka zachowuje dokładny span.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/value.rs`
Odpowiedzialność: wartości runtime i MapKey.
Niezmiennik: klucz mapy zachowuje typ.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/stdlib.rs`
Odpowiedzialność: implementacja modułów builtin.
Niezmiennik: brak catch-all i pozornego sukcesu.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/vm/compiler.rs`
Odpowiedzialność: AST do bytecode.
Niezmiennik: compile_with_entry izoluje testy.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/vm/machine.rs`
Odpowiedzialność: wykonanie stosowe.
Niezmiennik: stan jest resetowany przed programem.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/vm/opcodes.rs`
Odpowiedzialność: zamknięty zestaw instrukcji.
Niezmiennik: zmiana wymaga compiler, verifier i VM.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/vm/verifier.rs`
Odpowiedzialność: kontrola bytecode.
Niezmiennik: uszkodzony program jest odrzucany.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/ui.rs`
Odpowiedzialność: renderer desktop Windows.
Niezmiennik: manager okien działa na main thread.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/ui_stub.rs`
Odpowiedzialność: kontrakty headless UI.
Niezmiennik: zachowuje typy bez udawania renderowania.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/audio.rs`
Odpowiedzialność: backend rodio.
Niezmiennik: aktywny tylko z feature audio.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/installer.rs`
Odpowiedzialność: plan instalacji.
Niezmiennik: najpierw waliduje komplet binarek.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/bin/vice-installer.rs`
Odpowiedzialność: UI instalatora eframe.
Niezmiennik: oddzielony od core.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/pkg.rs`
Odpowiedzialność: Package Manager (`vice pkg add`, `vice pkg list`, generowanie `vice.lock`).
Niezmiennik: zależności pobierane lub instalowane do katalogu `modules/` z zachowaniem integralności lockfile.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/db_sqlite.rs`
Odpowiedzialność: natywny wbudowany silnik SQLite (`sqlite.open`, `execute`, `query`, `close`, `last_insert_rowid`).
Niezmiennik: bezpieczne mapowanie typów parametrów i wyników, transakcyjność, obsługa bazy w pamięci (`:memory:`).
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `src/reg_engine.rs`
Odpowiedzialność: silnik wyrażeń regularnych (`regex.is_match`, `find`, `find_all`, `replace`, `split`).
Niezmiennik: standardowa semantyka Regex ze spójną obsługą błędów niepoprawnego wzorca.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji studentr, developer i AI.

### `src/matrix.rs`
Odpowiedzialność: wektorowo-macierzowe obliczenia numeryczne (`matrix.zeros`, `from_array`, `to_array`, `shape`, `mul`, `dot`, `transpose`).
Niezmiennik: weryfikacja wymiarów przed operacjami, szybkie operacje algebry liniowej.

### `src/crypto.rs`
Odpowiedzialność: funkcje kryptograficzne i kodowanie (`crypto.sha256`, `sha512`, `base64_encode`, `base64_decode`, `random_bytes`).
Niezmiennik: bezpieczne operacje oparte o sprawdzoną kryptografię (ring / base64).

### `src/net_http.rs`
Odpowiedzialność: natywne wywołania sieciowe REST/AI (`http.get`, `http.post`).
Niezmiennik: sprawdzanie uprawnień sandboxu (flaga allow_net) przed wysłaniem pakietu.

### `src/ffi.rs`
Odpowiedzialność: dynamiczny interfejs C-ABI (`ffi.load`, `ffi.call`, `ffi.free`).
Niezmiennik: rygorystyczna walidacja w sandboxie, obsługa wskaźników i typów numerycznych C.

### `tests/runner_300_matrix.rs`
Odpowiedzialność: automatyczna bateria 300 programów testowych Vice podzielonych na 3 zestawy po 100 programów:
- `tests/programs/simple/` (100 prostych testów składniowych, pętli, modułów podstawowych)
- `tests/programs/medium/` (100 średnich programów algorytmicznych: sortowanie, drzewa BST, kalkulator RPN, plecak DP)
- `tests/programs/heavy/` (100 ciężkich kompletnych silników: in-memory DB, sieci neuronowe forward pass, raycasting 2D, mikrowirtualna maszyna bajtkodowa, ECS, symulator blockchaina Proof-of-Work, solver Gaussa, router API, VFS)
Niezmiennik: każdy z 300 programów musi uruchomić się w VM bez błędów i wyemitować `PASS`.

### `tests/core.rs`
Odpowiedzialność: CLI, formatter i integracje.
Niezmiennik: główny kontrakt regresyjny.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `tests/corpus.rs`
Odpowiedzialność: przykłady i migracje.
Niezmiennik: każdy przykład przechodzi pełny check.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `tests/control_flow.rs`
Odpowiedzialność: return i match.
Niezmiennik: każda ścieżka niewoidowa terminuję wartością.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `tests/vm_stability.rs`
Odpowiedzialność: limity i verifier.
Niezmiennik: brak panik na złym bytecode.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `tests/traits_and_try.rs`
Odpowiedzialność: traits, Result, Option.
Niezmiennik: testy pozytywne i negatywne.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `tests/maps.rs`
Odpowiedzialność: typowane klucze map.
Niezmiennik: int i string pozostają różne.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `tests/stdlib_advanced.rs`
Odpowiedzialność: native runtime.
Niezmiennik: testuje realne workflow.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `vscode/candidate/vice-language-0.2.0/extension.js`
Odpowiedzialność: diagnostyki i formatter.
Niezmiennik: spawn bez shell, limit i timeout.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `Web/firestore.rules`
Odpowiedzialność: autoryzacja forum.
Niezmiennik: reguły serwera są nadrzędne.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `Web/storage.rules`
Odpowiedzialność: autoryzacja avatarów.
Niezmiennik: właściciel, ścieżka, typ i rozmiar.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `Web/firebase.json`
Odpowiedzialność: Hosting i CSP.
Niezmiennik: nowe źródła wymagają jawnej decyzji.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `Web/js/forum.js`
Odpowiedzialność: forum klienta.
Niezmiennik: klient nie jest granicą bezpieczeństwa.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `Web/js/profile.js`
Odpowiedzialność: konto i avatar.
Niezmiennik: avatar jest usuwany przed auth user.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `Web/js/admin.js`
Odpowiedzialność: konsola operacyjna.
Niezmiennik: widoki odpowiadają rolom.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `Web/tests/firestore.rules.test.mjs`
Odpowiedzialność: macierz allow i deny.
Niezmiennik: testy korzystają z emulatora.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

### `Web/tests/storage.rules.test.mjs`
Odpowiedzialność: granice Storage.
Niezmiennik: bez produkcyjnych zapisów.
Zmiana wymaga testu najbliższego kontraktowi.
Zmiana publiczna wymaga aktualizacji student, developer i AI.

## Decyzje architektoniczne

### Jeden runtime
Decyzja: aktywny jest wyłącznie zweryfikowany VM.
Uzasadnienie: zapobiega rozjazdowi semantyki.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Arrow syntax
Decyzja: Vice zachowuje strzałki =>, <- i ->.
Uzasadnienie: nowe API ma brzmieć jak Vice.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Statyczna kontrola
Decyzja: arity i typy sprawdza semantic analyzer.
Uzasadnienie: runtime obsługuje dane i środowisko.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Spany
Decyzja: każdy token i AST ma lokalizację.
Uzasadnienie: CLI i edytor używają wspólnej diagnostyki.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Bytecode verifier
Decyzja: CompiledProgram jest weryfikowany.
Uzasadnienie: VM nie ufa strukturze programu.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Test runner
Decyzja: @test wybiera osobny entry.
Uzasadnienie: przypadki są izolowane.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Błąd testu
Decyzja: v0.2 używa `fail <- message`, a `error() <- message` pozostaje legacy aliasem.
Uzasadnienie: testy mają czytelny czasownik, bez kopiowania obcej składni assert.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Argumenty
Decyzja: tylko wartości po -- trafiają do process.args.
Uzasadnienie: flagi CLI nie wyciekają.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Manifest
Decyzja: `vice.toml` jest manifestem głównym; legacy `vice.json` jest czytany tylko dla kompatybilności.
Uzasadnienie: blokuje wyjście z projektu.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Formatter
Decyzja: zachowuje # i //.
Uzasadnienie: formatowanie nie niszczy wiedzy autora.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### JSON CLI
Decyzja: check, test, doctor i doc mają tryb maszynowy.
Uzasadnienie: narzędzia nie parsują tekstu ludzkiego.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Safe stdlib
Decyzja: `fs.try_read_text`, `fs.try_write_text` i `json.try_parse` zwracają `Result`, a `env.get` oraz `cli.option` zwracają `Option`.
Uzasadnienie: lokalne narzędzia potrzebują obsłużyć brak pliku, niepoprawny JSON i brak argumentu bez natychmiastowego runtime error.
Konsekwencja: rejestr natywny, VM i dokumentacja muszą zgadzać się co do pól `is_ok`, `value`, `error`, `is_some`.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Native registry
Decyzja: rejestr jest zamknięty.
Uzasadnienie: nieistniejąca metoda jest błędem.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Audio feature
Decyzja: brak backendu daje jawny błąd.
Uzasadnienie: stub nie udaje sukcesu.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Listowanie katalogu
Decyzja: wynik jest sortowany.
Uzasadnienie: programy i testy są deterministyczne.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Unicode
Decyzja: string.len liczy znaki.
Uzasadnienie: nie zwraca liczby bajtów UTF-8.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### MapKey
Decyzja: tożsamość zachowuje typ i float bits.
Uzasadnienie: tekst nie definiuje klucza.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Return flow
Decyzja: analyzer dowodzi zwrotu.
Uzasadnienie: jednostronny if nie wystarcza.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Match
Decyzja: wildcard jest ostatni.
Uzasadnienie: bool i enum mogą być wyczerpujące.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Traits
Decyzja: impl ma dokładny kontrakt.
Uzasadnienie: brak, nadmiar i zły typ są błędem.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Read effects
Decyzja: @read blokuje efekty i mutacje.
Uzasadnienie: kontrakt jest tranzytywny.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Hybrid mode
Decyzja: highlevel chroni precyzję.
Uzasadnienie: lowlevel pozwala kontrolowaną truncation.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Limity VM
Decyzja: stack, depth i budget są kontrolowane.
Uzasadnienie: host może zatrzymać program.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Output
Decyzja: VM buforuje linie.
Uzasadnienie: test przypisuje output do przypadku.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Referencje
Decyzja: lokalne wartości używają stabilnych Slot.
Uzasadnienie: closure nie traci tożsamości.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Windows UI
Decyzja: manager działa na main thread.
Uzasadnienie: winit zachowuje kontrakt platformy.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Optional desktop
Decyzja: core nie ładuje wgpu.
Uzasadnienie: CLI konsolowe pozostaje lekkie.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Installer
Decyzja: ma osobną binarkę i feature.
Uzasadnienie: UAC nie obciąża core.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### VS Code
Decyzja: procesy są spawn bez powłoki.
Uzasadnienie: ścieżka nie staje się kodem shell.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Edytor
Decyzja: limit output to 4 MiB, timeout 20 sekund.
Uzasadnienie: narzędzie nie zawiesza UI.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Sieć
Decyzja: martwy net.rs usunięto.
Uzasadnienie: wróci dopiero z timeout, close i limitami.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### FFI
Decyzja: zaimplementowano bezpieczny mostek C-ABI FFI (`libloading`).
Uzasadnienie: ładowanie bibliotek dynamicznych (.dll, .so) bezpośrednio z poziomu VM za pomocą `ffi.load`, `ffi.call` i `ffi.free` z kontrolą sandboxa.
Konsekwencja: kod, test i dokumentacja są w pełni zsynchronizowane.

### Firestore
Decyzja: role sprawdza serwer.
Uzasadnienie: ukryty przycisk nie autoryzuje.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Username
Decyzja: rezerwacja jest atomowa.
Uzasadnienie: wdrożenie wymaga migracji istniejących nazw.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Reports
Decyzja: id to postId_uid.
Uzasadnienie: użytkownik nie mnoży zgłoszeń.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Role
Decyzja: admin nie banuje owner lub admin.
Uzasadnienie: chroni wyższą rolę.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Profile
Decyzja: listowanie prywatne wymaga admina.
Uzasadnienie: brak masowego wycieku.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Avatar
Decyzja: zapisuje właściciel pod własnym uid.
Uzasadnienie: reguły walidują plik.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### CSP
Decyzja: Hosting ogranicza źródła.
Uzasadnienie: zewnętrzny host wymaga przeglądu.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Emulatory
Decyzja: security tests nie piszą do produkcji.
Uzasadnienie: walidacja jest bezpieczna.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Dokumentacja
Decyzja: istnieją trzy źródła prawdy.
Uzasadnienie: plany nie udają funkcji.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

### Git
Decyzja: obce zmiany workspace są zachowane.
Uzasadnienie: AGENTS.md nie trafia do commitów.
Konsekwencja: kod, test i dokumentacja muszą pozostać zgodne.
Wyjątek wymaga jawnego ADR w commicie.
Nie opisuj roadmapy jako stanu istniejącego.

## AST i typy

`Program` przechowuje imports, functions, data_declarations, impls, top_level i sources.
`Span` ma line, column, length i source_id.
`Function` ma type_params, attributes, params, return_type, body i span.
`StmtKind` obejmuje deklaracje, sterowanie, match, IO, error i assignment.
`ExprKind` obejmuje literały, kolekcje, call, method, lambda, range, try i unwrap.
`Type` obejmuje prymitywy, array, map, function, data, enum, reference i unknown.
`Unknown` jest granicą dynamiczną JSON, nie zgodą na pomijanie kontroli.
`HybridMode` jest Default, HighLevel albo LowLevel.
`MapKey` rozróżnia Int, Bool, String i Float bits.
Trait wymaga self jako pierwszego parametru.
Impl traitu ma wszystkie i tylko zadeklarowane metody.
@read blokuje mutację parametrów, kolekcji i wywołania efektowe.
@autocheck uruchamia ścisłą inferencję.
@leaf gwarantuje, że funkcja nie wykonuje żadnych wywołań innych funkcji, metod ani domknięć (konflikt z @tailrec, @retry, @max_depth).
@wrapping wymusza arytmetykę zawijania (wrapping arithmetic) przy operacjach całkowitych (wyklucza @checked i @saturating).
@nothrow / @no_fail gwarantuje brak rzucania błędów (zakaz fail <-, zakaz unwrap ! oraz bezpośredniego wywoływania funkcji @fallible).
@no_io blokuje operacje I/O (zakaz emit <-, output <-, input -> oraz wołania modułów fs/io/println).
@sealed na enumie oraz traicie zabrania tworzenia bloków impl poza plikiem definicji.
@private / @internal ogranicza widoczność funkcji, struktury danych lub enumu wyłącznie do pliku, w którym zostały zadeklarowane (zakaz dostępu z innych plików via use/import; zakaz oznaczania funkcji @entry jako @private).
Optymalizator peephole kompilatora składa stałe binarne i unarne (constant folding) w fazie optymalizacji bajtkodu.

## Bytecode

### `Opcode::LoadConst`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::LoadLocal`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::StoreLocal`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::LoadGlobal`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::StoreGlobal`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::ReferenceLocal`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::ReferenceProperty`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::ReferenceIndex`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Dereference`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::AssignReference`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Add`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Sub`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Mul`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Div`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Mod`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Pow`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Neg`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Eq`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Neq`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Lt`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Lte`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Gt`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Gte`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::And`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Or`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Not`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Jump`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::JumpIfFalse`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::JumpIfTrue`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Loop`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Call`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::MakeClosure`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Return`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::MakeArray`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::MakeMap`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::MakeInstance`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::GetProperty`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::SetProperty`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::GetIndex`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::SetIndex`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Unwrap`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Try`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::MethodCall`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::NativeCall`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Raise`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Dup`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Pop`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Len`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Output`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::HighLevelDiv`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::TruncateToInt`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Input`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Swap`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

### `Opcode::Rotate3`
Compiler emituje tę instrukcję z poprawnym układem stosu.
Verifier sprawdza strukturę przed wykonaniem.
VM musi zwrócić ViceError zamiast paniki.

## VM — niezmienniki

VmConfig ma max_call_depth 64.
VmConfig ma max_stack_size 1_000_000.
instruction_budget jest opcjonalny.
interpret czyści frames, stack, output i licznik instrukcji.
Globals zachowują builtin modules i callable programu.
Brak entry podczas interpretacji jest runtime error.
CallFrame przechowuje function, ip i Vec<Slot>.
Referencja może wskazywać local, property albo index path.
Output jest buforem, nie bezpośrednim stdout.
Input czyta kolejne elementy wektora.
NativeCall deleguje do call_native_vm.
Stack trace dostaje function, file, line i column.

## CLI — kontrakty

`run file|dir [-- args...]` wykonuje entry.
`check [--json] file|dir` nie wykonuje kodu.
`test [--json] [path]` odkrywa @test.
`new path` odmawia nadpisania.
`fmt path` zapisuje format.
`fmt --check path` nie zapisuje.
`fmt --stdout file` wymaga jednego pliku.
`fmt --stdin` formatuje bufor edytora.
`doctor [--json]` raportuje toolchain.
`ui-check` raportuje UI bindings.
Exit 0 oznacza sukces.
Exit 1 oznacza błąd źródła, semantyki, runtime lub IO.
Exit 2 oznacza niepoprawne argumenty CLI.
Manifest entry jest względny i nie może zawierać parent/root/prefix.
Rekurencyjny scan pomija katalogi ukryte, target i node_modules.

## Standard library — reguły

BUILTIN_MODULES, NativeRegistry i call_native_vm są jednym kontraktem.
Każdy native ma dokładną arity i typ wyniku.
Brak metody daje Unknown method przed runtime.
Błąd IO zawiera operację i ścieżkę.
fs.list_dir sortuje nazwy.
fs.create_dir używa create_dir_all.
fs.write_json zapisuje pretty JSON z newline.
string.len liczy Unicode scalar values.
math.floor, ceil i round zwracają int.
random nie jest kryptograficzny.
process.args używa thread-local state bieżącego wykonania.
audio bez feature nie udaje sukcesu.
UI headless zwraca prawidłowy kształt typu.

## Web — granica bezpieczeństwa

Authentication identyfikuje użytkownika, ale nie zastępuje reguł.
Firestore Rules autoryzują ownership i role.
Username reservations są atomowe.
Profile list jest administracyjny.
Staff może czytać właściwe rekordy operacyjne.
Post author nie może fałszować autora ani liczników.
Like i reply są przypisane do uid.
Report ma deterministyczny identyfikator.
Resolution raportu zapisuje wykonawcę.
Admin nie może banować owner ani admin.
Owner zarządza rolami.
Storage ogranicza avatar do ścieżki uid właściciela.
CSP ogranicza script, style, img i connect.
Security tests używają emulatorów.
Deploy restrykcyjnych reguł wymaga migracji username reservations.

## Procedura zmiany języka

1. Zapisz semantykę i granice.
2. Dodaj test pozytywny.
3. Dodaj test negatywny.
4. Rozszerz token tylko jeśli konieczne.
5. Rozszerz AST ze Span.
6. Rozszerz parser.
7. Rozszerz semantic analyzer.
8. Skompiluj do opcode.
9. Rozszerz verifier jeśli trzeba.
10. Rozszerz VM.
11. Rozszerz formatter.
12. Rozszerz VS Code grammar i snippets.
13. Dodaj przykład do corpus.
14. Uruchom pełne testy.
15. Zaktualizuj trzy dokumentacje.

## Procedura native

1. Wybierz moduł.
2. Dodaj moduł do BUILTIN_MODULES tylko jeśli nowy.
3. Dodaj NativeFuncDef.
4. Dodaj dokładną gałąź call_native_vm.
5. Sprawdź arity w runtime.
6. Nie dodawaj catch-all.
7. Przetestuj typ poprawny.
8. Przetestuj typ błędny.
9. Przetestuj realny rezultat.
10. Przetestuj błąd środowiska.
11. Dla feature przetestuj wariant włączony i wyłączony.

## Komendy walidacji

`cargo fmt --all -- --check`.
`cargo test --all-targets`.
`cargo test --features desktop,audio --lib --bin vice`.
`cargo clippy --all-targets --all-features -- -D warnings`.
`npm test` w Web.
`npm run test:rules` w Web.
`npm audit --omit=dev` w Web.
`node --check` dla extension.js.
`git diff --check`.

## Czysty Vice (Pure Vice) — System Atrybutów i Algorytmów

### Nowe Atrybuty Semantyczne (v6 & v7)
- `@thread_safe` (alias `@sync`): Wymusza bezpieczeństwo współbieżności. Blokuje mutację zmiennych globalnych, wywołania modułów o efektach ubocznych (`fs`, `ui`, `net`, `time`, `env`, `sys`, `os`, `process`) oraz wywołania funkcji nieoznaczonych jako bezpieczne.
- `@pure_math` (alias `@strict_math`): Dedykowany czystym obliczeniom matematycznym. Wymaga co najmniej jednego parametru, numerycznego typu zwracanego (`int` lub `float`), zabrania użycia z `@entry` oraz blokuje wywołania modułów niemematycznych (dozwolone wyłącznie `math` i `hash`).
- `@pure_string` (alias `@string_only`): Dedykowany deterministycznemu przetwarzaniu tekstu. Wymaga parametrów typu `string` lub `int`, zwraca `string`, `int` lub `bool`, zabrania `@entry` i mutacji zmiennych globalnych, ogranicza wywołania do modułów `string` oraz `hash`.
- `@no_alias`: Statyczna weryfikacja unikalności referencji. Wymaga co najmniej dwóch parametrów referencyjnych (`&int`, `&mut T`). Odrzuca w czasie kompilacji jakiekolwiek wywołania przekazujące tę samą zmienną źródłową do więcej niż jednego argumentu referencyjnego.
- `@clamp(min, max)`: Weryfikacja zakresu wartości zwracanych w czasie kompilacji dla wartości stałych oraz walidacja warunku `min <= max`.
- `@non_empty`: Wymaga co najmniej jednego parametru kolekcji (`string`, `array`, `map`) i statycznie odrzuca wywołania z pustymi literałami (`""`, `[]`, `{}`).
- `@non_negative`: Zapewnia, że argumenty numeryczne są nieujemne ($\ge 0$). Odrzuca stałe ujemne literały w miejscu wywołania.
- `@bitwise_only`: Wymusza operacje wyłącznie na liczbach całkowitych (`int`) dla parametrów i zwracanej wartości; statycznie zabrania występowania literałów zmiennoprzecinkowych.

### Nowe Algorytmy Biblioteki Standardowej
- `string`:
  - `s.levenshtein(other)` / `string.levenshtein(s1, s2)`: Oblicza odległość edycyjną Levenshteina zoptymalizowaną pamięciowo ($O(\min(N, M))$).
  - `s.hamming(other)` / `string.hamming(s1, s2)`: Oblicza odległość Hamminga dla ciągów równej długości (zwraca `-1` przy różnych długościach).
  - `s.slugify()` / `string.slugify(s)`: Konwertuje tekst na bezpieczny format URL/nazw plików (lowercase, myślniki, strip znaków specjalnych).
  - `s.truncate(max_len, ellipsis)`: Bezpieczne ucinanie napisów z dołączaniem wielokropka.
  - `s.hex_encode()` / `s.hex_decode()`: Kodowanie i dekodowanie szesnastkowe.
- `math`:
  - `math.clamp01(val)`: Obcinanie wartości do przedziału $[0.0, 1.0]$.
  - `math.lerp_inv(a, b, v)`: Odwrotna interpolacja liniowa.
  - `math.remap(v, in_min, in_max, out_min, out_max)`: Liniowe mapowanie wartości między przedziałami.
  - `math.midpoint(a, b)`: Odporne na przepełnienie obliczanie punktu środkowego.
  - `math.approx_eq(a, b, eps)`: Porównywanie liczb zmiennoprzecinkowych z tolerancją $\varepsilon$.
  - `math.is_prime(n)`: Test pierwszości liczby całkowitej.
  - `math.count_ones(n)`, `math.leading_zeros(n)`, `math.trailing_zeros(n)`: Operacje bitowe na liczbach całkowitych.
  - `math.is_power_of_two(n)`, `math.next_power_of_two(n)`: Potęgi dwójki.
- `array`:
  - `arr.binary_search(target)`: Wyszukiwanie binarne $O(\log n)$.
  - `arr.rotate(k)`: Cykliczne przesunięcie elementów.
  - `array.repeat(val, count)`: Generowanie tablicy powtórzeń.
  - `arr.windows(size)`: Nakładające się okna podtablic.
  - `arr.frequencies()`: Zliczanie wystąpień elementów do mapy.
- `hash`:
  - `hash.xxhash32(data, seed)`: Implementacja algorytmu XXH32.
  - `hash.murmur3(data)`: Hash Murmur3-32.
  - `hash.rot13(s)` / `s.rot13()`: Szyfr przesuwający ROT13.

### Optymalizacje VM i Kompilatora
- **Constant Folding**: Ewaluacja w czasie kompilacji dla `levenshtein`, `slugify`, `hex_encode`, `hex_decode`, `clamp01`, `midpoint`, `remap`, `is_prime`, `count_ones`, `sum`, `product`, `dedup`, `reverse`.
- **Peephole Optimizer**:
  - Inwersja komparatorów ze swapem: `[Swap, Lt] => [Gt]`, `[Swap, Lte] => [Gte]`, `[Swap, Gt] => [Lt]`, `[Swap, Gte] => [Lte]`.
  - Eliminacja podwójnej negacji: `[Not, Not] => []`.
  - Składanie stałych i skróty skoków warunkowych.

## Lista audytowa


## Koniec

Dokument opisuje stan implementacji, nie obietnice.
Roadmapa należy do issue trackera i nie jest kontraktem runtime.

