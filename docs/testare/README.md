# Specificație de testare manuală — eSoft Facturi

## Versiunea curentă: v1.0.15 (compactă)

- `Specificatie_testare_eSoft_Facturi_v1.0.15.pdf` — documentul de livrat testerului
  (**63 pagini, 251 cazuri de test**).
- `Specificatie_testare_eSoft_Facturi_v1.0.15.html` — sursa asamblată.
- `parts-compact/` — capitolele separate, pentru editare.

### Ce s-a schimbat față de v1.0.9

Aceleași cazuri, jumătate din pagini: **126 → 63**, cu 15 cazuri în plus.

- Fiecare caz era un card cu chenar, linie de meta (Rol / Ecran / Durată) și un tabel de
  verdict cu trei celule. Cardul consuma mai mult spațiu decât testul din el. Acum e **un
  rând de tabel**: ID + prioritate + rol într-o coloană, pași, rezultat așteptat, și o
  coloană `P F B` de bifat.
- Capitolul de instalare a mediului (`01-mediu.html`) a fost scos — testerul primește
  tableta deja configurată.
- Capitol nou `19-nou.html` cu cele 15 cazuri pentru modificările din v1.0.10–v1.0.15:
  ordinea factură-înainte-de-chitanță, chitanțele care așteaptă factura, rotunjirea la 2
  zecimale, salvarea setărilor, viteza de deschidere a Setărilor, tranzacțiile de
  sincronizare și tipărirea.

Niciun caz din v1.0.9 nu a fost pierdut — verificat prin compararea ID-urilor.

## Regenerare PDF după modificări

```bash
cd docs/testare

cat parts-compact/00-head.html parts-compact/02-cfg.html parts-compact/03-aut.html \
    parts-compact/05-sin.html parts-compact/06-hom.html parts-compact/08-fac.html \
    parts-compact/09-lst.html parts-compact/11-prn.html parts-compact/12-cer.html \
    parts-compact/13-chi.html parts-compact/14-cen.html parts-compact/15-set.html \
    parts-compact/16-off.html parts-compact/17-rec.html parts-compact/18-reg.html \
    parts-compact/19-nou.html parts-compact/20-anexe.html \
    > Specificatie_testare_eSoft_Facturi_v1.0.15.html

"C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe" \
  --headless --disable-gpu --no-pdf-header-footer --virtual-time-budget=8000 \
  --print-to-pdf="<cale absolută>\Specificatie_testare_eSoft_Facturi_v1.0.15.pdf" \
  "file:///<cale absolută>/Specificatie_testare_eSoft_Facturi_v1.0.15.html"
```

`parts-compact/00-head.html` conține stilurile. Capitolele separate **nu au stiluri proprii**
— dacă vrei să previzualizezi unul singur, lipește-i întâi blocul `<style>` din head, altfel
se randează ca text simplu.

## Versiunea veche

`Specificatie_testare_eSoft_Facturi_v1.0.9.*` și `parts/` rămân pentru referință. Dacă
v1.0.15 e acceptată, se pot șterge.
