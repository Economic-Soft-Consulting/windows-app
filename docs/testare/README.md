# Specificație de testare manuală — eSoft Facturi v1.0.9

- `Specificatie_testare_eSoft_Facturi_v1.0.9.pdf` — documentul de livrat testerului (126 pagini, 235 cazuri de test).
- `Specificatie_testare_eSoft_Facturi_v1.0.9.html` — sursa asamblată.
- `parts/` — capitolele separate, pentru editare.

## Regenerare PDF după modificări

```bash
cat parts/00-head.html parts/01-mediu.html parts/02-cfg.html parts/03-aut.html \
    parts/05-sin.html parts/06-hom.html parts/08-fac.html parts/09-lst.html \
    parts/11-prn.html parts/12-cer.html parts/13-chi.html parts/14-cen.html \
    parts/15-set.html parts/16-off.html parts/17-rec.html parts/18-reg.html \
    parts/20-anexe.html > Specificatie_testare_eSoft_Facturi_v1.0.9.html

"C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe" \
  --headless --disable-gpu --no-pdf-header-footer \
  --print-to-pdf="<cale absolută>\Specificatie_testare_eSoft_Facturi_v1.0.9.pdf" \
  "file:///<cale absolută>/Specificatie_testare_eSoft_Facturi_v1.0.9.html"
```

`parts/00-head.html` conține stilurile (format A4, blocuri de caz de test, priorități).
