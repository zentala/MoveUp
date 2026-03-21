# Product Vision — Ergonomics Coach & Business Model

_Date: 2026-03-21 | Source: brainstorming session with zentala_

---

## Core Philosophy (najważniejsze)

> **Najgorsza pozycja to ta, w której siedzisz za długo.**

Cel aplikacji NIE jest "więcej stać".
Cel jest: **częsta zmiana pozycji = zdrowy rytm pracy**.

- Tracker = zbiera dane, user nic nie zmienia → przegrywa
- **Coach = interpretuje dane, mówi co robić TERAZ → wygrywa**

**North star metric:** % dni z ≥3 zmianami pozycji

---

## Spektrum pozycji (nie tylko siedzenie/stanie)

### Statyczne
- klasyczne siedzenie (90°)
- siedzenie aktywne (piłka, stołek balansujący)
- klęcznik (otwarty kąt bioder)
- półsiedzenie / "perching" (wysoki stołek)
- stanie

### Pół-dynamiczne (najciekawsze)
- **lean** — oparcie o podpórkę/biurko z odciążeniem
- jedna noga wyżej (podnóżek)
- przysiad podparty ("rest squat")
- opieranie się biodrem o blat

### Mobilne
- klęk (jedno/dwa kolana)
- chodzenie (walking pad)
- pacing przy pracy głosowej
- siad na podłodze (skrzyżny, 90/90)

**Na MVP: tylko siedzenie i stanie. Architektura musi być extensible.**

---

## Scoring Model

```
+1  zmiana pozycji
+2  stanie aktywne (z ruchem)
+3  chodzenie / walking
-2  60 min bez zmiany
-5  90 min bez zmiany (kara)
```

Nagradzaj **zmianę i różnorodność**, nie czas trwania.
Spacer = 2–3x więcej punktów niż stanie.

---

## Architektura systemu (docelowa)

### Warstwy sensorów
1. **Biurko** — czujnik wysokości (już mamy: ESP32 + VL53L1X)
2. **Obecność** — mmWave lub PIR sensor (planowane)
3. **Ciało** — smartwatch (ruch, HRV, proximity BLE)
4. **Komputer** — keyboard/mouse activity, voice activity

### Hub użytkownika
- stary telefon / tablet na biurku
- zbiera eventy ze wszystkich sensorów
- działa BEZ instalacji na komputerze firmowym (fallback)
- proximity detection przez BLE (telefon ↔ smartwatch)

### Tryby pracy (future)
- Deep Work — stabilne pozycje, dłuższe cykle
- Voice/Thinking — ruch, pacing
- Call/Meeting — stanie + chodzenie
- Low Energy — aktywne siedzenie, mikroruchy

---

## Konteksty użytkownika (scenariusze)

| Scenariusz | Dostępne dane | Zachowanie systemu |
|---|---|---|
| Przy komputerze | height + input + presence | pełna precyzja |
| Przy biurku bez kompa | presence + watch | system działa |
| Chodzenie po domu | tylko watch | "mobility mode" |
| Praca firmowa (brak instalacji) | telefon + sensor | fallback działa |
| Outdoor (docelowe) | watch + voice | brak biurka, optymalizacja energii |

---

## Wizja outdoor / AR (długoterminowa)

- Okulary AR (XREAL Air 2, Rokid Max — ~1000–3000 zł) jako "monitor przed oczami"
- Smartwatch steruje agentem AI przez głos
- User chodzi po parku, rozmawia z agentem, widzi ekran w okularach
- Agent koordynuje pracę innych agentów na stacjonarnym komputerze

> "Operating system dla pracy człowieka w erze AI"

---

## Model biznesowy

### Faza 1 — teraz
- **Afiliacja**: polecanie mat, stołków, walking padów, klęczników
- **Aplikacja free** — tracker + podstawowy coach

### Faza 2
- **Własny sensor** (hardware kit do każdego biurka)
- Bundle: sensor + mata + aplikacja

### Faza 3
- **SaaS premium** — zaawansowany coach, historia, insighty, integracje
- Własna linia hardware (biurka, akcesoria ergonomiczne)

### Sklep / content
- Baza wiedzy: "jak dobrać klęcznik", "mata vs brak maty", "jak ustawić stację pracy"
- Edukacja in-app: mikro-porady w kontekście (np. "dlaczego teraz lean?")
- Onboarding: "ustaw swoją stację pracy" → personalizacja → zakup akcesoriów

---

## Co NIE należy robić teraz (backlog)

- AI agenci
- Timeline screenshotów
- AR / okulary
- Smartwatch integracja
- Scraping serwisów społecznościowych
- Kamera / detekcja postury
- Inteligentne lustro (długoterminowe)

---

## MVP — co dowieźć najpierw

1. Detekcja pozycji (już mamy)
2. Prosty coach — overlay: "Siedzisz 63 min → zmień tryb"
3. Mobility score (minimalny)
4. Manual input (tap: zmieniłem pozycję / tryb pracy)
5. Mikro-feedback natychmiastowy ("+2 za zmianę")

**KPI do mierzenia:**
- % dni z ≥3 zmianami pozycji
- avg stagnation time
- % sugestii coach'a, na które user reaguje
