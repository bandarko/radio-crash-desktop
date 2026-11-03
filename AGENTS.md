# Owner review before publication

For every new app release, first prepare a local build for the owner to try.
Wait for the owner's explicit approval of that tested build before putting the
new release on GitHub. This includes prerelease/test release uploads. Do not
replace existing public release assets or publish a tag as a substitute for
local owner review. During local review, keep implementation commits local.

Standing owner instruction, 2026-11-02:
"I kad radiš novi release prije nego ga staviš na GitHub ja ću svaki puta probati pa ti onda odobriti da ide ako je sve u redu."

VFD Digital - Black Aluminium remains the default skin. Analog VU - Silver
Aluminium is optional and saves its selection. Its detector is fixed classical
full-wave average VU with 300 ms movement and a scale ending at +3 VU. Only
analog display level is adjustable; preserve the separate digital RMS/Peak
preferences. All instruments must use actual consumed playback PCM.

Analog VU - Ivory Glass is a third optional skin, sharing exactly the same fixed
analog detector and sensitivity. Its continuous ivory centre has no dark display
rectangle. Any future material/layout change must keep native control geometry
aligned with the photographed controls at compact and large window sizes.

Owner-approved geometry direction for Ivory Glass: straight horizontal scale
and label rows with fanned divisions, not an oval/arc. Needle travel stays
circular and divisions must intersect its actual measurement ray. Keep Silver
Aluminium's existing curved card and low centre window unchanged.

Both silver wordmarks must use the exact supplied station bitmap, like VFD,
with transparent coverage and dark ink on aluminium. Do not generate/retype them.
Windows chassis drags must not enter native SC_MOVE; keep frame updates active
and use one consistent physical screen coordinate system to avoid DPI offsets.
