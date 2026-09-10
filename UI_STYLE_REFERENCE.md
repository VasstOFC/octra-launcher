# NoPixel-Style UI Reference

## Kolorystyka

### Tła
- Główny: `#0a0a0f` (czarny z lekkim granatem)
- Paneli: `rgba(15, 15, 25, 0.85)` z `backdrop-filter: blur(20px)`
- Karty: `rgba(20, 20, 35, 0.9)`
- Hover: `rgba(30, 30, 50, 0.95)`

### Akcenty
- Primary: `#00d4ff` (turkus/neon cyan)
- Secondary: `#7c3aed` (fiolet)
- Success: `#10b981` (zieleń)
- Warning: `#f59e0b` (żółty/pomarańczowy)
- Error: `#ef4444` (czerwony)

### Tekst
- Główny: `#ffffff`
- Secondary: `rgba(255, 255, 255, 0.6)`
- Muted: `rgba(255, 255, 255, 0.35)`

## Typografia

### Fonty
- Główny: `Inter`, `SF Pro Display`, system-ui
- Monospace (kody/logi): `JetBrains Mono`, `Fira Code`

### Rozmiary
- Nagłówek: `1.5rem` (24px), font-weight: `700`
- Podtytuł: `1.125rem` (18px), font-weight: `600`
- Body: `0.875rem` (14px), font-weight: `400`
- Caption: `0.75rem` (12px), font-weight: `500`

## Komponenty

### Panel/Modal
```css
.panel {
  background: rgba(15, 15, 25, 0.85);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  box-shadow: 
    0 0 0 1px rgba(0, 212, 255, 0.1),
    0 25px 50px -12px rgba(0, 0, 0, 0.5);
}
```

### Przycisk
```css
.btn-primary {
  background: linear-gradient(135deg, #00d4ff 0%, #7c3aed 100%);
  border: none;
  border-radius: 10px;
  padding: 10px 20px;
  font-weight: 600;
  color: white;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.4);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 10px 20px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.8);
}
```

### Input
```css
.input {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 12px 16px;
  color: white;
  transition: border-color 0.2s;
}

.input:focus {
  outline: none;
  border-color: #00d4ff;
  box-shadow: 0 0 0 3px rgba(0, 212, 255, 0.15);
}
```

### Karta (Card)
```css
.card {
  background: rgba(20, 20, 35, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  padding: 16px;
  transition: all 0.2s ease;
}

.card:hover {
  background: rgba(30, 30, 50, 0.95);
  border-color: rgba(0, 212, 255, 0.2);
  transform: translateY(-2px);
}
```

### Scrollbar
```css
::-webkit-scrollbar {
  width: 6px;
}

::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.25);
}
```

## Animacje

### Przejścia
- Otwieranie paneli: `0.3s cubic-bezier(0.4, 0, 0.2, 1)`
- Hover efekty: `0.2s ease`
- Transformacje: `0.15s ease-out`

### Efekty
- Glow: `box-shadow: 0 0 20px rgba(0, 212, 255, 0.3)`
- Pulse: `animation: pulse 2s infinite`
- Slide-in: `transform: translateX(-10px); opacity: 0` → `transform: translateX(0); opacity: 1`

## Ikony
- Styl: Outline lub Duotone
- Rozmiar: 20-24px dla nawigacji, 16px dla inline
- Grubość stroke: 1.5-2px
- Kolory: dziedziczą z kontekstu lub białe z opacity

## Spacing
- Padding mini: `8px`
- Padding small: `12px`
- Padding medium: `16px`
- Padding large: `24px`
- Gap small: `8px`
- Gap medium: `12px`
- Gap large: `16px`

## Zasady ogólne
1. **Glassmorphism** — prawie zawsze używaj `backdrop-filter: blur()`
2. **Subtelne granice** — cienkie, półprzezroczyste border-y
3. **Neonowe akcenty** — delikatne glow na hover/focus
4. **Ciemne tła** — nigdy czyste czarne, zawsze z lekkim odcieniem
5. **Animacje** — płynne, nie agresywne
6. **Ikony > tekst** — preferuj ikony nad długimi opisami
7. **Minimalizm** — mniej znaczy więcej
