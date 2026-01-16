# Resources Folder

## SumatraPDF

Pentru a include SumatraPDF în installer:

1. Descarcă SumatraPDF portable (64-bit) de la:
   https://www.sumatrapdfreader.org/download-free-pdf-viewer

2. Extrage `SumatraPDF.exe` și pune-l în acest folder (`src-tauri/resources/`)

3. La build, fișierul va fi automat inclus în installer și copiat în folder-ul aplicației

## Notă

Dacă nu incluzi SumatraPDF aici, aplicația va căuta automat în:
- Folder-ul aplicației (resources bundled)
- `%USERPROFILE%\AppData\Local\SumatraPDF\`
- `C:\Program Files\SumatraPDF\`
- `C:\Program Files (x86)\SumatraPDF\`

Utilizatorul poate instala manual SumatraPDF dacă nu e bundled, dar se recomandă să-l incluzi pentru experiență seamless.
