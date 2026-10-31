# Radio Crash Desktop 1.1.0

31. listopada 2026.

- VU sada zadano prikazuje stvarni digitalni nivo, bez prethodnog pomaka od +18 dB. Skala do +18 ostaje, a studijska kalibracija dostupna je u postavkama.
- RMS odziv smiren je 50 ms integracijom, uz brz i neovisan peak hold.
- Ispravljena je normalizacija analizatora spektra i poravnanje njegove dB skale. Svih deset frekvencijskih pojaseva provjereno je sintetičkim tonovima.
- Lijevi klik na lijevi gornji vijak izravno minimizira player u tray s L/R instrumentima. Klik na tray ikonu vraća prozor.
- Dodana je automatska provjera novih verzija, prozor s obavijesti i gumb za preuzimanje. Ručna provjera nalazi se u Settings i tray izborniku.
- Ispravljen je natpis godine u SINCE 1986. Postojeći HD Radio iluminator ostaje nepromijenjen.

## Važno za korisnike 1.0.0

Verzija 1.0.0 nema provjeru ažuriranja. Preuzmite i jednom ručno instalirajte 1.1.0. Nakon toga aplikacija će obavještavati o budućim stabilnim izdanjima. Postavke ostaju spremljene. Ažuriranje se preuzima iz preglednika; aplikacija se ne zamjenjuje automatski.

## Provjere i ograničenja

Deterministički testovi provjeravaju RMS i FFT kalibraciju, sve frekvencijske pojaseve, brzinu odziva, klik na vijak pri različitim veličinama te HTTP/ETag provjeru ažuriranja. Windows reprodukcija i dalje nije ručno provjerena; Windows EXE nije Authenticode potpisan. Podržani su izravni AAC-LC i MP3 streamovi. HD Radio je vlastita oznaka Radio Crash kvalitete, ne HD Radio/IBOC dekoder.

## English

Digital VU levels no longer include an implicit +18 dB offset. A fast 50 ms RMS integrator improves ballistics, while sample peak hold remains independent. Spectrum normalization now reports stereo band RMS dBFS. Left-clicking the upper-left screw minimizes directly to tray. Stable-release update notifications are available in the app, settings and tray. The station inscription is SINCE 1986; the existing HD Radio illuminator is unchanged. Version 1.0 users must install 1.1 once manually before future in-app notices can work. Windows playback has not been manually verified and the Windows executable is unsigned.

---

# Radio Crash Desktop 1.0.0

Prvo izdanje · 30. listopada 2026.

## Što donosi

- Izvorni desktop player za Radio Crash, s crnim brušenim aluminijem, staklenim VFD prikazom i originalnim bijelim logotipom.
- Reprodukcija AAC-LC i MP3 streamova, Play/Stop, Power, glasnoća i Mute.
- Neovisni stereo VU instrumenti i 10-pojasni analizator spektra pokretani stvarnim zvukom. VU skala −40 do +18 dB, s vidljivim zadržavanjem vrha; 0 dB na instrumentu odgovara −18 dBFS.
- Automatsko osvježavanje izvođača i naslova te izmjereni format, bitrate, frekvencija uzorkovanja i kanali.
- Prozor veličine 780 × 286 točaka, s promjenom veličine, manjim presetima i pamćenjem korisničkih dimenzija.
- Minimiziranje u macOS traku izbornika / Windows notification area, gdje mali L/R instrumenti nastavljaju pratiti zvuk. Klik na ikonu vraća player.
- Postavke streama i metapodataka, izlaznog uređaja, automatskog pokretanja reprodukcije, minimiziranja, VU odziva i spektra.
- Automatsko ponovno povezivanje nakon prekida mreže i spremanje postavki između pokretanja.
- Prolazni audio underrun i automatska promjena izlazne rute više se ne tretiraju kao iskopčan uređaj: postojeći stream nastavlja raditi bez nepotrebnog ponovnog spajanja.

HD RADIO / HD SOUND je vlastita Radio Crash oznaka kvalitete streama prema postavljenom bitrate pragu. Ne označava prijam HD Radio/IBOC emitiranja.

## Instalacija i korištenje

Pročitajte priloženi `INSTALL-macOS.md` ili `INSTALL-Windows.md`. Internet je potreban za reprodukciju. Prozor se pomiče povlačenjem gornjeg dijela kućišta. Desni klik na kućište otvara izbornik za minimiziranje, veličinu, postavke i izlaz. Za minimiziranje samo u tray odaberite **Minimize to tray · L/R meters**; klik na instrumente vraća player. Glasnoća se podešava povlačenjem ili kotačićem miša preko potenciometra.

## Provjere i poznata ograničenja

Mac reprodukcija i životni ciklus minimiziranja provjereni su na Apple Silicon Macu. Universal paket sadrži i Intel izvršni kod; rad na fizičkom Intel Macu nije provjeren. Deterministički DSP i ostali testovi prolaze, a MP3/AAC ponovno povezivanje provjereno je odvojeno od GUI-ja. Kratke provjere ne zamjenjuju dugotrajno slušanje na svim DAC-ovima.

Windows x64 build pripremljen je bez tvrdnje o ručnoj provjeri reprodukcije na Windows 10/11. Windows izdanje nije Authenticode potpisano. Sustav može prikazati upozorenje o nepoznatom izdavaču.

Podržani su izravni AAC-LC i MP3 streamovi. HE-AAC/SBR, HLS, autentificirani streamovi i umetnuti ICY blokovi nisu podržani u ovom izdanju. Metapodaci koriste konfigurirani JSON endpoint. Promjena audio uređaja tijekom rada i stvarni mrežni prekidi na različitim mrežama trebaju dodatnu provjeru. Jednake nazive više izlaznih uređaja nije moguće jednoznačno razlikovati.

## English

First release of the native Radio Crash receiver: photographic aluminium fascia, live stereo VU and FFT spectrum, stream metadata, volume/mute, persistent compact window dimensions, automatic reconnect and live L/R tray meters. The relative VU reference is 0 dB = −18 dBFS. The HD emblem is a station-defined stream-quality badge, not an HD Radio/IBOC decoder.

Apple Silicon playback and hidden-window tray updates have been checked locally. Intel code is included but has not been run on a physical Intel Mac. Windows playback has not been manually verified on Windows 10/11; the executable is unsigned. Supports direct AAC-LC and MP3 streams, not HE-AAC/SBR, HLS, authenticated streams or ICY-block demultiplexing. See the platform installation file and `VERIFICATION.md` for evidence and limits.
