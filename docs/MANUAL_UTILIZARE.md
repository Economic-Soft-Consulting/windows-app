# Manual de Utilizare - eSoft Facturi v0.6.8

## Cuprins

1. [Introducere](#introducere)
2. [Autentificare](#autentificare)
3. [Panou Principal (Dashboard)](#panou-principal)
4. [Crearea unei Facturi](#crearea-unei-facturi)
5. [Gestionarea Facturilor](#gestionarea-facturilor)
6. [Date / Parteneri](#date--parteneri)
7. [Setări (Administrator)](#setări-administrator)
8. [Diferențe Agent vs Administrator](#diferențe-agent-vs-administrator)

---

## Introducere

**eSoft Facturi** este o aplicație desktop pentru gestionarea și emiterea facturilor. Aplicația funcționează offline și sincronizează datele cu serverul WinMentor când există conexiune la internet.

### Cerințe sistem
- Windows 10/11 (64-bit)
- Conexiune la internet pentru sincronizare
- Imprimantă termală 80mm (opțional, pentru printare)

---

## Autentificare

La pornirea aplicației, se afișează ecranul de autentificare cu două opțiuni:

### Agent
- **Nu necesită parolă** - click pe „Agent" și apoi „Continuă"
- Acces la: Dashboard, Facturi, Date/Parteneri
- Vede doar facturile din ziua curentă
- Sesiunea rămâne salvată

### Administrator
- **Necesită parolă** - click pe „Administrator" și introduceți parola
- Acces complet la toate funcționalitățile
- Vede toate facturile (istoric complet)
- Acces la Setări
- Sesiunea NU rămâne salvată (reconectare necesară)

---

## Panou Principal

Dashboard-ul afișează un sumar al activității:

### Statistici afișate
| Statistică | Agent | Administrator |
|------------|-------|---------------|
| Facturi | Doar azi | Total toate |
| În Așteptare | Azi | Total |
| Trimise | Azi | Total |
| Eșuate | Azi | Total |

### Totaluri Financiare
- **Fără TVA** - suma totală fără TVA
- **Cu TVA** - suma totală cu TVA (19%)
- **Volum Doc.** - numărul total de articole vândute

### Navigare Rapidă
- **Factură Nouă** - buton mare pentru creare rapidă
- **Facturi** - lista și istoricul facturilor
- **Date/Parteneri** - căutare clienți și produse
- **Configurare** - setări (doar Administrator)
- **Web eSoft** - deschide site-ul companiei

---

## Crearea unei Facturi

Procesul de creare a unei facturi are **4 pași**:

### Pasul 1: Selectare Partener
1. Căutați partenerul după nume, CUI sau cod
2. Click pe partenerul dorit din listă
3. Se afișează detalii: CUI, Reg. Com., Nr. Locații

### Pasul 2: Selectare Locație
1. Se afișează toate locațiile partenerului selectat
2. Puteți filtra locațiile după nume sau adresă
3. Click pe locația unde se livrează factura
4. Se afișează: Județ, Localitate, Adresă completă

### Pasul 3: Adăugare Produse
1. Căutați produse după nume sau cod
2. Specificați cantitatea pentru fiecare produs
3. Produsele se adaugă în coș (lista din dreapta)
4. Puteți modifica cantitățile sau șterge articole
5. Se afișează totalul curent al facturii

### Pasul 4: Revizuire și Confirmare
1. Verificați toate detaliile facturii:
   - Partener și locație
   - Lista articolelor cu prețuri
   - Total fără TVA și cu TVA
2. Opțional: adăugați observații
3. Click **„Creează Factură"** pentru finalizare

### După creare
- Factura se salvează local cu status „În așteptare"
- Se poate printa imediat
- Se sincronizează automat când există conexiune

---

## Gestionarea Facturilor

### Lista Facturilor
Pagina Facturi afișează toate facturile. Puteți:

- **Filtra** după status: Toate, În așteptare, Trimise, Eșuate
- **Căuta** după partener sau număr
- **Schimba vizualizarea**: Tabel sau Grid

### Statusuri Facturi
| Status | Descriere | Culoare |
|--------|-----------|---------|
| În așteptare | Nesincronizată | Galben |
| Se trimite | În curs de sincronizare | Albastru |
| Trimisă | Confirmată în WinMentor | Verde |
| Eșuată | Eroare la sincronizare | Roșu |

### Acțiuni disponibile (meniu 3 puncte)
- **Detalii** - vizualizare completă a facturii
- **Trimite** - forțează sincronizarea (doar pt. În așteptare/Eșuate)
- **Șterge** - șterge factura (doar Administrator)

### Detalii Factură
Click pe o factură pentru a vedea:
- Informații partener și locație
- Lista completă a articolelor
- Istoric status și mesaje de eroare (dacă există)
- **Buton de printare** - în partea de jos a dialogului

### Printare Factură (Backup)
Dacă printarea automată la creare eșuează:
1. Deschideți factura (click pe ea sau „Detalii" din meniu)
2. În partea de jos a dialogului, click pe **„Printează Factura"**
3. Factura va fi trimisă la imprimanta selectată în Setări

---

## Date / Parteneri

Această pagină permite vizualizarea datelor sincronizate:

### Tab-uri disponibile
- **Parteneri** - lista tuturor clienților
- **Produse** - catalogul de produse

### Căutare Parteneri
- Căutați după: nume, CUI, cod intern/extern
- Click pe un partener pentru detalii complete

### Detalii Partener (popup)
Afișează 3 tab-uri:
1. **Info General** - date fiscale, financiare, observații
2. **Locații** - toate sediile/punctele de lucru
3. **Oferte & Prețuri** - prețuri speciale pentru acest client

### Căutare Produse
- Căutați după: nume, cod, clasă
- Vizualizați: unitate de măsură, preț, cotă TVA

---

## Setări (Administrator)

> ⚠️ **Doar pentru Administrator** - necesită autentificare cu parolă

### Secțiuni Setări

#### 1. Sincronizare Date
- **Ultima sincronizare** - data și ora ultimei actualizări
- **Status conexiune** - Online/Offline
- **Buton Sincronizează Acum** - forțează actualizarea datelor

#### 2. Configurare Agent
Setări pentru facturare:
- Nume agent
- Serie carnet facturi
- Interval numere facturi (Start - End - Curent)
- Simbol carnet livrare
- Simbol gestiune
- Nume și act identitate delegat

#### 3. Setări Printare
- **Imprimantă** - selectați din lista disponibilă
- **Număr copii** - câte exemplare să printeze
- **Printare automată** - la crearea facturii
- **Previzualizare** - afișare înainte de printare
- **Lățime hârtie** - 58mm sau 80mm

#### 4. Informații Aplicație
- Versiunea curentă
- Locația fișierelor
- Informații tehnice

---

## Diferențe Agent vs Administrator

| Funcționalitate | Agent | Administrator |
|-----------------|-------|---------------|
| Autentificare | Fără parolă | Cu parolă |
| Sesiune salvată | ✅ Da | ❌ Nu |
| Dashboard | Doar ziua curentă | Istoric complet |
| Creare facturi | ✅ Da | ✅ Da |
| Vizualizare facturi | Doar azi | Toate |
| Ștergere facturi | ❌ Nu | ✅ Da |
| Acces Setări | ❌ Nu | ✅ Da |
| Sincronizare manuală | ❌ Nu | ✅ Da |

---

## Sfaturi și Rezolvare Probleme

### Aplicația nu pornește
1. Verificați că Windows este actualizat
2. Rulați ca Administrator
3. Reinstalați aplicația

### Nu se sincronizează datele
1. Verificați conexiunea la internet
2. Status-ul trebuie să fie „Online"
3. Contactați suportul tehnic dacă problema persistă

### Printarea nu funcționează
1. Verificați că imprimanta este pornită și conectată
2. Selectați imprimanta corectă în Setări
3. Verificați lățimea hârtiei (80mm recomandat)

### Eroare la trimiterea facturii
1. Verificați datele introduse (cantități, prețuri)
2. Vedeți mesajul de eroare în detalii factură
3. Înlocuiți factura dacă este necesar

---

## Contact Suport

Pentru asistență tehnică:
- **Website**: [softconsulting.ro](https://www.softconsulting.ro)
- **Versiune**: 0.6.8
- © 2026 eSoft
