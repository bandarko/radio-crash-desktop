# Radio Crash Desktop - Windows x64

Verzija 1.3.1 · Namijenjeno za Windows 10/11, 64-bitni x64 sustav.

1. Raspakirajte cijeli ZIP u vlastitu mapu, primjerice Documents/Radio Crash Desktop.
2. Pokrenite **RadioCrashDesktop.exe** i pritisnite Power ili Play.

Paket je prenosiv i nema instalacijski program. Postavke se spremaju u korisnički AppData direktorij. Internet je potreban za reprodukciju. Nisu priloženi vanjski dekoderi.

Ovo izdanje nije Authenticode potpisano. Windows može prikazati upozorenje o nepoznatom izdavaču. Preuzimajte samo s Radio Crash weba ili službenog GitHub repozitorija i po potrebi usporedite SHA-256 datoteke paketa. Reprodukcija na Windows 10/11 nije ručno provjerena; nemojte tumačiti uspješan build kao provjeru zvuka ili hardvera.

Kliknite postojeći **POWER** gumb: **OFF** zaustavlja radio i ostavlja aplikaciju otvorenom, a **EXIT** potpuno zatvara aplikaciju. U standby stanju isti izbornik nudi **ON / EXIT**. Možete izaći i tipkama **Alt+F4**. Minimiziranje ostavlja radio aktivnim.

Glavni receiver nema Windows naslovnu traku ni vanjski okvir. Za pomicanje povucite gornji aluminijski dio, a za veličinu koristite desni ili donji rub te donji desni kut kućišta. Isključena je dodatna Windows sjena koja uvodi mali pomak početka sadržaja. Manifest za Per-Monitor V2 DPI i zajedničke klik koordinate ostaju uključeni. Ručna potvrda na starom HP-u i novijem NVIDIA sustavu još je potrebna.

Za minimiziranje kliknite desnom tipkom na gornji dio kućišta i odaberite **Minimize to tray · L/R meters**. Klik na L/R instrumente u notification area vraća prozor; Windows ih može smjestiti u izbornik skrivenih ikona. Ctrl+M standardno minimizira u taskbar; Settings → Minimize to tray uključuje minimiziranje u tray.

Release notes su u `RELEASE-NOTES.md`. Podaci o bibliotekama i njihovim izvornim kodovima nalaze se u `THIRD_PARTY_NOTICES.md`, `DEPENDENCIES.md` i `licenses/`.

English: Extract the entire ZIP and run RadioCrashDesktop.exe. No installer is required. This executable is unsigned and Windows may warn about an unknown publisher. Windows 10/11 playback has not been manually verified. Click POWER for OFF (standby) or EXIT (close app), or ON / EXIT when in standby. The receiver has no Windows title bar or outer border. Drag the upper chassis to move it, use its rim/corner to resize, and press Alt+F4 to exit. Right-click the upper chassis to minimize to live L/R notification-area meters; click the meters to restore.

## Ažuriranja

Za korisnike verzije 1.0.0 potrebna je jedna ručna instalacija 1.1.0. Od 1.1.0 player automatski provjerava nova stabilna izdanja. Provjera je dostupna i u Settings te tray izborniku. Za minimiziranje kliknite lijevom tipkom na lijevi gornji vijak kućišta.
