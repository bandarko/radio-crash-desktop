# Radio Crash Desktop 1.4.1 (local owner preview)

- Both silver skins now use the exact supplied Radio Crash logo bitmap, tinted as dark ink directly on aluminium. The photographic header is continuous bare aluminium, with no rectangular cover or logo backdrop.
- Windows chassis dragging uses nonmodal physical desktop-coordinate movement so the rendering loop can continue while the mouse is held. Mouse-up, focus loss and Escape cancel movement. Physical Windows runtime review remains pending.
- Third optional skin: Analog VU - Ivory Glass. One continuous warm ivory backplate behind shared glass, live now-playing lettering in the centre, a large round right-hand volume knob, and functional controls underneath.
- Ivory Glass instruments use a straight horizontal scale and fanned radial divisions. Needle length is fixed and every division intersects the actual needle ray; the fixed +3 VU calibration and 300 ms detector are preserved.
- Refined Analog VU - Silver Aluminium: slightly narrower instrument apertures, a wider central information display, and a smaller round volume knob.
- Both silver SETTINGS buttons have clean metal caps, with the SETTINGS label underneath and no black gear or LED.
- Skin selection, illumination, standby clock and analog sensitivity persist. VFD Digital - Black Aluminium remains the default.
- Both analog skins retain the approved fixed classical 300 ms VU detector and +3 VU scale. Playback, detector calibration and separate digital response preferences are unchanged.
- Local preview only. Publication requires owner testing and explicit approval. Windows build validation is separate from physical Windows runtime testing.

# Radio Crash Desktop 1.4.0

- New optional Analog VU - Silver Aluminium skin in Settings > Receiver skin. VFD Digital - Black Aluminium remains the default, and the selected skin is remembered.
- Natural silver brushed aluminium, warm backlit analog instruments, slim amber now-playing display, large central volume knob and SETTINGS without an LED.
- Fixed classical full-wave-average VU measurement from actual playback PCM. Independent L/R channels, 300 ms moving-coil response, constant-length circular 90-degree needle travel and a scale ending at +3 VU.
- Analog display sensitivity is calibrated separately at -8 dBFS sine RMS by default. Only its level can be adjusted. Digital RMS/Peak, decay, reference and hold preferences are preserved separately. Audio gain and stream quality are unchanged.
- Live artist/title, Croatian characters, scrolling text, stream properties, instrument illumination and standby clock in the new skin.
- Windows: correct the overly strict graphics eligibility check so capable OpenGL 3.3 hardware can use Enhanced. Preserve Classic fallback for older or unsupported graphics.
- Windows: retain the borderless receiver and use a native popup window without caption/resize frame bits. Physical alignment on the older HP still needs owner confirmation.
- Clean downloads: application, short instructions/release notes and required licenses, without development documents in the ZIP root.

Validation: 63 local macOS ARM tests and strict Clippy passed. The owner tried and approved the revised analog skin and movement. macOS Intel and Windows packages build from the shared Rust source; physical runtime on those platforms has not been verified for this release. Windows EXE is unsigned. Mac release signing/notarization and native CI are recorded in the release manifest.

# 1.3.2 candidate (not a stable release)

- Fix the overstrict OpenGL 4.1 eligibility requirement. The effects use desktop OpenGL 3.3; eframe requests that version even on an RTX 4070. The owner's actual 3.3.0 NVIDIA 596.49 context is now a regression fixture.
- Candidate HP alignment correction: remove native caption/resize frame style bits, use a true popup receiver and reapply only after native style changes. This has not yet been verified on the physical HP or RTX machine.
- Add a copyable graphics/window diagnostic report in Settings.
- Existing Classic fallback, DSP/audio and Mac appearance are retained. Do not replace the current stable release before hardware confirmation.

# Radio Crash Desktop 1.3.1

- Windows: removed the OS title bar and rectangular frame from the main receiver. The component now floats on the desktop like the Mac version.
- Windows: disabled the extra undecorated DWM shadow and its one-physical-pixel client inset. Existing DPI handling and shared pointer geometry are retained, without a guessed mouse offset.
- Move the receiver by its upper aluminium chassis, resize with its rim/corner, minimize with the upper-left screw or chassis menu, and exit with POWER > EXIT or Alt+F4. Settings keeps its compact native window.
- A native Windows hidden-window test checks that client and outer origins/sizes coincide after configuration. This does not replace testing on the physical HP or NVIDIA GPU.
- macOS: version/build update only, with the same receiver appearance and behavior. Both platforms use 1.3.1 so existing automatic update checks can discover the release correctly.
- GPU quality selection, audio, meters, spectrum, night display, clock, logo and approved HD Radio illuminator are unchanged.

# Radio Crash Desktop 1.3.0

- Adaptive GPU optics add glass scattering, actual VFD phosphor bloom and subtle brushed-metal and rotary-knob highlights on supported modern graphics.
- Automatic mode keeps the original receiver appearance on older, unknown, software and virtual graphics. Classic remains available manually.
- Enhanced graphics have bounded allocations, checked shader/framebuffer initialization, asynchronous GPU timing and automatic fallback. An interrupted enhanced graphics session starts safely in Classic on the next launch.
- Settings show the graphics selection, actual GPU and fallback reason. Changing graphics does not reconnect or process the audio.
- The small original white logo, SINCE 1986, approved HD Radio illuminator, VU detectors/hold, Night VFD, standby clock and tray meters are preserved.
- This release uses standard dynamic range output. True HDR/EDR presentation is not enabled.
- macOS Apple Silicon runtime is checked separately from Intel/Windows builds. Physical Windows GPU and playback verification is not claimed.

# Radio Crash Desktop 1.2.0

1. studenoga 2026.

- Settings > VU response sada nudi tri stvarna odziva: Hi-Fi RMS (kao u 1.1.5), Fast RMS (življi odziv) i Peak (kratki vrhovi). Izbor se pamti i odmah vrijedi za glavni instrument i tray, bez prekida reprodukcije.
- Fast RMS mijenja i vrijeme samog mjerenja, s 50 na 15 ms, uz istu kalibraciju za stalni signal. Novi korisnici dobivaju preporučeni Fast RMS. Postojeće postavke čuvaju mirniji profil ako brži odziv nije bio uključen.
- Peak hold u svakom profilu pamti maksimum ISTOG detektora kao njegova traka, dvije sekunde. Ostaje puni svjetleći segment u L/R retku s bojom svoje zone, bez pokazivača ili tanke oznake iznad traka.
- Settings > Live signal measurements odvojeno prikazuje oversampled True Peak u dBTP te momentary (400 ms) i short-term (3 s) LUFS. Vrijednosti čekaju puni prozor mjerenja; tišina se prikazuje kao nedostupna.
- Tray poštuje i Faster VU decay. Sačuvani su VU osjetljivost, glasnoća, FFT, HD Radio iluminator i izgled receivera. Nema normalizacije, kompresije ili umjetnog pojačavanja prikaza.

- Automatska provjera novih verzija uvijek je uključena, pri pokretanju i svakih šest sati. Stara spremljena opcija za isključivanje više je ne može zaustaviti.
- Prag HD oznake jasnije je označen: upravlja samo paljenjem oznake. Ne ograničava bitrate i ne smanjuje kvalitetu izvornog streama.

- Aplikacija je namijenjena samo Radio Crashu. Uklonjena su polja za promjenu stream URL-a i JSON endpointa; koriste se službeni Radio Crash izvori, uz očuvanje ostalih postavki.

- Noćni VFD: spremljiva svjetlina 25-100 %, uz Day i Night prečace. Prigušuje se samo display; aluminij, logo, gumbi i odobreni HD Radio iluminator ostaju isti.
- Standby sat: kad je reprodukcija zaustavljena, središnji VFD prikazuje veliki dot-matrix sat i lokalni datum. PLAY vraća Now Playing. Povezivanje i ponovna spajanja i dalje prikazuju svoje stvarno stanje.

## English

Three persistent VU response profiles now select genuine consumed-PCM detectors on both the receiver and tray: the original 50 ms Hi-Fi RMS, recommended 15 ms Fast RMS, and an instantaneous sample-peak envelope with controlled 20 dB / 320 ms release. Every profile holds the maximum of its own detector for two seconds, retaining the approved full zone-colored in-row filament. Selecting a profile does not restart playback or change gain. Separate advanced measurements show oversampled True Peak in dBTP and bounded momentary/short-term LUFS, with complete-window warm-up and silence handling. True Peak does not drive an RMS bar's front-panel hold. Automatic update checks are always on, including legacy configurations that disabled them. HD badge threshold wording now makes clear that it only controls illumination and never caps audio bitrate. No normalization, compression, spectrum calibration or fascia change is made. Day/Night VFD presets and a persistent 25-100% brightness control affect only display emission. A local 24-hour dot-matrix standby clock/date appears when playback is stopped and yields immediately to Now Playing on PLAY; it never hides connection errors. The product is fixed to the official Radio Crash stream and metadata endpoint; URL editors are removed and old custom URL preferences are ignored. This is not a certified meter or exact analogue hardware emulation.

Packaging and platform validation are recorded in the release manifest. Windows hardware playback remains manually unverified; Windows downloads are not Authenticode-signed.

---

# Radio Crash Desktop 1.1.5

1. studenoga 2026.

- Peak hold sada zadržava najveći nivo istog RMS detektora koji pokreće VU trake. Sirovi sample peak ostaje odvojena dijagnostička vrijednost.
- Zadržavanje traje oko dvije sekunde od novog/jednakog maksimuma, neovisno za L i R. Nakon isteka oznaka se automatski postavlja na trenutačni nivo, bez prethodnog sporog klizanja.
- Oznaka je svijetleći segment u samom retku instrumenta i koristi boju svoje zone. Uklonjeni su odvojena točka i vertikalna crta iznad trake.
- Traka i zadržani segment koriste isti prag, bez zaokruživanja u viši segment. Usklađene su i male tray oznake; tišina ostaje ugašena pri svim osjetljivostima.
- Kalibracija 0 VU = -15 dBFS, 50 ms RMS odziv, glasnoća i FFT ostaju isti. Ovo usklađuje princip peak-holda, a ne pretvara RMS VU u kopiju cijelog analognog peak-metra.

Usporedba izvora nalazi se u `METER-REFERENCE.md` u izvornom repozitoriju. Mac paket potpisan je i notariziran; Windows EXE ostaje nepotpisan i njegov rad na fizičkom računalu nije ručno provjeren u ovoj provjeri.

## English

Peak hold now captures the maximum of the same continuous RMS envelope as the VU bar, preserving sub-frame envelope maxima. Each channel holds a new/equal maximum for two seconds and then automatically resets to its current level. The marker is an in-row, zone-colored filament segment. Bar/hold thresholds and compact tray quantization are aligned, including silence at maximum sensitivity. Raw sample peaks remain diagnostics. Audio gain, the -15 dBFS reference, 50 ms RMS response and FFT are unchanged. The app keeps its RMS VU identity rather than claiming a complete emulation of an analogue peak-reading meter.

---

# Radio Crash Desktop 1.1.4

1. studenoga 2026.

- Ispravljen preslab kontrast izbornika POWER u svijetloj temi sustava. OFF, ON i EXIT sada koriste svijetli, veći i podebljani tekst na tamnoj podlozi.
- Povećane su površine stavki, a stavka pod pokazivačem dobiva jasno istaknutu tirkiznu podlogu.
- Izgled kućišta, funkcije POWER izbornika, audio, VU i provjera ažuriranja ostaju nepromijenjeni.

Mac paket je potpisan i notariziran. Windows EXE nije Authenticode potpisan; fizički Windows rad nije ručno provjeren u ovoj provjeri.

## English

POWER menu items now use an explicit dark UI palette and bright 16-point bold text, fixing dark system-theme text inherited over the custom dark popup. Rows are taller and have a distinct hover background. The receiver artwork, ON/OFF/EXIT actions, audio, DSP and updater behavior are unchanged.

---

# Radio Crash Desktop 1.1.3

1. studenoga 2026.

- Ispravljena provjera ažuriranja pri pokretanju: nova provjera obavlja se nakon pet sekundi i kad postoji rezultat spremljen prije manje od šest sati. Brza ponovna pokretanja poštuju razmak od jedne minute između provjera. Tijekom neprekidnog rada provjera ostaje svakih šest sati.
- Broj pokrenute verzije stalno je vidljiv na vrhu Settings prozora.
- Automatska provjera, gumb **Check for updates now** i rezultat provjere sada su stalno vidljivi ispod postavki. Više nisu skriveni u području koje se skrola.
- Ažuriranja se i dalje preuzimaju u pregledniku. Aplikacija se ne zamjenjuje sama.
- POWER izbornik OFF/ON/EXIT, Windows naslovna traka, VU kalibracija, audio i izgled receivera ostaju kao u 1.1.2.

Mac paket je Developer ID potpisan i notariziran. Windows EXE nije Authenticode potpisan; rad na fizičkom Windows računalu nije ručno provjeren u ovoj provjeri.

## English

Startup checks no longer wait for the previous six-hour cache period to expire. Each launch schedules an ETag check after five seconds, with a one-minute throttle for rapid relaunches; continuous sessions still check every six hours. The running version is fixed at the top of Settings, and update controls/status remain visible outside its scrollable area. Updates open the browser and do not silently install. POWER actions, Windows frame, PCM/audio, VU calibration and the receiver artwork are unchanged.

---

# Radio Crash Desktop 1.1.2

1. studenoga 2026.

- Postojeći **POWER** gumb sada otvara **OFF / EXIT** na Macu i Windowsima. OFF zaustavlja radio u standby, a EXIT potpuno zatvara aplikaciju. U standby stanju izbornik nudi **ON / EXIT**. Nema dodatnog gumba na aluminijskoj ploči.
- Windows izdanje ima standardnu naslovnu traku s gumbima za minimiziranje i **X** za izlaz. **Alt+F4** također zatvara player. Time se izbjegava prikaz bez okvira, povezan s pomakom slike i klikova na nekim starijim Intel grafičkim sustavima pod Windowsom 10.
- Windows EXE sada sadrži izričit Per-Monitor V2 DPI manifest s Per-Monitor fallbackom.
- Klik površine Power, Play/Stop, Mute i Settings obuhvaćaju cijeli vidljivi metalni gumb. Položaji prikaza i klikova koriste jedan izračun pri svim veličinama prozora.
- VU kalibracija, stvarni audio i odobreni HD Radio iluminator ostaju nepromijenjeni.

Regresijski testovi provjeravaju klikove u sredini i blizu svih rubova gumba, skale 100/125/150/200 %, promjene veličine te odvojene ON/OFF/EXIT akcije i zatvaranje izbornika bez promjene reprodukcije. To nije zamjena za ručnu provjeru na prijavljenom HP Windows 10 Pro notebooku; njegov grafički sustav nije dostupan u ovoj provjeri. Windows EXE nije Authenticode potpisan.

## English

The existing POWER switch now opens OFF / EXIT on both platforms, or ON / EXIT in standby. OFF stops playback while keeping the app open; EXIT closes the entire app. No additional exit switch is drawn on the fascia. Windows uses its standard native title bar with X / Alt+F4, avoiding the borderless OpenGL rendering/input path implicated in legacy Intel Windows 10 offset reports. An embedded manifest declares Per-Monitor V2 DPI awareness with a Per-Monitor fallback. Click targets span the visible metal buttons and share the fascia's geometry. Pointer tests cover multiple sizes, aspect ratios and scaling factors; manual confirmation on the affected HP remains pending. Audio, VU calibration and the approved HD Radio illuminator are unchanged. The Windows download remains unsigned.

---

# Radio Crash Desktop 1.1.1

1. studenoga 2026.

- Novi zadani VU profil Radio Crash Hi-Fi: 0 VU = -15 dBFS. Signal od -11 dBFS prikazuje +4, a jači signal od -7 dBFS prikazuje +8. Vrijednosti i dalje prate stvarni zvuk.
- Dodano fino podešavanje VU osjetljivosti. Glavni i tray instrumenti sada koriste potpuno istu odabranu kalibraciju.
- Stari digitalni zadani profil iz 1.1.0 jednom se prebacuje na Hi-Fi. Digital i Studio ostaju dostupni u Settings. Jednom spremljen osobni izbor više se ne mijenja pri pokretanju.
- Glasnoća, dekoder, RMS odziv, FFT, HD iluminator i natpis SINCE 1986 nisu promijenjeni ovom kalibracijom.

Mac aplikacija potpisana je Developer ID certifikatom i notarizirana. Uobičajeni prozor macOS-a koji pri prvom otvaranju traži Open za aplikaciju preuzetu s interneta može se pojaviti i kod potpisanih aplikacija. [Appleovo objašnjenje](https://support.apple.com/en-us/102445).

Windows EXE nije Authenticode potpisan. Windows reprodukcija nije ručno provjerena.

## English

The recommended VU calibration is now 0 VU = -15 dBFS, giving +4 VU for -11 dBFS programme RMS and +8 VU for -7 dBFS. Fine sensitivity adjustment is available, and tray meters share the main panel's reference. A one-time migration corrects the 1.1.0 digital default while retaining Studio/custom values and future explicit choices. Audio gain, decoding, RMS ballistics and FFT are unchanged. The Mac app is Developer ID signed and notarized; the standard first-open confirmation can still appear. Windows playback remains manually unverified and its executable is unsigned.

---

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
