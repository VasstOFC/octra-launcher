# EMERUSUI — MASTER DESIGN SYSTEM

## 0. DEFINICJA STYLU

Zaprojektuj cały interfejs produktu zgodnie z systemem **EmerusUI**.

EmerusUI to autorski, premium design system przeznaczony do tworzenia spójnych interfejsów dla:

* launcherów gier,
* aplikacji desktopowych,
* stron internetowych,
* aplikacji mobilnych,
* pseudo-aplikacji uruchamianych w grze,
* paneli administracyjnych,
* tabletów i komputerów w środowisku gry,
* HUD-ów i menu,
* dashboardów,
* systemów zarządzania kontem,
* sklepów,
* ekranów ustawień,
* systemów inventory,
* paneli pojazdów,
* systemów społecznościowych,
* aplikacji związanych z Minecraft/FiveM/GTA i podobnymi środowiskami.

EmerusUI NIE jest zwykłym "dark mode".

Ma sprawiać wrażenie produktu stworzonego przez bardzo doświadczony zespół product designerów.

Główne odczucie:

**premium + cinematic + technical + minimal + mature + calm + precise**

Interfejs powinien wyglądać drogo, ale nie krzykliwie.

Powinien sprawiać wrażenie:

> "To jest profesjonalny produkt, w którym każdy element znajduje się tutaj z konkretnego powodu."

Nie kopiuj bezpośrednio żadnego istniejącego produktu ani marki.

Inspiruj się zasadami profesjonalnego UI/UX, ale zachowaj własną identyfikację EmerusUI.

---

# 1. NAJWAŻNIEJSZA ZASADA

## CLARITY OVER DECORATION

Najważniejsza jest czytelność i zrozumiałość.

Nie dodawaj elementów tylko dlatego, że wyglądają efektownie.

Każdy:

* panel,
* cień,
* blur,
* gradient,
* border,
* glow,
* animacja,
* ikona,
* separator,
* badge,
* kolor

musi mieć funkcję.

Jeżeli dekoracyjny element pogarsza czytelność, usuń go.

EmerusUI ma być atrakcyjny nawet wtedy, gdy usunie się 80% dekoracyjnych efektów.

---

# 2. GŁÓWNE ZASADY UX

Stosuj następujące zasady w KAŻDYM ekranie:

1. Visual hierarchy
2. Consistent spacing
3. Strong alignment
4. Clear grouping
5. Predictable interaction
6. Obvious affordances
7. Sufficient contrast
8. Minimal cognitive load
9. Clear feedback
10. Clear states
11. Progressive disclosure
12. Consistent component behavior
13. Minimal friction
14. Readable typography
15. Information density controlled by hierarchy

Użytkownik powinien w ciągu kilku sekund rozumieć:

* gdzie jest,
* co może zrobić,
* co jest najważniejsze,
* co jest drugorzędne,
* jaki element jest interaktywny,
* jaki jest aktualny stan,
* co wydarzy się po kliknięciu.

Nie zmuszaj użytkownika do "rozgryzania" interfejsu.

---

# 3. KOLORY

## GŁÓWNA PALETA

Dominującą estetyką jest ciemna, chłodna paleta grafitowo-szara.

Nie używaj czystego #000000 jako głównego tła.

Bazowa paleta:

Background:
#0B0D0C

Background elevated:
#101311

Surface:
#151917

Surface elevated:
#1A1F1C

Surface strong:
#202622

Border subtle:
rgba(255,255,255,0.06)

Border default:
rgba(255,255,255,0.09)

Border strong:
rgba(255,255,255,0.14)

Primary text:
#F1F4F2

Secondary text:
#A8B0AB

Muted text:
#737B76

Disabled text:
#505752

---

# 4. EMERUS GREEN

Kolorem akcentowym systemu jest elegancka, butelkowa zieleń.

Nie używaj agresywnej neonowej zieleni jako podstawowego koloru.

Primary:

#1F6B4F

Primary hover:

#287D5D

Primary active:

#18543E

Primary subtle:

rgba(31,107,79,0.14)

Primary glow:

rgba(31,107,79,0.22)

Jasny akcent:

#55A982

Najważniejszy kolor CTA powinien być zielony, ale zielony NIE powinien znajdować się wszędzie.

Zielony oznacza:

* działanie,
* aktywny stan,
* sukces,
* potwierdzenie,
* wybrany element,
* najważniejsze CTA,
* pozytywną informację.

Nie używaj zieleni jako dekoracji każdego elementu.

Jeżeli wszystko jest zielone, nic nie jest ważne.

---

# 5. KOLORY STATUSÓW

Success:

#4FAF7B

Warning:

#C49A52

Danger:

#C96B6B

Info:

#6F96B5

Każdy status powinien posiadać:

* kolor,
* ikonę,
* tekstowy opis,

gdy sam kolor mógłby być niewystarczający.

---

# 6. KONTRAST

Nie buduj interfejsu wyłącznie przez różne odcienie szarości.

Hierarchię twórz jednocześnie poprzez:

* luminancję,
* wielkość,
* weight,
* spacing,
* pozycję,
* grupowanie,
* border,
* background,
* kolor akcentowy.

Elementy nie mogą zlewać się ze sobą.

Jeżeli dwa sąsiadujące elementy mają podobny kolor i podobną strukturę, dodaj:

* większy spacing,
* subtelny border,
* zmianę surface level,
* separator,
* wyraźniejszą hierarchię typograficzną.

---

# 7. SURFACE SYSTEM

EmerusUI wykorzystuje system warstw.

Nie używaj jednego koloru panelu dla całego UI.

Poziomy:

Level 0:
główne tło

Level 1:
duże powierzchnie / sekcje

Level 2:
karty

Level 3:
dropdowny / modale / popovery

Level 4:
elementy najwyżej położone w hierarchii

Każda kolejna warstwa powinna być subtelnie jaśniejsza lub wyraźniej odseparowana.

Nie przesadzaj z różnicą.

---

# 8. GLASS / FROSTED SURFACES

Glassmorphism stosuj oszczędnie.

Panel może mieć:

background:
rgba(20,25,22,0.78)

backdrop-filter:
blur(18px)

border:
1px solid rgba(255,255,255,0.07)

Jednak:

BLUR NIE MOŻE ZASTĘPOWAĆ HIERARCHII.

Nie stosuj blur do każdego elementu.

Glass powinien być używany głównie dla:

* overlay,
* modal,
* floating panel,
* navigation,
* dropdown,
* context menu,
* elementów znajdujących się nad inną zawartością.

---

# 9. BORDER SYSTEM

Bordery powinny być bardzo subtelne.

Nigdy nie używaj grubych jasnych obramowań bez powodu.

Standard:

1px solid rgba(255,255,255,0.06)

Hover:

rgba(255,255,255,0.10)

Active:

rgba(31,107,79,0.55)

Focus:

0 0 0 2px rgba(31,107,79,0.25)

Border ma przede wszystkim separować elementy.

Nie ma być dekoracją.

---

# 10. RADIUS

Używaj ograniczonego zestawu radiusów.

Small:
6px

Medium:
8px

Large:
12px

XL:
16px

Modal:
20px

Nie mieszaj losowych wartości.

Elementy należące do tej samej kategorii powinny mieć taki sam radius.

---

# 11. SPACING SYSTEM

Stosuj konsekwentny system spacingu:

4
8
12
16
20
24
32
40
48
64

Unikaj losowych wartości typu:

13px,
17px,
23px,
27px,

chyba że jest ku temu konkretny powód.

Najczęściej:

8px — tight relationship

12px — related content

16px — normal relationship

24px — group separation

32px — section separation

48px+ — major layout separation

Whitespace jest elementem hierarchii.

Nie wypełniaj pustej przestrzeni tylko dlatego, że "jest pusta".

---

# 12. GRID I ALIGNMENT

Elementy powinny mieć logiczne punkty wyrównania.

Preferuj:

* wspólny left edge,
* wspólną baseline,
* powtarzalne kolumny,
* przewidywalne marginesy.

Nie ustawiaj elementów "na oko".

Cały interfejs powinien sprawiać wrażenie precyzyjnie zaprojektowanego.

---

# 13. TYPOGRAFIA

Typografia ma być nowoczesna i neutralna.

Preferuj fonty w stylu:

Inter,
Geist,
Manrope,
SF Pro-like sans,
system-ui.

Nie używaj futurystycznych fontów jako głównego fontu.

Hierarchia:

Display:
32–48px

Page title:
24–32px

Section title:
18–22px

Card title:
15–17px

Body:
14–16px

Secondary:
13–14px

Metadata:
11–13px

Micro:
10–11px

Body text powinien być bardzo łatwy do czytania.

Nie używaj uppercase dla dużych ilości tekstu.

Uppercase stosuj głównie do:

* kategorii,
* metadata,
* małych labeli,
* statusów.

---

# 14. FONT WEIGHT

Nie używaj bold wszędzie.

Najczęściej:

400 — body

500 — labels

600 — headings / important values

700 — wyjątkowo ważne elementy

Hierarchia ma wynikać z różnicy.

---

# 15. BUTTON SYSTEM

Każdy przycisk musi wyglądać jak przycisk.

Primary:

ciemna lub średnia butelkowa zieleń,
jasny tekst,
subtelny border,
delikatny shadow.

Hover:

minimalne rozjaśnienie,
subtelne zwiększenie contrast,
opcjonalnie bardzo delikatny glow.

Active:

lekko ciemniejszy,
minimalnie mniejszy wizualnie.

Disabled:

niski contrast,
brak glow,
pointer-events disabled.

Nie twórz gigantycznych przycisków bez potrzeby.

CTA powinno być łatwe do znalezienia.

---

# 16. BUTTON HIERARCHY

Primary:
najważniejsza akcja.

Secondary:
alternatywna akcja.

Tertiary:
akcja drugorzędna.

Ghost:
minimalna akcja.

Danger:
destrukcyjna akcja.

Nie pokazuj 4 przycisków jako primary.

Na jednym ekranie użytkownik powinien jasno widzieć:

"To jest główna rzecz, którą mogę tutaj zrobić."

---

# 17. INPUT

Input powinien wyglądać na interaktywny.

Default:

background:
#111512

border:
rgba(255,255,255,0.08)

height:
40–44px

radius:
8px

padding:
12–14px

Placeholder:

#6F7772

Focus:

border:
Emerus green

subtle green focus ring.

Input focus nie powinien powodować agresywnego świecenia.

---

# 18. INPUT STATES

Każdy input musi mieć:

default
hover
focus
filled
disabled
error
success

Error:

border + icon + message.

Nie polegaj tylko na czerwonej ramce.

Komunikat błędu powinien powiedzieć:

* co jest nie tak,
* jak to naprawić.

Zamiast:

"Invalid input"

preferuj:

"Username must contain at least 3 characters."

---

# 19. LABELS

Label zawsze powinien być jasno związany z inputem.

Nie polegaj wyłącznie na placeholderze jako labelu.

Przykład:

Username

[ Damian______ ]

Placeholder może pokazywać przykład, ale nie powinien zastępować nazwy pola.

---

# 20. SELECT / DROPDOWN

Select powinien wyglądać jak input, ale posiadać wyraźny affordance.

Chevron po prawej.

Dropdown powinien pojawiać się bezpośrednio przy elemencie.

Dropdown:

* elevated surface,
* subtle border,
* shadow,
* radius 10–12px,
* odpowiedni padding.

Opcje:

normal
hover
selected
disabled.

Selected option może posiadać:

* check icon,
* subtelne tło,
* Emerus green accent.

Nie podświetlaj całego dropdownu agresywną zielenią.

---

# 21. CHECKBOX

Checkbox powinien być prosty.

Unchecked:

dark surface + subtle border.

Hover:

border becomes clearer.

Checked:

Emerus green background + white check.

Focus:

green focus ring.

Animation:

checkmark pojawia się subtelnie.

Checkbox powinien mieć odpowiednio duży obszar klikalny.

Nie zmuszaj użytkownika do kliknięcia dokładnie w mały kwadrat.

---

# 22. RADIO BUTTON

Radio button powinien jasno pokazywać jeden aktywny wybór.

Inactive:
subtle border.

Active:
Emerus green outer ring + inner dot.

Cały label powinien być klikalny.

---

# 23. TOGGLE / SWITCH

Toggle służy do ustawień typu:

ON / OFF.

Nie używaj go do wyboru spośród wielu opcji.

OFF:
dark surface.

ON:
Emerus green.

Thumb powinien płynnie przesuwać się podczas animacji.

Animacja:

150–220ms.

---

# 24. SLIDER

Track:

ciemny.

Active track:

Emerus green.

Thumb:

jasny / neutralny.

Hover:

subtle scale.

Nie używaj slidera, jeżeli użytkownik powinien wpisać dokładną wartość.

---

# 25. TABS

Tabs mają być bardzo czytelne.

Nie stosuj ciężkich kart jako tabów.

Preferowana forma:

tekst + subtelny active indicator.

Active:

Emerus green text lub neutralny tekst + green indicator.

Inactive:

muted text.

Hover:

secondary text.

Transition:
150–200ms.

---

# 26. CARDS

Karta nie powinna być automatycznie obramowana z każdej strony.

Może być oddzielona przez:

* surface,
* spacing,
* subtle border,
* shadow.

Card hierarchy:

Title

Supporting information

Primary content

Optional metadata

Optional action

Nie wrzucaj wszystkiego do jednej karty.

---

# 27. PANELS

Duże panele powinny mieć wyraźną strukturę:

Header

Content

Optional footer/actions

Header powinien jasno określać:

CO znajduje się w panelu.

Nie używaj paneli bez potrzeby.

Jeżeli dwie sekcje można rozdzielić samym spacingiem, nie twórz dwóch ciężkich paneli.

---

# 28. MODALS

Modal powinien skupiać uwagę użytkownika.

Overlay:

rgba(0,0,0,0.60)

Modal:

dark elevated surface

blur za modalem

radius:
16–20px

padding:
24px

Struktura:

Title

Description

Content

Actions

Primary action po prawej.

Modal nie może wyglądać jak osobna aplikacja.

Musi należeć do tego samego design systemu.

---

# 29. DRAWERS

Drawer służy do pokazania dodatkowych informacji bez opuszczania obecnego kontekstu.

Powinien:

* pojawić się płynnie,
* zachować kontekst,
* posiadać wyraźny header,
* mieć close button,
* mieć odpowiedni scroll.

---

# 30. TOOLTIP

Tooltip tylko wtedy, gdy naprawdę pomaga.

Nie używaj tooltipów do informacji, które powinny być widoczne.

Tooltip:

* mały,
* ciemny,
* kontrastowy,
* radius 6–8px,
* subtle shadow,
* krótki tekst.

---

# 31. BADGES

Badge powinien być mały.

Przykłady:

ONLINE
NEW
BETA
PREMIUM
ADMIN

Nie rób z badge'a dużego kolorowego przycisku.

Status badge może używać bardzo subtelnego zielonego tła:

rgba(31,107,79,0.14)

---

# 32. ICONS

Ikony:

* proste,
* outline,
* geometryczne,
* spójne,
* najlepiej 1.5–2px stroke.

Preferowane rozmiary:

14px
16px
18px
20px
24px

Nie mieszaj różnych stylów ikon.

Każda ikona powinna mieć sens.

Nie używaj ikony tylko jako dekoracji.

---

# 33. NAVIGATION

Navigation ma być spokojna i przewidywalna.

Active item:

subtelna surface,
Emerus accent,
opcjonalnie mały indicator.

Nie rób ogromnych świecących active states.

Sidebar:

logo / brand

primary navigation

secondary navigation

utility section

profile / settings

Elementy powinny być logicznie pogrupowane.

---

# 34. DASHBOARD

Dashboard powinien odpowiadać na pytanie:

"Co muszę teraz wiedzieć?"

Nie pokazuj wszystkiego jednocześnie.

Najważniejsze informacje:

największe,

najbardziej kontrastowe,

najwyżej.

Drugorzędne:

mniejsze,

bardziej stonowane.

Metadata:

najmniejsze.

---

# 35. EMPTY STATES

Empty state NIE jest błędem projektowym.

Powinien powiedzieć:

1. co tutaj powinno się znajdować,
2. dlaczego aktualnie niczego nie ma,
3. co użytkownik może zrobić.

Przykład:

"No servers yet"

"Create your first server to get started."

[ Create server ]

---

# 36. LOADING STATES

Unikaj pustego ekranu.

Używaj:

* skeleton,
* progress,
* spinner tylko dla krótkich operacji.

Skeleton powinien przypominać strukturę finalnego UI.

Nie pokazuj spinnera przez kilka sekund bez informacji.

---

# 37. ERROR STATES

Error powinien być:

widoczny,
zrozumiały,
naprawialny.

Nie:

"Something went wrong."

Lepiej:

"Unable to connect to the server."

"Check your connection and try again."

[ Retry ]

---

# 38. TOAST / NOTIFICATIONS

Toast powinien być dyskretny.

Pojawia się w przewidywalnym miejscu.

Success:
zielony akcent.

Warning:
amber.

Error:
red.

Info:
blue/neutral.

Nie pokazuj toastów za każdą drobną akcję.

---

# 39. SEARCH

Search musi być łatwy do znalezienia.

Input powinien posiadać:

search icon

placeholder

clear action

opcjonalny keyboard shortcut.

Na desktop:

Ctrl/Cmd + K

może otwierać global search.

---

# 40. TABLES

Tabela musi być łatwa do skanowania.

Nie obramowuj każdej komórki.

Preferuj:

* spacing,
* subtle row separators,
* hover surface.

Header:

small,
muted,
uppercase lub semibold.

Important values:

primary text.

Secondary data:

muted.

---

# 41. LISTS

Lista powinna mieć rytm.

Każdy element:

icon / avatar

primary text

secondary information

optional metadata

action

Nie upychaj 10 informacji w jednej linii.

---

# 42. AVATARS

Avatar może posiadać:

online indicator,
status,
badge.

Online indicator powinien być mały.

Nie używaj ogromnych kolorowych ringów.

---

# 43. PROGRESS

Progress bar:

track:
dark

progress:
Emerus green

radius:
full

Ważny jest również tekst:

"72%"

jeżeli dokładna wartość jest istotna.

---

# 44. GAME UI / IN-GAME PANELS

EmerusUI powinien działać również w środowisku gry.

Panel w grze może być bardziej cinematic.

Jednak zasady UX pozostają identyczne.

Przykładowo:

dark translucent surface

subtle glass

Emerus green accent

minimal border

clear typography

soft shadow

Wszystko powinno wyglądać jak część jednego ekosystemu.

---

# 45. MINECRAFT LAUNCHER

Launcher powinien korzystać z tej samej biblioteki komponentów.

Przykładowa struktura:

Sidebar

Profile

Minecraft version

Instance selector

Play CTA

News

Server status

Friends

Downloads

Settings

Play button może być największym CTA.

Nie przesadzaj z ilością informacji na ekranie głównym.

Launcher ma przede wszystkim umożliwiać:

OPEN → UNDERSTAND → PLAY

---

# 46. WEBSITE

Website może być bardziej przestronny.

Więcej whitespace.

Większe typography.

Większe visual hierarchy.

Ale komponenty powinny nadal wyglądać jak EmerusUI.

Button na stronie i button w launcherze powinny być rozpoznawalnie tym samym komponentem.

---

# 47. MOBILE / PHONE UI

Mobile musi być bardziej kompaktowe.

Nie kopiuj desktopowego layoutu.

Przemyśl:

bottom navigation

thumb reach

larger touch targets

swipe interactions

mobile drawers

mobile sheets

Minimalny komfortowy obszar interakcji:

około 44px.

Nie zmniejszaj przycisków tylko po to, aby zmieścić więcej treści.

---

# 48. RESPONSIVE DESIGN

System musi działać na:

mobile

tablet

desktop

ultrawide

in-game overlay

Nie skaluj wszystkiego liniowo.

Zmienia się:

layout,
spacing,
navigation,
content density,
interaction model.

---

# 49. ANIMACJE

Animacje EmerusUI mają być:

subtle

fast

smooth

purposeful

Nie mogą wyglądać jak showcase CSS.

Najczęściej:

120–180ms — micro interactions

180–240ms — component transitions

240–350ms — panels/modals

300–450ms — larger layout transitions

---

# 50. EASING

Preferuj:

ease-out

cubic-bezier(0.22, 1, 0.36, 1)

dla wejścia elementów.

Element powinien:

szybko rozpocząć ruch,

a następnie delikatnie wyhamować.

---

# 51. HOVER

Hover nie powinien powodować gwałtownych zmian.

Może zmienić:

background

border

text brightness

icon brightness

shadow

minimalnie scale.

Nie rób:

scale(1.1)

na normalnych buttonach.

---

# 52. CLICK / ACTIVE

Po kliknięciu użytkownik musi dostać natychmiastowy feedback.

Może to być:

subtle scale

surface change

color change

icon animation

loading state.

Użytkownik nigdy nie powinien zastanawiać się:

"Czy kliknięcie zadziałało?"

---

# 53. PAGE TRANSITIONS

Przejścia między ekranami powinny być krótkie.

Preferuj:

fade + subtle translate

lub

crossfade.

Nie używaj dużych slide animations jako domyślnej nawigacji desktopowej.

---

# 54. REDUCED MOTION

Respektuj:

prefers-reduced-motion.

W takim przypadku ogranicz:

scale,
blur animations,
large transforms,
parallax.

---

# 55. MICROINTERACTIONS

EmerusUI powinien posiadać bardzo subtelne microinteractions.

Przykłady:

checkbox:
checkmark animuje się.

toggle:
thumb przesuwa się.

button:
subtle press.

dropdown:
fade + translateY(4px).

modal:
opacity + translateY(8px).

toast:
fade + translateY(8px).

tooltip:
fade + scale(0.98 → 1).

To są małe szczegóły, ale właśnie one budują feeling premium.

---

# 56. GLOW

Glow jest elementem specjalnym.

Używaj go bardzo rzadko.

Emerus green glow może pojawić się przy:

* aktywnym CTA,
* selected item,
* success,
* focus,
* ważnym interactive element.

Nigdy nie używaj green glow jako stałego efektu całego interfejsu.

EmerusUI nie jest neon cyberpunk.

---

# 57. SHADOWS

Cienie mają być miękkie i głębokie.

Nie używaj mocnych czarnych cieni.

Preferuj:

soft multi-layer shadows.

Shadow ma budować:

depth

layering

separation.

---

# 58. VISUAL DENSITY

EmerusUI powinien być "dense enough to be useful, spacious enough to breathe."

Nie twórz:

* ogromnych pustych kart,
* zbyt dużych paddingów,
* niepotrzebnych separatorów.

Ale również nie twórz:

* tabel pełnych tekstu,
* wszystkiego upchniętego obok siebie,
* 10 informacji w jednym cardzie.

---

# 59. INFORMATION HIERARCHY

Każdy ekran powinien posiadać:

PRIMARY

SECONDARY

TERTIARY

PRIMARY:
co użytkownik ma zauważyć jako pierwsze.

SECONDARY:
co powinien zobaczyć później.

TERTIARY:
metadata i informacje pomocnicze.

Nie dawaj wszystkim elementom tej samej wizualnej ważności.

---

# 60. COPYWRITING W UI

Teksty powinny być:

krótkie,

konkretne,

naturalne,

zrozumiałe.

Unikaj:

"Proceed with operation"

Preferuj:

"Continue"

Unikaj:

"An error has occurred during authentication."

Preferuj:

"Couldn't sign you in."

CTA powinno mówić CO SIĘ STANIE.

---

# 61. ACCESSIBILITY

Nie polegaj wyłącznie na kolorze.

Każdy ważny stan powinien być możliwy do rozpoznania przez:

* tekst,
* ikonę,
* pozycję,
* shape,
* kolor.

Keyboard navigation musi być możliwa.

Focus state musi być widoczny.

---

# 62. TOUCH

W aplikacjach mobilnych i in-game UI:

interactive areas powinny być odpowiednio duże.

Nie projektuj małych elementów, które wymagają precyzyjnego kliknięcia.

Klikalny powinien być cały logiczny obszar.

---

# 63. CONSISTENCY

Jeżeli komponent występuje 30 razy:

MUSI zachowywać się tak samo w każdym miejscu.

Ten sam:

radius

padding

font

icon size

hover

focus

animation

color

behavior

Nie twórz 15 wariantów tego samego buttona bez potrzeby.

---

# 64. DESIGN TOKENS

Implementuj EmerusUI jako system tokenów.

Przykładowo:

--emerus-bg-0
--emerus-bg-1
--emerus-bg-2
--emerus-surface
--emerus-surface-hover
--emerus-border
--emerus-border-strong

--emerus-primary
--emerus-primary-hover
--emerus-primary-active
--emerus-primary-subtle

--emerus-text
--emerus-text-secondary
--emerus-text-muted

--emerus-success
--emerus-warning
--emerus-danger
--emerus-info

--emerus-radius-sm
--emerus-radius-md
--emerus-radius-lg
--emerus-radius-xl

--emerus-space-1
--emerus-space-2
--emerus-space-3
...

Dzięki temu cały ekosystem może korzystać z jednej biblioteki wizualnej.

---

# 65. COMPONENT ARCHITECTURE

Zaprojektuj system komponentów jako:

Foundation

→ colors
→ typography
→ spacing
→ radius
→ shadows
→ motion

Primitives

→ Button
→ Input
→ Select
→ Checkbox
→ Radio
→ Toggle
→ Badge
→ Icon
→ Avatar
→ Tooltip

Components

→ Card
→ Dropdown
→ Modal
→ Drawer
→ Tabs
→ Toast
→ Table
→ Navigation
→ Search

Patterns

→ Login
→ Settings
→ Dashboard
→ Profile
→ Inventory
→ Store
→ Server list
→ Launcher home
→ Game tablet
→ Admin panel

Pages

→ Dashboard
→ Launcher
→ Store
→ Profile
→ Settings
→ Game UI
→ Website

---

# 66. RESPONSIBILITY OF THE DESIGNER

Przed zaprojektowaniem każdego ekranu odpowiedz sobie:

1. Co użytkownik chce tutaj zrobić?
2. Co jest najważniejszą informacją?
3. Co jest drugorzędne?
4. Jaki jest główny CTA?
5. Czy użytkownik rozumie ten ekran bez instrukcji?
6. Czy elementy są wystarczająco od siebie oddzielone?
7. Czy hierarchia jest oczywista?
8. Czy coś jest niepotrzebne?
9. Czy interakcje mają jasny feedback?
10. Czy ten ekran wygląda jak część EmerusUI?

---

# 67. NIE RÓB TEGO

NIGDY nie twórz:

* przypadkowych gradientów,
* neonowego green everywhere,
* ogromnych glowów,
* ciężkich borders,
* przypadkowych radiusów,
* losowych spacingów,
* ogromnej ilości cards,
* tekstu bez hierarchii,
* ikon bez znaczenia,
* animacji dla samej animacji,
* wszystkiego centered,
* wszystkiego uppercase,
* wszystkiego bold,
* ogromnych pustych przestrzeni,
* paneli wewnątrz paneli bez potrzeby,
* 5 primary buttons,
* elementów wyglądających jak klikane, jeśli nie są klikane,
* elementów klikalnych bez odpowiedniego feedbacku.

---

# 68. ZASADA "ONE SYSTEM"

Launcher Minecraft,

website,

mobile app,

in-game tablet,

admin dashboard,

game menu

mają wyglądać jak różne produkty należące do **jednego ekosystemu Emerus**.

Użytkownik powinien móc przejść:

Website
→ Launcher
→ Mobile
→ In-game UI

i natychmiast rozpoznać wspólny język wizualny.

Nie oznacza to identycznych layoutów.

Oznacza:

identyczne DNA.

---

# 69. PREMIUM FEEL

Premium feeling nie pochodzi z dodawania efektów.

Powstaje z:

* perfekcyjnego spacingu,
* dobrego typography,
* odpowiedniego contrastu,
* konsekwentnych komponentów,
* subtelnych shadows,
* dobrze dobranych powierzchni,
* płynnych animacji,
* dobrego copy,
* logicznego UX,
* braku chaosu.

EmerusUI powinien sprawiać wrażenie:

**quiet luxury for digital interfaces.**

Nie krzyczy:

"LOOK HOW COOL I AM."

Zamiast tego komunikuje:

"Everything is intentional."

---

# 70. FINAL QUALITY CHECK

Przed uznaniem ekranu za ukończony wykonaj wizualny i UX audit.

Sprawdź:

### Hierarchy

Czy od razu wiadomo, co jest najważniejsze?

### Spacing

Czy spacing jest konsekwentny?

### Alignment

Czy elementy są idealnie wyrównane?

### Contrast

Czy wszystko jest czytelne?

### Separation

Czy elementy się nie zlewają?

### Interaction

Czy wiadomo, co jest klikalne?

### Feedback

Czy każda akcja ma odpowiednią reakcję?

### States

Czy komponent posiada wszystkie potrzebne stany?

### Motion

Czy animacje są subtelne i użyteczne?

### Consistency

Czy komponent wygląda tak samo jak jego odpowiedniki?

### Accessibility

Czy interfejs jest czytelny również bez polegania na kolorze?

### Responsiveness

Czy działa na mobile, desktopie i w grze?

### Polish

Czy istnieje jakiś mały szczegół, który sprawia, że interfejs wygląda tanio?

Jeżeli tak — popraw go.

---

# 71. OSTATECZNA ZASADA EMERUSUI

Nie projektuj UI tak, aby wyglądało dobrze na screenshotcie.

Projektuj je tak, aby:

**wyglądało świetnie, działało świetnie i było oczywiste w użyciu.**

EmerusUI powinien łączyć:

UXPeak-style design judgment
+
premium dark interface
+
bottle-green identity
+
cinematic atmosphere
+
professional product design
+
consistent design system
+
subtle motion
+
high information clarity.

Rezultat powinien wyglądać jak dojrzały, komercyjny produkt klasy premium, a nie jak template z marketplace'u.

**EMERUSUI = CLARITY × CONSISTENCY × POLISH × CHARACTER**
