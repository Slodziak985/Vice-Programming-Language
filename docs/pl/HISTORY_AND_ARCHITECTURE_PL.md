# đźŚŚ JÄ™zyk Programowania Vice (v0.2.0 â€” Final Release)
### Oficjalna Kronika, Architektura i PodrÄ™cznik Projektu

[![Wersja](https://img.shields.io/badge/Vice-0.2.0--stable-blue.svg)](https://github.com/Slodziak985/Vice-Programming-Language)
[![Status](https://img.shields.io/badge/Status-Project%20Completed%20%7C%20Feature--Complete-success.svg)](https://github.com/Slodziak985/Vice-Programming-Language)
[![Pipeline](https://img.shields.io/badge/Architecture-AST%20%E2%86%92%20Bytecode%20%E2%86%92%20VM-orange.svg)](https://github.com/Slodziak985/Vice-Programming-Language)
[![Rust](https://img.shields.io/badge/Implementation-Rust%202021-red.svg)](https://www.rust-lang.org/)
[![Licencja](https://img.shields.io/badge/License-MIT%20%2F%20Open-green.svg)](LICENSE)

> *"Kod powinien jasno pokazywaÄ‡, skÄ…d dane przychodzÄ…, jak sÄ… przeksztaĹ‚cane i dokÄ…d zmierzajÄ…. Bez magii, bez ukrytych efektĂłw ubocznych."*  
> â€” **Slodziak985, TwĂłrca i GĹ‚Ăłwny Architekt JÄ™zyka Vice**

---

## đź“– 1. Geneza i Filozofia Vice

Projekt **Vice** narodziĹ‚ siÄ™ z potrzeby stworzenia jÄ™zyka programowania o **bezwzglÄ™dnie czytelnym przepĹ‚ywie danych** i **deterministycznym Ĺ›rodowisku wykonawczym**. 

Podczas gdy wspĂłĹ‚czesne jÄ™zyki programowania stajÄ… siÄ™ coraz bardziej wieloparadygmatowe, skomplikowane i przeĹ‚adowane skĹ‚adniowym lukrem, Vice stawia na surowÄ… prostotÄ™, architektonicznÄ… przejrzystoĹ›Ä‡ oraz izolowane Ĺ›rodowisko wykonawcze (sandboxing).

### GĹ‚Ăłwne filary filozofii Vice:
1. **ReguĹ‚a Trzech StrzaĹ‚ek**:
   - `=>` â€” **WiÄ…zanie i Inicjalizacja**: Deklaracja zmiennej (`let x => 42;`), dopasowanie w `match` lub nazwanie argumentu.
   - `<-` â€” **Emisja i WyjĹ›cie**: Wyprowadzenie wartoĹ›ci do strumienia (`emit <- wartosc;`), zgĹ‚oszenie niepowodzenia w teĹ›cie (`fail <- "komunikat";`).
   - `->` â€” **PrzepĹ‚yw i WejĹ›cie**: Wskazanie typu zwracanego przez funkcjÄ™ (`fn suma() -> int`) lub skierowanie wejĹ›cia do zmiennej (`input -> bufor;`).
2. **Determinizm i Semantyka WartoĹ›ci**:
   Struktury `data` oraz tablice `array` posiadajÄ… semantykÄ™ wartoĹ›ci (*pass-by-value / immutable updates*). Modyfikacje struktur nie wywoĹ‚ujÄ… niejawnych skutkĂłw ubocznych w odlegĹ‚ych czÄ™Ĺ›ciach programu.
3. **BezpieczeĹ„stwo w epoce AI**:
   Polecenie `vice schema` generuje Ĺ›cisĹ‚Ä…, maszynowÄ… specyfikacjÄ™ JSON wszystkich moduĹ‚Ăłw i ich sygnatur. Modele AI mogÄ… pisaÄ‡ kod w Vice bez halucynacji o nieistniejÄ…cych funkcjach, a wirtualna maszyna moĹĽe uruchamiaÄ‡ taki kod w bezpiecznej piaskownicy.
4. **Wbudowane obliczenia macierzowe w stylu NumPy**:
   ModuĹ‚ `matrix` jest zintegrowany natywnie w jÄ…drze jÄ™zyka, umoĹĽliwiajÄ…c implementacjÄ™ warstw sieci neuronowych i algebry liniowej bezpoĹ›rednio w skĹ‚adni Vice.

---

## âŹł 2. Kronika Rozwoju: Droga od Prototypu do Wersji 0.2.0

Droga do ukoĹ„czenia Vice byĹ‚a peĹ‚na odwaĹĽnych decyzji architektonicznych, radykalnych refaktoryzacji i nieustannego dÄ…ĹĽenia do inĹĽynieryjnej doskonaĹ‚oĹ›ci:

```mermaid
flowchart TD
    A["Faza 0.1.x: Eksploracja i Prototyp"] -->|Drzewiasty Interpreter AST| B["Kryzys SkalowalnoĹ›ci"]
    B -->|Wielka Refaktoryzacja Rust| C["Projekt Wirtualnej Maszyny (VM)"]
    C -->|Kompilacja do Bajtkodu + Verifier| D["Faza 0.2.0: Stabilizacja Architektury"]
    D -->|Natywne moduĹ‚y: matrix, sqlite, ffi, crypto| E["Rozbudowa Ekosystemu"]
    E -->|CLI, GUI Installer, VS Code, Web Platform| F["Weryfikacja: 50+ Test Suites"]
    F --> G["Vice 0.2.0: Projekt UkoĹ„czony (Feature-Complete)"]
```

### Faza I: Narodziny i wczesne eksperymenty (v0.1.x)
W poczÄ…tkowej fazie Vice byĹ‚ dynamicznie interpretowanym jÄ™zykiem drzewiastym. SkĹ‚adnia eksperymentowaĹ‚a z mechanizmem `@mount` do Ĺ‚Ä…czenia plikĂłw oraz instrukcjÄ… `output <-`. ChoÄ‡ pozwalaĹ‚o to na bĹ‚yskawiczne prototypowanie prostych gier i skryptĂłw, interpreter drzewiasty miaĹ‚ swoje ograniczenia: brakowaĹ‚o rygorystycznej kontroli typĂłw, zuĹĽycie pamiÄ™ci byĹ‚o trudne do przewidzenia, a optymalizacja pÄ™tli napotykaĹ‚a barierÄ™ narzutu wÄ™zĹ‚Ăłw AST.

### Faza II: Wielka Refaktoryzacja i Narodziny Maszyny Wirtualnej (PrzejĹ›cie do v0.2.0)
Autor podjÄ…Ĺ‚ kluczowÄ… decyzjÄ™ inĹĽynieryjnÄ…: **caĹ‚kowite odrzucenie powolnego interpretera drzewiastego i przepisanie rdzenia w Rust na model wieloetapowego kompilatora bajtkodu oraz weryfikowanej maszyny wirtualnej (Stack-based VM)**:
- Wprowadzono **Loader z detekcjÄ… cykli importu**,
- Zbudowano **Semantic Analyzer** sprawdzajÄ…cy typy, kontrakty cech (`traits`) oraz poprawnoĹ›Ä‡ przepĹ‚ywu sterowania przed uruchomieniem ani jednej linijki bajtkodu,
- Zaimplementowano **Verifier**, ktĂłry skanuje skompilowane instrukcje (Chunki) i gwarantuje, ĹĽe skoki i ramki stosu nie wywoĹ‚ajÄ… *undefined behavior*,
- Przeprojektowano kontrakt projektowy na nowoczesny standard `vice.toml`.

### Faza III: Ekosystem, ModuĹ‚y Natywne i NarzÄ™dzia
Gdy silnik osiÄ…gnÄ…Ĺ‚ stabilnoĹ›Ä‡, zintegrowano z nim kluczowe moduĹ‚y przemysĹ‚owe:
- **`matrix`**: MnoĹĽenie, transpozycja, iloczyn skalarny, reshape, wektory wag neuronowych,
- **`sqlite`**: Bezpieczne, wbudowane bazy danych relacyjnych prosto z Vice,
- **`ffi`**: MoĹĽliwoĹ›Ä‡ wywoĹ‚ywania bibliotek dynamicznych (.dll / .so) zgodnych z C-ABI,
- **`crypto`** & **`regex`**: SHA256, Base64, silnik wyraĹĽeĹ„ regularnych,
- **Desktop & Installer**: ObsĹ‚uga okien i akceleracji sprzÄ™towej (`wgpu`, `winit`), dĹşwiÄ™k (`rodio`) oraz graficzny instalator Windows oparty o `egui`.

### Faza IV: ZwieĹ„czenie i ZamkniÄ™cie Projektu
Vice przeszedĹ‚ rygorystycznÄ… seriÄ™ ponad **50 zautomatyzowanych zestawĂłw testĂłw integracyjnych** (testujÄ…cych sandboxing pamiÄ™ci, odpornoĹ›Ä‡ parsera, limity alokacji, rekurencjÄ™ i operacje numeryczne). OsiÄ…gajÄ…c bezkompromisowÄ… stabilnoĹ›Ä‡, projekt zostaĹ‚ formalnie uznany za **ukoĹ„czony**.

---

## đźŹ›ď¸Ź 3. Architektura WewnÄ™trzna (Deep Dive)

Pipeline wykonawczy w Vice 0.2.0 jest czysty, moduĹ‚owy i w peĹ‚ni zaimplementowany w jÄ™zyku Rust:

```
[ Kod ĹşrĂłdĹ‚owy *.vice ]
         â”‚
         â–Ľ
 1. Loader (Resolving use, detekcja cykli, weryfikacja granic projektu)
         â”‚
         â–Ľ
 2. Lexer (StrumieĹ„ tokenĂłw ze spanami: linia, kolumna, dĹ‚ugoĹ›Ä‡)
         â”‚
         â–Ľ
 3. Parser (Budowanie Drzewa SkĹ‚adni Abstrakcyjnej - AST)
         â”‚
         â–Ľ
 4. Semantic Analyzer (RozwiÄ…zywanie nazw, typĂłw, traits, efektĂłw)
         â”‚
         â–Ľ
 5. Bytecode Compiler (Generowanie ChunkĂłw bajtkodu)
         â”‚
         â–Ľ
 6. Verifier (Statyczna weryfikacja niezmiennikĂłw bajtkodu i skokĂłw)
         â”‚
         â–Ľ
 7. Stack VM (Wirtualna Maszyna: ramki wywoĹ‚aĹ„, ochrona stosu, pamiÄ™Ä‡)
         â”‚
         â–Ľ
[ Wynik wykonania / Emisja bufora wyjĹ›cia ]
```

### Specyfikacja podsystemĂłw:
- **Loader**: Gwarantuje bezpieczeĹ„stwo Ĺ›cieĹĽek â€” importy `use` nie mogÄ… wychodziÄ‡ poza root projektu ani odwoĹ‚ywaÄ‡ siÄ™ do niebezpiecznych Ĺ›cieĹĽek bezwzglÄ™dnych.
- **Formatter (`vice fmt`)**: Zintegrowany z AST i tokenami, dba o zachowanie komentarzy (`#`, `//`) i prawidĹ‚owe formatowanie 4 spacjami.
- **Wirtualna Maszyna (VM)**: ObsĹ‚uguje stos wartoĹ›ci (liczby `f64`/`int`, stringi, bool, struktury `data`, enumi, mapy, tablice). VM jest w peĹ‚ni resetowalna i izolowana.

---

## đź’» 4. Prezentacja SkĹ‚adni i PrzykĹ‚ady Kodu

### 1. Klasyczny "Witaj Ĺšwiecie" i Jawny PrzepĹ‚yw
```vice
use "std/io.vice";

@entry
fn main() {
    let powitanie => "Witaj w Ĺ›wiecie Vice 0.2.0!";
    emit <- powitanie;
}
```

### 2. Warstwa Sieci Neuronowej z UĹĽyciem ModuĹ‚u `matrix`
PrzykĹ‚ad obliczajÄ…cy krok w przĂłd (*forward pass*) warstwy neuronowej:
```vice
# Forward pass: Y = W * X
fn main() {
    emit <- "--- Vice Neural Layer Simulation ---";

    # Wagi neuronowe (2 neurony, 3 wejscia): macierz 2x3
    mut weights_data = [0.5, -0.2, 0.8, 0.1, 0.9, -0.4];
    mut W = matrix.from_array(weights_data, 2, 3);

    # Wejscie (3 cechy): macierz 3x1 (wektor kolumnowy)
    mut inputs_data = [1.0, 2.0, 0.5];
    mut X = matrix.from_array(inputs_data, 3, 1);

    # Mnozenie macierzowe: Y = W * X (wynik 2x1)
    mut Y = matrix.mul(W, X);
    emit <- "Wyjscie neuronow:";
    emit <- matrix.to_array(Y);

    # Suma wszystkich wag
    let suma => matrix.sum(W);
    emit <- suma;
}
```

### 3. Struktury Danych, Typy i Pattern Matching
```vice
data Punkt {
    x: int => 0;
    y: int => 0;
}

enum Status {
    Aktywny,
    Wstrzymany,
    Blad(string)
}

fn sprawdz(status: Status) -> string {
    return match (status) {
        Status.Aktywny => "System dziala.",
        Status.Wstrzymany => "Oczekiwanie.",
        Status.Blad(komunikat) => komunikat,
        _ => "Nieznany stan."
    };
}
```

### 4. Wbudowane Testy Jednostkowe
```vice
@test
fn test_arytmetyki() {
    let a => 10;
    let b => 20;
    if (a + b != 30) {
        fail <- "Blad dodawania w Vice!";
    }
}
```

---

## đź› ď¸Ź 5. Zestaw NarzÄ™dzi CLI

Wszystkie operacje projektowe wykonuje uniwersalna binarka `vice`:

| Polecenie | Opis |
|---|---|
| `vice run <plik\|.>` | Kompiluje program do bajtkodu i natychmiast wykonuje go w VM |
| `vice check <plik\|.>` | Wykonuje peĹ‚ny pipeline (Loader $\to$ Parser $\to$ Semantyka) bez uruchamiania |
| `vice test <.>` | Automatycznie wykrywa i uruchamia testy oznaczone adnotacjÄ… `@test` |
| `vice fmt <plik\|.>` | Formatuje pliki ĹşrĂłdĹ‚owe zachowujÄ…c komentarze i strukturÄ™ |
| `vice schema` | Zwraca peĹ‚ny schemat JSON API dostÄ™pnego dla modeli AI |
| `vice new <nazwa>` | Generuje kompletny szablon nowego projektu z `vice.toml` |
| `vice migrate` | Automatycznie konwertuje kod ze starego formatu v0.1.1 do standardu v0.2.0 |

---

## đź“¦ 6. ZawartoĹ›Ä‡ Paczki Wydaniowej (Release Artifacts)

Kompilacja dystrybucyjna dla Windows x86-64 zawiera:
- `vice.exe` â€” GĹ‚Ăłwny kompilator, weryfikator i maszyna wirtualna z obsĹ‚ugÄ… desktop/GPU.
- `ViceInstaller.exe` â€” Dedykowany instalator Windows z graficznym interfejsem uĹĽytkownika.
- `vicetool.exe` â€” NarzÄ™dzie pomocnicze (Vice Asset/App Manager).
- `std/` â€” Katalog standardowych moduĹ‚Ăłw i bibliotek Vice SDK.
- `examples/` â€” PrzykĹ‚ady referencyjne (gry, algorytmy, operacje macierzowe).

---

## đźŚź 7. Epilog i Dziedzictwo Projektu

Projekt Vice udowodniĹ‚, ĹĽe niezaleĹĽny twĂłrca oprogramowania, wyposaĹĽony w determinacjÄ™, pasjÄ™ i nowoczesne narzÄ™dzia systemowe, jest w stanie zaprojektowaÄ‡ i zbudowaÄ‡ od zera kompletnÄ… platformÄ™ programistycznÄ….

Vice nie musi rywalizowaÄ‡ z gigantami rynku, by mieÄ‡ sens. Jego wartoĹ›Ä‡ tkwi w:
- **Kunszcie inĹĽynierskim**: Czystym, przetestowanym i bezpiecznym kodzie Rust,
- **WartoĹ›ci edukacyjnej**: Stanowi ĹĽywy, dziaĹ‚ajÄ…cy podrÄ™cznik konstrukcji maszyn wirtualnych dla studentĂłw i pasjonatĂłw informatyki,
- **Precyzji**: Pokazuje, jak czytelny i przewidywalny moĹĽe byÄ‡ jÄ™zyk zorientowany na jawny przepĹ‚yw danych.

Projekt osiÄ…gnÄ…Ĺ‚ swĂłj cel i przechodzi w stan spoczynku jako dzieĹ‚o kompletne.

---

**Autor i Architekt**: Slodziak985  
**Data wydania v0.2.0**: PaĹşdziernik 2026  
**JÄ™zyk rdzenia**: Rust 2021  
*Wszelkie prawa zastrzeĹĽone. Projekt Open Source.*

