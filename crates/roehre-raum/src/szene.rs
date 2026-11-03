//! Die Szene: ein Wohnzimmer der späten 80er bei Nacht. Ein Röhrenfernseher mit
//! Nussbaumgehäuse steht auf einem Eichentisch, daneben ein Videorekorder und Videokassetten,
//! Tasse und Fernbedienung; ein Gummibaum, eine Stehlampe, ein Bild an der Wand, ein Fenster
//! mit Vorhang und Heizkörper, ein Bücherregal. Alles selbst modelliert, ohne Marken.
//!
//! Maße in Metern. Der Fernseher steht auf dem Tisch nahe dem Ursprung, sein Bildschirm zeigt
//! nach −z (zur Kamera).

use crate::geometrie::{add, cross, mul, norm, sub, Netz, Trafo, V3};
use crate::texturen::Material::*;

pub const TISCH_HOEHE: f32 = 0.70;
/// Mitte des Bildschirms (Weltkoordinaten) und halbe Größe der Bildröhre.
pub const BILD_MITTE: V3 = [-0.06, TISCH_HOEHE + 0.035 + 0.245, -0.18];
pub const BILD_HALB: [f32; 2] = [0.20, 0.15];
/// Lage des Pflanztopfs: rechts neben dem Tisch, im Licht der Lampe.
pub const PFLANZE: V3 = [1.12, 0.0, 0.08];
/// Lage der Glühbirne der Stehlampe (Lichtquelle), in der hinteren rechten Zimmerecke.
pub const LAMPE: V3 = [1.95, 1.48, 0.70];

/// Baut die ganze Szene.
pub fn bauen() -> Netz {
    let mut n = Netz::default();
    zimmer(&mut n);
    tisch(&mut n);
    fernseher(&mut n);
    videorekorder(&mut n);
    kleinkram(&mut n);
    gummibaum(&mut n);
    kabel(&mut n);
    stehlampe(&mut n);
    regal(&mut n);
    n
}

fn zimmer(n: &mut Netz) {
    let (links, rechts, hinten, vorn, decke) = (-2.4, 2.4, 1.05, -4.6, 2.6);
    // Boden (Dielen), Teppich darauf
    let boden = Trafo::bei([0.0, 0.0, (hinten + vorn) / 2.0]).nach(&Trafo::dreh_x(std::f32::consts::FRAC_PI_2));
    n.flaeche(&boden, [(rechts - links) / 2.0, (hinten - vorn) / 2.0], Dielen, [0.0, 0.0, (rechts - links) / 1.6, (hinten - vorn) / 1.6]);
    let teppich = Trafo::bei([0.0, 0.004, -0.35]).nach(&Trafo::dreh_x(std::f32::consts::FRAC_PI_2));
    n.flaeche(&teppich, [1.3, 0.95], Teppich, [0.0, 0.0, 1.0, 1.0]);
    // Rückwand mit Fensteröffnung (aus vier Teilflächen um das Fenster)
    let (fx0, fx1, fy0, fy1) = (-1.85, -0.85, 0.95, 2.15);
    let wand = |n: &mut Netz, x0: f32, x1: f32, y0: f32, y1: f32| {
        let t = Trafo::bei([(x0 + x1) / 2.0, (y0 + y1) / 2.0, hinten]);
        n.flaeche(&t, [(x1 - x0) / 2.0, (y1 - y0) / 2.0], Tapete, [x0 / 1.2, -y1 / 1.2, (x1 - x0) / 1.2, (y1 - y0) / 1.2]);
    };
    wand(n, links, fx0, 0.0, decke);
    wand(n, fx1, rechts, 0.0, decke);
    wand(n, fx0, fx1, 0.0, fy0);
    wand(n, fx0, fx1, fy1, decke);
    // Seitenwände und Decke
    let seite_l = Trafo::bei([links, decke / 2.0, (hinten + vorn) / 2.0]).nach(&Trafo::dreh_y(-std::f32::consts::FRAC_PI_2));
    n.flaeche(&seite_l, [(hinten - vorn) / 2.0, decke / 2.0], Tapete, [0.0, 0.0, (hinten - vorn) / 1.2, decke / 1.2]);
    let seite_r = Trafo::bei([rechts, decke / 2.0, (hinten + vorn) / 2.0]).nach(&Trafo::dreh_y(std::f32::consts::FRAC_PI_2));
    n.flaeche(&seite_r, [(hinten - vorn) / 2.0, decke / 2.0], Tapete, [0.0, 0.0, (hinten - vorn) / 1.2, decke / 1.2]);
    let d = Trafo::bei([0.0, decke, (hinten + vorn) / 2.0]).nach(&Trafo::dreh_x(-std::f32::consts::FRAC_PI_2));
    n.flaeche(&d, [(rechts - links) / 2.0, (hinten - vorn) / 2.0], Decke, [0.0, 0.0, 3.0, 3.0]);
    // Fußleisten: schmal, dunkel gebeizt, oben abgerundet
    n.quader(&Trafo::bei([0.0, 0.035, hinten - 0.008]), [(rechts - links) / 2.0, 0.035, 0.008], 0.003, Furnier, 2.0);
    n.quader(&Trafo::bei([links + 0.008, 0.035, (hinten + vorn) / 2.0]), [0.008, 0.035, (hinten - vorn) / 2.0], 0.003, Furnier, 2.0);
    n.quader(&Trafo::bei([rechts - 0.008, 0.035, (hinten + vorn) / 2.0]), [0.008, 0.035, (hinten - vorn) / 2.0], 0.003, Furnier, 2.0);

    // Fenster: Nacht dahinter, Rahmen mit Sprossen, Fensterbank
    let nacht = Trafo::bei([(fx0 + fx1) / 2.0, (fy0 + fy1) / 2.0, hinten + 0.12]);
    n.flaeche(&nacht, [(fx1 - fx0) / 2.0 + 0.1, (fy1 - fy0) / 2.0 + 0.1], Nacht, [0.0, 0.0, 1.0, 1.0]);
    let (mx, my) = ((fx0 + fx1) / 2.0, (fy0 + fy1) / 2.0);
    for (x, y, hx, hy) in [
        (mx, fy0, 0.53, 0.03),
        (mx, fy1, 0.53, 0.03),
        (fx0, my, 0.03, 0.63),
        (fx1, my, 0.03, 0.63),
        (mx, my + 0.15, 0.5, 0.018),
        (mx, my, 0.018, 0.6),
    ] {
        n.quader(&Trafo::bei([x, y, hinten + 0.04]), [hx, hy, 0.035], 0.004, Lack, 3.0);
    }
    // Laibung (Wandstärke) und Fensterbank
    n.quader(&Trafo::bei([mx, fy0 - 0.02, hinten - 0.02]), [0.62, 0.02, 0.1], 0.006, Lack, 3.0);
    for x in [fx0 - 0.005, fx1 + 0.005] {
        n.quader(&Trafo::bei([x, my, hinten + 0.06]), [0.005, 0.6, 0.06], 0.0, Tapete, 1.0);
    }
    n.quader(&Trafo::bei([mx, fy1 + 0.005, hinten + 0.06]), [0.5, 0.005, 0.06], 0.0, Tapete, 1.0);
    // Vorhangstange und zwei Vorhänge in Falten (Windgewicht für sachte Bewegung)
    n.zylinder(&Trafo::bei([fx0 - 0.25, fy1 + 0.15, hinten - 0.1]).nach(&Trafo::dreh_z(-std::f32::consts::FRAC_PI_2)), 0.012, fx1 - fx0 + 0.5, 10, RahmenHolz);
    for (x0, breite) in [(fx0 - 0.25, 0.38), (fx1 - 0.08, 0.32)] {
        let hoehe = fy1 + 0.13 - 0.35;
        n.gitter(&Trafo::bei([x0, fy1 + 0.13, hinten - 0.1]), (24, 8), Vorhang, 1.0, |s, q| {
            let falte = (s * 24.0 * std::f32::consts::PI).sin();
            ([s * breite, -q * hoehe, 0.035 * falte], crate::geometrie::norm([-0.035 * 24.0 * std::f32::consts::PI / breite * (s * 24.0 * std::f32::consts::PI).cos(), 0.0, -1.0]), [s * 2.0, q * 3.0])
        });
    }
    // Heizkörper unter dem Fenster: Rippen, Rohre
    for i in 0..14 {
        let x = fx0 + 0.12 + i as f32 * 0.055;
        n.quader(&Trafo::bei([x, 0.5, hinten - 0.09]), [0.02, 0.28, 0.04], 0.012, Lack, 3.0);
    }
    n.rohr(&[[fx0 + 0.08, 0.25, hinten - 0.09], [fx1 - 0.15, 0.25, hinten - 0.09]], 0.012, 8, Lack);
    n.rohr(&[[fx0 + 0.08, 0.75, hinten - 0.09], [fx1 - 0.15, 0.75, hinten - 0.09]], 0.012, 8, Lack);
    n.rohr(&[[fx1 - 0.1, 0.55, hinten - 0.09], [fx1 - 0.1, 0.0, hinten - 0.09]], 0.01, 8, Chrom);

    // Bild an der Wand rechts vom Fernseher: alter Goldrahmen, Passepartout, Foto
    bilderrahmen(n, [0.55, 1.62, hinten], [0.27, 0.27], 0.065);

}

/// Goldrahmen mit geschwungenem Profil (Kehle, Wulst, Perlstab), auf Gehrung; innen ein
/// Passepartout und das Wandbild (zur Laufzeit austauschbar, `Raum::wandbild_setzen`).
fn bilderrahmen(n: &mut Netz, mitte: V3, aussen: [f32; 2], b: f32) {
    let pi = std::f32::consts::PI;
    // Profiltiefe über die Breite (0 außen an der Wand, 1 innen am Passepartout)
    let tiefe = move |q: f32| -> f32 {
        let wulst = (q * pi).sin().max(0.0).powf(0.6) * (1.0 - 0.4 * q);
        let kehle = -0.004 * ((q - 0.45) * 14.0).cos().max(0.0) * (1.0 - ((q - 0.45) * 4.0).abs()).max(0.0);
        let stufe = 0.003 * (1.0 - (q - 0.85).abs() * 12.0).max(0.0);
        0.006 * q + 0.032 * wulst + kehle + stufe
    };
    for k in 0..4 {
        let (entlang, innen, l0, e0) = match k {
            0 => ([1.0f32, 0.0], [0.0f32, -1.0], aussen[0], aussen[1]),
            1 => ([-1.0, 0.0], [0.0, 1.0], aussen[0], aussen[1]),
            2 => ([0.0, -1.0], [1.0, 0.0], aussen[1], aussen[0]),
            _ => ([0.0, 1.0], [-1.0, 0.0], aussen[1], aussen[0]),
        };
        let lage = move |s: f32, q: f32| -> V3 {
            let o = q * b;
            let l = (2.0 * s - 1.0) * (l0 - o);
            let e = e0 - o;
            [mitte[0] + entlang[0] * l - innen[0] * e, mitte[1] + entlang[1] * l - innen[1] * e, mitte[2] - tiefe(q)]
        };
        n.gitter(&Trafo::bei([0.0; 3]), (2, 24), Goldrahmen, 0.0, |s, q| {
            let eps = 1e-3;
            let du = sub(lage((s + eps).min(1.0), q), lage((s - eps).max(0.0), q));
            let dq = sub(lage(s, (q + eps).min(1.0)), lage(s, (q - eps).max(0.0)));
            let mut nn = norm(cross(du, dq));
            if nn[2] > 0.0 {
                nn = mul(nn, -1.0);
            }
            (lage(s, q), nn, [s * l0 * 6.0, q])
        });
    }
    let innen = [aussen[0] - b, aussen[1] - b];
    n.flaeche(&Trafo::bei(add(mitte, [0.0, 0.0, -0.006])), innen, Passepartout, [0.0, 0.0, 1.0, 1.0]);
    let foto = [innen[0] - 0.045, innen[1] - 0.045];
    n.flaeche(&Trafo::bei(add(mitte, [0.0, 0.0, -0.0075])), foto, Wandbild, [0.0, 0.0, 1.0, 1.0]);
}

/// Tisch aus massiver Eiche: Platte mit gerundeter Oberkante und abgesetzter Unterkante
/// (Stufenprofil), an den Enden Kopfleisten mit quer laufender Faser, Zarge, gedrechselte
/// Beine mit Wulst.
fn tisch(n: &mut Netz) {
    let (z0, uvm) = (-0.05f32, 1.1);
    let dicke = 0.016;
    // Mittelteil der Platte (Faser entlang der Tischlänge)
    n.quader(&Trafo::bei([0.0, TISCH_HOEHE - dicke, z0]), [0.735, dicke, 0.42], 0.011, Tischholz, uvm);
    // Kopfleisten an beiden Enden (Faser quer, sichtbare Fuge zur Platte)
    for sx in [-1.0f32, 1.0] {
        let t = Trafo::bei([sx * 0.772, TISCH_HOEHE - dicke, z0]).nach(&Trafo::dreh_y(std::f32::consts::FRAC_PI_2));
        n.quader(&t, [0.42, dicke, 0.036], 0.011, Tischholz, uvm);
    }
    // Abgesetzte Unterkante: etwas schmaler, bildet mit der Platte eine Stufe
    n.quader(&Trafo::bei([0.0, TISCH_HOEHE - 2.0 * dicke - 0.007, z0]), [0.79, 0.007, 0.405], 0.003, Tischholz, uvm);
    // Zarge
    n.quader(&Trafo::bei([0.0, TISCH_HOEHE - 0.085, z0]), [0.70, 0.04, 0.345], 0.004, Tischholz, uvm);
    // Gedrechselte Beine: Fuß, Wulst unter der Zarge, leicht verjüngt
    let h = TISCH_HOEHE - 0.04;
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let p = [0.70 * sx, 0.0, z0 + 0.335 * sz];
        n.drehkoerper(
            &Trafo::bei(p),
            &[(0.0, 0.0), (0.022, 0.0), (0.026, 0.012), (0.024, 0.03), (0.028, 0.32), (0.034, h - 0.16), (0.038, h - 0.15), (0.034, h - 0.14), (0.032, h), (0.0, h)],
            12,
            Tischholz,
            1.0,
        );
    }
}

/// Punkt auf dem Umfang eines Rechtecks mit gerundeten Ecken (halbe Größe `halb`, Eckradius
/// `r`), `s` ∈ [0, 1) einmal herum. Acht gleich lange Abschnitte des Parameters (vier Kanten,
/// vier Ecken), unabhängig vom Radius – so passen Umfänge mit verschiedenen Radien zueinander
/// und lassen sich zu Ringen verbinden (Radius 0 ergibt ein scharfkantiges Rechteck).
fn rund_umfang(halb: [f32; 2], r: f32, s: f32) -> [f32; 2] {
    let s = s.rem_euclid(1.0) * 8.0;
    let (k, t) = (s.floor() as i32, s.fract());
    let (ax, ay) = (halb[0] - r, halb[1] - r);
    let ecke = |cx: f32, cy: f32, w0: f32| {
        let w = (w0 - 90.0 * t).to_radians();
        [cx + r * w.cos(), cy + r * w.sin()]
    };
    match k {
        0 => [-ax + 2.0 * ax * t, halb[1]],
        1 => ecke(ax, ay, 90.0),
        2 => [halb[0], ay - 2.0 * ay * t],
        3 => ecke(ax, -ay, 0.0),
        4 => [ax - 2.0 * ax * t, -halb[1]],
        5 => ecke(-ax, -ay, -90.0),
        6 => [-halb[0], -ay + 2.0 * ay * t],
        _ => ecke(-ax, ay, 180.0),
    }
}

/// Ring zwischen zwei gerundeten Rechtecken um `mitte` (x, y): von (halb, Radius, z) außen
/// nach innen. Flach (gleiches z) ist es eine Blende mit runder Öffnung, mit verschiedenem z
/// ein schräger Tunnel. Die Normale zeigt zum Betrachter (−z).
fn rund_ring(n: &mut Netz, mitte: [f32; 2], aussen: ([f32; 2], f32, f32), innen: ([f32; 2], f32, f32), m: crate::texturen::Material, uvm: f32) {
    let lage = |s: f32, q: f32| -> V3 {
        let a = rund_umfang(aussen.0, aussen.1, s);
        let b = rund_umfang(innen.0, innen.1, s);
        [mitte[0] + a[0] + (b[0] - a[0]) * q, mitte[1] + a[1] + (b[1] - a[1]) * q, aussen.2 + (innen.2 - aussen.2) * q]
    };
    let umfang = 4.0 * (aussen.0[0] + aussen.0[1]);
    let breite = ((aussen.0[1] - innen.0[1]).powi(2) + (aussen.2 - innen.2).powi(2)).sqrt();
    n.gitter(&Trafo::bei([0.0; 3]), (8 * 12, 2), m, 0.0, |s, q| {
        let p = lage(s, q);
        let e = 1e-3;
        let ds = sub(lage(s + e, q), lage(s - e, q));
        let dq = sub(lage(s, (q + e).min(1.0)), lage(s, (q - e).max(0.0)));
        let mut nn = norm(cross(ds, dq));
        if nn[2] > 0.0 {
            nn = mul(nn, -1.0);
        }
        (p, nn, [s * umfang * uvm, q * breite * uvm])
    });
}

/// Röhrenfernseher der frühen 80er in grober Anlehnung an US-Tischgeräte jener Zeit:
/// Nussbaumgehäuse, eingesetzte Front aus dunkel eloxiertem Aluminium, tiefer Bildtunnel mit
/// gerundeten Ecken, rechts eine eigene Bediensäule (Netz/Lautstärke, Feinregler hinter
/// einer schwarzen Leiste, VHF- und UHF-Wähler, Lautsprecherlamellen), darunter ein
/// zurückgesetzter Sockel. Ohne Marke.
fn fernseher(n: &mut Netz) {
    let y0 = TISCH_HOEHE + 0.035;           // Unterkante Gehäuse (auf dem Sockel)
    let mitte = [0.0, y0 + 0.245, 0.0];
    let (hb, hh, ht) = (0.34, 0.245, 0.205);
    let w = 0.026;                                      // Stärke des Holzmantels
    n.quader(&Trafo::bei([0.0, mitte[1] + hh - w / 2.0, 0.0]), [hb, w / 2.0, ht], 0.006, Furnier, 2.5);
    n.quader(&Trafo::bei([0.0, mitte[1] - hh + w / 2.0, 0.0]), [hb, w / 2.0, ht], 0.006, Furnier, 2.5);
    n.quader(&Trafo::bei([-hb + w / 2.0, mitte[1], 0.0]), [w / 2.0, hh, ht], 0.006, Furnier, 2.5);
    n.quader(&Trafo::bei([hb - w / 2.0, mitte[1], 0.0]), [w / 2.0, hh, ht], 0.006, Furnier, 2.5);
    // Rückwand ragt in den Mantel hinein (ohne Spalt blitzt die Lampe nicht durch)
    let ein = w / 2.0;
    n.quader(&Trafo::bei([0.0, mitte[1], ht - 0.01]), [hb - w + ein, hh - w + ein, 0.01], 0.0, KunststoffSchwarz, 4.0);
    // Rückteil (Röhrenhals), verjüngt, mit Lüftungsschlitzen
    n.quader(&Trafo::bei([0.0, mitte[1] + 0.01, 0.28]), [0.24, 0.19, 0.09], 0.03, KunststoffSchwarz, 4.0);
    for i in 0..9 {
        n.quader(&Trafo::bei([-0.16 + i as f32 * 0.04, mitte[1] + 0.205, 0.26]), [0.012, 0.002, 0.05], 0.001, Gummi, 8.0);
    }
    // Sockel: schwarz, ringsum zurückgesetzt, mit vier Gummifüßen
    let sockel_h = (y0 - TISCH_HOEHE - 0.004) / 2.0;
    n.quader(&Trafo::bei([0.0, TISCH_HOEHE + 0.004 + sockel_h, 0.01]), [hb - 0.035, sockel_h, ht - 0.03], 0.003, KunststoffSchwarz, 4.0);
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        n.zylinder(&Trafo::bei([(hb - 0.06) * sx, TISCH_HOEHE, 0.14 * sz]), 0.012, 0.0045, 10, Gummi);
    }

    // Front: Innenmaß des Mantels, darin von links die Bildblende und rechts die Bediensäule
    let fz = -ht + 0.007;                               // Vorderfläche der Blende
    let (ix0, ix1) = (-hb + w, hb - w);
    let (iy0, iy1) = (mitte[1] - hh + w, mitte[1] + hh - w);
    let (bx, by) = (BILD_MITTE[0], BILD_MITTE[1]);
    let (hx, hy) = (BILD_HALB[0], BILD_HALB[1]);

    // Bildblende, mittig um das Bild: außen fast rechteckig, innen ein tiefer Tunnel mit weit
    // gerundeten Ecken; rechts daneben (nach einer Fuge) die Bediensäule
    let blende_halb = [bx - ix0 - 0.004, (iy1 - iy0) / 2.0 - 0.004];
    let bm = [bx, by];
    let saeule_x0 = bx + blende_halb[0] + 0.003;
    // Schattenfugen: dunkler Grund hinter den Fugen, ragt in den Mantel; um die Bildblende
    // als Ring (die Öffnung bleibt frei), hinter der Säule als Platte
    rund_ring(n, bm, ([blende_halb[0] + 0.008, blende_halb[1] + 0.008], 0.0, fz + 0.006), ([blende_halb[0] - 0.02, blende_halb[1] - 0.02], 0.0, fz + 0.006), Gummi, 4.0);
    n.quader(&Trafo::bei([(saeule_x0 - 0.006 + ix1 + ein) / 2.0, mitte[1], fz + 0.012]), [(ix1 + ein - saeule_x0 + 0.006) / 2.0, hh - w + ein, 0.004], 0.0, Gummi, 4.0);
    rund_ring(n, bm, (blende_halb, 0.008, fz), ([blende_halb[0] - 0.012, blende_halb[1] - 0.012], 0.03, fz), Blende, 6.0);
    // abgeschrägte Kante zwischen flacher Blende und Tunnel
    rund_ring(n, bm, ([blende_halb[0] - 0.012, blende_halb[1] - 0.012], 0.03, fz), ([blende_halb[0] - 0.016, blende_halb[1] - 0.016], 0.034, fz + 0.003), Chrom, 6.0);
    // Tunnel: von der Blende schräg in die Tiefe bis knapp vor das Glas
    rund_ring(n, [bx, by], ([blende_halb[0] - 0.016, blende_halb[1] - 0.016], 0.034, fz + 0.003), ([hx + 0.008, hy + 0.008], 0.028, fz + 0.034), Blende, 6.0);
    // Gummidichtung um das Glas
    rund_ring(n, [bx, by], ([hx + 0.008, hy + 0.008], 0.028, fz + 0.034), ([hx - 0.004, hy - 0.004], 0.02, fz + 0.035), Gummi, 8.0);
    // Glasmitte (Scheitel der Wölbung) liegt auf BILD_MITTE
    debug_assert!((BILD_MITTE[2] + 0.018 - (fz + 0.036)).abs() < 1e-4);
    n.bildroehre(&Trafo::bei([bx, by, BILD_MITTE[2] + 0.018]), [hx, hy], 0.018);

    bediensaeule(n, saeule_x0, ix1, iy0, iy1, fz);

    // Zimmerantenne auf dem Gerät: Fuß und zwei Teleskopstäbe mit Kugeln
    let fuss = [0.05, y0 + 0.49, 0.06];
    n.drehkoerper(&Trafo::bei(fuss), &[(0.0, 0.0), (0.06, 0.0), (0.055, 0.012), (0.03, 0.035), (0.012, 0.045), (0.0, 0.046)], 20, KunststoffSchwarz, 2.0);
    for (w, neigung) in [(0.35f32, 0.12f32), (-0.42, 0.18)] {
        let t = Trafo::bei([fuss[0], fuss[1] + 0.04, fuss[2]]).nach(&Trafo::dreh_z(w)).nach(&Trafo::dreh_x(neigung));
        let spitze = t.punkt([0.0, 0.48, 0.0]);
        n.rohr(&[t.punkt([0.0, 0.0, 0.0]), t.punkt([0.0, 0.2, 0.0])], 0.004, 8, Chrom);
        n.rohr(&[t.punkt([0.0, 0.2, 0.0]), spitze], 0.0028, 8, Chrom);
        n.drehkoerper(&Trafo::bei(spitze), &[(0.0, -0.006), (0.006, 0.0), (0.0, 0.006)], 10, Chrom, 1.0);
    }
}

/// Ein Aufdruck aus dem Atlas `Beschriftung` (Zeile 0–7), mittig bei (x, y), Breite `b`.
fn aufdruck(n: &mut Netz, x: f32, y: f32, z: f32, b: f32, zeile: u32) {
    n.flaeche(&Trafo::bei([x, y, z]), [b / 2.0, b / 16.0], Beschriftung, [0.0, zeile as f32 / 8.0, 1.0, 1.0 / 8.0]);
}

/// Senderwähler bei (x, y): Skalenring auf der Platte, runder Knopf mit Griffrillen und
/// quer darüber ein Knebel aus Metall, gedreht um `winkel` (Grad), mit hellem Zeigerende.
fn waehler(n: &mut Netz, x: f32, y: f32, z: f32, r: f32, skala: crate::texturen::Material, winkel: f32) {
    let achse = Trafo::bei([x, y, z - 0.0004]).nach(&Trafo::dreh_x(-std::f32::consts::FRAC_PI_2));
    n.flaeche(&Trafo::bei([x, y, z - 0.0005]), [r, r], skala, [0.0, 0.0, 1.0, 1.0]);
    let rk = r * 0.62;
    n.drehkoerper(&achse, &[(0.0, 0.0), (rk, 0.0), (rk, 0.0), (rk, 0.008), (rk, 0.008), (rk - 0.002, 0.010), (0.0, 0.010)], 36, Knopf, 1.0);
    let knebel = Trafo::bei([x, y, z - 0.014]).nach(&Trafo::dreh_z(winkel.to_radians()));
    n.quader(&knebel, [rk * 0.95, 0.0055, 0.006], 0.0025, Chrom, 8.0);
    n.quader(&knebel.nach(&Trafo::bei([0.0, 0.0, -0.0061])), [rk * 0.85, 0.0026, 0.0005], 0.0, Blende, 8.0);
    n.quader(&knebel.nach(&Trafo::bei([rk * 0.78, 0.0, -0.0063])), [0.0035, 0.0012, 0.0004], 0.0, Lack, 4.0);
}

/// Bediensäule rechts neben dem Bild (x0 … x1, y0 … y1 auf der Front `fz`), von oben:
/// Netz- und Lautstärketeil mit Drucktaste, Zugknopf und Kontrolllampe; schwarze Leiste mit
/// vier Feinreglern (Bildlage, Schärfe, Helligkeit, Farbton) und Farbknopf; daneben die
/// Wählerplatte mit VHF-Kanalring und UHF-Feinskala; unten Lautsprecherlamellen.
fn bediensaeule(n: &mut Netz, x0: f32, x1: f32, y0: f32, y1: f32, fz: f32) {
    let rx = Trafo::dreh_x(-std::f32::consts::FRAC_PI_2);
    let platte = |n: &mut Netz, a: f32, b: f32, c: f32, d: f32, z: f32, m| {
        n.quader(&Trafo::bei([(a + b) / 2.0, (c + d) / 2.0, z + 0.004]), [(b - a) / 2.0, (d - c) / 2.0, 0.004], 0.0015, m, 6.0);
    };
    let fuge = 0.003;
    // Teilung in der Höhe
    let oben = y1 - 0.075;
    let unten = y0 + 0.135;

    // 1) Kopfteil: Blende, schwarzes Schriftfeld, Drucktaste NETZ, Zugknopf LAUTSTÄRKE, Lampe
    platte(n, x0, x1, oben + fuge, y1, fz, Blende);
    platte(n, x0 + 0.008, x1 - 0.008, y1 - 0.022, y1 - 0.008, fz - 0.0005, KunststoffSchwarz);
    let (tx, kx) = (x0 + 0.032, x1 - 0.034);
    aufdruck(n, tx, y1 - 0.015, fz - 0.0006, 0.026, 0);
    aufdruck(n, kx, y1 - 0.015, fz - 0.0006, 0.05, 1);
    n.quader(&Trafo::bei([tx, oben + 0.030, fz - 0.003]), [0.011, 0.0065, 0.003], 0.0015, KunststoffSchwarz, 8.0);
    n.drehkoerper(&Trafo::bei([kx, oben + 0.030, fz]).nach(&rx), &[(0.0, 0.0), (0.0055, 0.0), (0.0055, 0.012), (0.0095, 0.013), (0.0098, 0.018), (0.0085, 0.020), (0.0, 0.0205)], 24, Chrom, 1.0);
    n.zylinder(&Trafo::bei([tx + 0.021, oben + 0.030, fz - 0.0003]).nach(&rx), 0.0018, 0.0012, 10, LedRot);

    // 2) Mittelteil: schwarze Leiste mit Feinreglern (links) und Wählerplatte (rechts)
    let leiste_x1 = x0 + 0.036;
    platte(n, x0, leiste_x1, unten + fuge, oben, fz + 0.004, KunststoffSchwarz);
    let regler = 4;
    for i in 0..regler {
        let y = oben - 0.032 - i as f32 * 0.036;
        aufdruck(n, (x0 + leiste_x1) / 2.0, y + 0.013, fz + 0.0035, 0.03, 2 + i as u32);
        // Rändelrad, halb in der Leiste versenkt
        n.drehkoerper(&Trafo::bei([(x0 + leiste_x1) / 2.0, y, fz + 0.004]).nach(&rx), &[(0.0, 0.0), (0.0055, 0.0), (0.0055, 0.0055), (0.0045, 0.0062), (0.0, 0.0064)], 24, Knopf, 1.0);
    }
    // Farbe: kleiner blauer Drucker unten in der Leiste
    aufdruck(n, (x0 + leiste_x1) / 2.0, unten + 0.03, fz + 0.0035, 0.022, 6);
    n.drehkoerper(&Trafo::bei([(x0 + leiste_x1) / 2.0, unten + 0.016, fz + 0.004]).nach(&rx), &[(0.0, 0.0), (0.006, 0.0), (0.006, 0.004), (0.004, 0.0052), (0.0, 0.0054)], 18, Fernbedienung, 1.0);
    // Wählerplatte
    let wx0 = leiste_x1 + fuge;
    platte(n, wx0, x1, unten + fuge, oben, fz, KunststoffSchwarz);
    let wm = (wx0 + x1) / 2.0;
    let r = ((x1 - wx0) / 2.0 - 0.006).min(0.04);
    let (vhf_y, uhf_y) = (oben - 0.012 - r, unten + 0.012 + r);
    waehler(n, wm, vhf_y, fz, r, Kanalskala, -30.0);
    waehler(n, wm, uhf_y, fz, r, Skala, 52.0);

    // 3) Lautsprecher: waagerechte Lamellen vor schwarzem Stoff, in einem Blendenrahmen
    platte(n, x0, x1, y0, unten, fz, Blende);
    let (lx0, lx1, ly0, ly1) = (x0 + 0.008, x1 - 0.008, y0 + 0.010, unten - 0.010);
    n.flaeche(&Trafo::bei([(lx0 + lx1) / 2.0, (ly0 + ly1) / 2.0, fz - 0.0004]), [(lx1 - lx0) / 2.0, (ly1 - ly0) / 2.0], Lautsprecherstoff, [0.0, 0.0, 4.0, 4.0]);
    let lamellen = 14;
    let schritt = (ly1 - ly0) / lamellen as f32;
    for i in 0..lamellen {
        let y = ly0 + (i as f32 + 0.5) * schritt;
        n.quader(&Trafo::bei([(lx0 + lx1) / 2.0, y, fz - 0.004]), [(lx1 - lx0) / 2.0, schritt * 0.28, 0.0035], 0.0012, Blende, 6.0);
    }
}

/// Videorekorder der 80er links neben dem Fernseher (ohne Marke): flaches schwarzes Gehäuse,
/// die Front mit Kassettenklappe, Leuchtanzeige („12:00“), Tasten und Netzschalter als Textur.
fn videorekorder(n: &mut Netz) {
    let (halb, fuss) = ([0.212f32, 0.044, 0.16], 0.006);
    let m = [-0.555, TISCH_HOEHE + fuss + halb[1], -0.06];
    n.quader(&Trafo::bei(m), halb, 0.004, KunststoffSchwarz, 6.0);
    // Front: Textur maßstäblich (256 Texel über die Breite, oberste 54 Zeilen)
    let front = Trafo::bei([m[0], m[1], m[2] - halb[2] - 0.0006]);
    n.flaeche(&front, [halb[0] - 0.002, halb[1] - 0.001], Videorekorder, [0.0, 0.0, 1.0, 54.0 / 256.0]);
    // Lüftungsschlitze oben hinten
    for i in 0..9 {
        n.quader(&Trafo::bei([m[0] - 0.12 + i as f32 * 0.03, m[1] + halb[1] + 0.0004, m[2] + 0.09]), [0.011, 0.0005, 0.04], 0.0, Gummi, 8.0);
    }
    for (sx, sz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        n.zylinder(&Trafo::bei([m[0] + sx * (halb[0] - 0.03), TISCH_HOEHE, m[2] + sz * (halb[2] - 0.03)]), 0.012, fuss, 10, Gummi);
    }
}

/// Eine Videohülle liegend: Rücken (Atlaszeile `zeile`) zur Kamera.
fn videohuelle(n: &mut Netz, t: &Trafo, zeile: u32) {
    let halb = [0.095f32, 0.0135, 0.0525];
    n.quader(t, halb, 0.002, KunststoffSchwarz, 8.0);
    n.flaeche(&t.nach(&Trafo::bei([0.0, 0.0, -halb[2] - 0.0004])), [halb[0] - 0.001, halb[1] - 0.0008], Kassette, [0.0, zeile as f32 / 8.0, 1.0, 1.0 / 8.0]);
}

fn kleinkram(n: &mut Netz) {
    // Videokassetten in Hüllen, gestapelt, leicht verdreht; die Rücken zeigen zur Kamera.
    // Rechts hinten drei, links vorn in der Tischecke vier (oben die selbst bespielte); die
    // Anzeige des Videorekorders bleibt frei.
    for (i, zeile) in [4u32, 5, 6].into_iter().enumerate() {
        let t = Trafo::bei([0.52, TISCH_HOEHE + 0.0135 + i as f32 * 0.027, -0.08]).nach(&Trafo::dreh_y(0.08 * (i as f32 - 1.0) + 0.1));
        videohuelle(n, &t, zeile);
    }
    for (i, (zeile, dreh, dx)) in [(0u32, -0.04f32, 0.0f32), (1, 0.05, 0.006), (2, -0.02, -0.004), (7, 0.16, 0.01)].into_iter().enumerate() {
        let t = Trafo::bei([-0.685 + dx, TISCH_HOEHE + 0.0135 + i as f32 * 0.027, -0.345]).nach(&Trafo::dreh_y(dreh));
        videohuelle(n, &t, zeile);
    }
    // Tasse mit Henkel
    let tasse = [0.40, TISCH_HOEHE, -0.30];
    n.drehkoerper(&Trafo::bei(tasse), &[(0.0, 0.002), (0.034, 0.0), (0.038, 0.01), (0.04, 0.09), (0.037, 0.092), (0.034, 0.085), (0.032, 0.015), (0.0, 0.016)], 24, Keramik, 3.0);
    let henkel: Vec<V3> = (0..=10).map(|i| {
        let w = -std::f32::consts::FRAC_PI_2 + i as f32 / 10.0 * std::f32::consts::PI;
        [tasse[0] + 0.04 + 0.025 * w.cos(), tasse[1] + 0.05 + 0.028 * w.sin(), tasse[2]]
    }).collect();
    n.rohr(&henkel, 0.006, 8, Keramik);
    // Fernbedienung: flaches, schlankes Gehäuse, Tasten als Textur auf der Oberseite
    let fb = Trafo::bei([0.22, TISCH_HOEHE + 0.007, -0.34]).nach(&Trafo::dreh_y(-0.45));
    n.quader(&fb, [0.022, 0.007, 0.092], 0.005, KunststoffSchwarz, 8.0);
    let oben = fb.nach(&Trafo::bei([0.0, 0.0072, 0.0])).nach(&Trafo::dreh_x(std::f32::consts::FRAC_PI_2));
    n.flaeche(&oben, [0.019, 0.088], Fernbedienung, [0.0, 0.0, 1.0, 1.0]);
}

/// Gummibaum (Ficus elastica): drei Triebe verschiedener Höhe aus einem Topf, große ovale
/// Blätter an kurzen Stielen, längs der Mittelrippe gefaltet und nach außen überhängend; an
/// jeder Triebspitze die rote Hülle des nächsten Blatts.
fn gummibaum(n: &mut Netz) {
    let topf = PFLANZE;
    n.drehkoerper(&Trafo::bei(topf), &[(0.0, 0.0), (0.12, 0.0), (0.15, 0.02), (0.18, 0.30), (0.195, 0.31), (0.195, 0.34), (0.17, 0.34), (0.16, 0.31)], 28, Terrakotta, 3.0);
    n.drehkoerper(&Trafo::bei([topf[0], 0.30, topf[2]]), &[(0.16, 0.0), (0.0, 0.012)], 20, Erde, 2.0);
    // Untersetzer
    n.drehkoerper(&Trafo::bei(topf), &[(0.0, 0.0), (0.2, 0.0), (0.21, 0.018), (0.19, 0.018), (0.0, 0.005)], 28, Terrakotta, 3.0);
    // (Fuß, Höhe, Neigung, Zahl der Blätter, Drehung)
    #[allow(clippy::type_complexity)]
    let triebe: [([f32; 2], f32, [f32; 2], usize, f32); 3] = [
        ([0.0, 0.0], 1.15, [-0.06, -0.05], 15, 0.0),
        ([0.05, 0.04], 0.85, [0.18, 0.07], 12, 1.3),
        ([-0.05, 0.03], 0.62, [-0.14, 0.12], 9, 2.6),
    ];
    for (fuss, hoehe, neigung, blaetter, dreh) in triebe {
        let stamm: Vec<V3> = (0..=12)
            .map(|i| {
                let s = i as f32 / 12.0;
                let w = s.powf(1.5);
                [topf[0] + fuss[0] + neigung[0] * w, 0.30 + s * hoehe, topf[2] + fuss[1] + neigung[1] * w]
            })
            .collect();
        // nach oben dünner werdend: drei Abschnitte
        n.rohr(&stamm[0..5], 0.0085, 8, Rinde);
        n.rohr(&stamm[4..9], 0.0065, 8, Rinde);
        n.rohr(&stamm[8..13], 0.0048, 8, Rinde);
        for i in 0..blaetter {
            let t = 0.18 + 0.8 * i as f32 / blaetter as f32;
            let k = t * 12.0;
            let (a, b) = (stamm[k.floor() as usize], stamm[(k.floor() as usize + 1).min(12)]);
            let ansatz = add(a, mul(sub(b, a), k.fract()));
            let alter = 1.0 - t; // unten alt, groß und hängend; oben jung, kleiner und aufrecht
            let winkel = dreh + i as f32 * 2.4;
            let laenge = 0.24 + 0.16 * alter;
            let rollen = ((i as f32 * 12.9898 + dreh * 78.233).sin() * 43758.545).fract() - 0.5;
            ficusblatt(n, ansatz, winkel, laenge, laenge * 0.5, 0.9 - 0.8 * alter, -0.1 - 0.8 * alter, rollen * 1.2);
        }
        // Rote Hülle an der Spitze
        let spitze = stamm[12];
        n.drehkoerper(&Trafo::bei(spitze), &[(0.0, 0.0), (0.006, 0.0), (0.006, 0.03), (0.003, 0.06), (0.0, 0.07)], 8, Triebhuelle, 1.0);
    }
}

/// Ein Gummibaumblatt: Stiel vom Ansatz schräg nach oben, dann die Blattspreite, deren
/// Neigung von `neig_anfang` zur Spitze hin auf `neig_ende` (Bogenmaß) abfällt; gefaltet
/// längs der Mittelrippe und um sie um `rollen` gedreht (so zeigen nicht alle Blätter die
/// Kante zum Betrachter).
#[allow(clippy::too_many_arguments)]
fn ficusblatt(n: &mut Netz, ansatz: V3, winkel: f32, laenge: f32, breite: f32, neig_anfang: f32, neig_ende: f32, rollen: f32) {
    let aus = [winkel.cos(), 0.0, winkel.sin()];
    let seite = [-winkel.sin(), 0.0, winkel.cos()];
    let oben = [0.0, 1.0, 0.0];
    let stiel_neig = 0.9f32;
    let stiel_ende = add(ansatz, add(mul(aus, 0.05 * stiel_neig.cos()), mul(oben, 0.05 * stiel_neig.sin())));
    n.rohr(&[ansatz, add(ansatz, mul(add(aus, oben), 0.012)), stiel_ende], 0.0028, 5, Rinde);
    // Mittelrippe als Polylinie
    const SCHRITTE: usize = 16;
    let mut mitte = vec![stiel_ende];
    let mut lot = Vec::new();
    for j in 0..=SCHRITTE {
        let t = j as f32 / SCHRITTE as f32;
        let e = neig_anfang + (neig_ende - neig_anfang) * t * t;
        let richtung = add(mul(aus, e.cos()), mul(oben, e.sin()));
        lot.push(add(mul(aus, -e.sin()), mul(oben, e.cos())));
        if j < SCHRITTE {
            let p = add(mitte[j], mul(richtung, laenge / SCHRITTE as f32));
            mitte.push(p);
        }
    }
    let falte = 0.32f32;
    let lage = |u: f32, t: f32| -> V3 {
        let k = (t * SCHRITTE as f32).min(SCHRITTE as f32 - 1e-3);
        let (i, f) = (k.floor() as usize, k.fract());
        let m = add(mitte[i], mul(sub(mitte[i + 1], mitte[i]), f));
        let l0 = norm(add(lot[i], mul(sub(lot[i + 1], lot[i]), f)));
        let r = rollen * (0.4 + 0.6 * t);
        let quer = add(mul(seite, r.cos()), mul(l0, r.sin()));
        let l = sub(mul(l0, r.cos()), mul(seite, r.sin()));
        let x = (u - 0.5) * breite;
        let welle = 0.004 * (t * 9.0 + u * 3.0).sin() * (x.abs() / breite) * 2.0;
        add(m, add(mul(quer, x * falte.cos()), mul(l, x.abs() * falte.sin() + welle)))
    };
    n.gitter(&Trafo::bei([0.0; 3]), (8, SCHRITTE as u32), Blatt, 0.0, |u, t| {
        let eps = 1e-3;
        let du = sub(lage((u + eps).min(1.0), t), lage((u - eps).max(0.0), t));
        let dt = sub(lage(u, (t + eps).min(1.0)), lage(u, (t - eps).max(0.0)));
        (lage(u, t), norm(cross(du, dt)), [u, t])
    });
}

/// Steckdose an der Rückwand, Strom- und Antennenkabel vom Fernseher dorthin.
fn kabel(n: &mut Netz) {
    let dose = [0.35, 0.32, 1.035];
    n.quader(&Trafo::bei(dose), [0.04, 0.04, 0.012], 0.01, Lack, 6.0);
    for dx in [-0.012f32, 0.012] {
        n.zylinder(&Trafo::bei([dose[0] + dx, dose[1], dose[2] - 0.012]).nach(&Trafo::dreh_x(-std::f32::consts::FRAC_PI_2)), 0.004, 0.002, 8, Gummi);
    }
    let stecker = [dose[0], dose[1], dose[2] - 0.03];
    n.quader(&Trafo::bei(stecker), [0.016, 0.022, 0.02], 0.006, Gummi, 6.0);
    // Stromkabel: aus dem Rückteil des Fernsehers, über die Tischkante, durchhängend zur Dose
    let start = [0.1, TISCH_HOEHE + 0.12, 0.37];
    let punkte: Vec<V3> = (0..=24)
        .map(|i| {
            let s = i as f32 / 24.0;
            let x = start[0] + (stecker[0] - start[0]) * s;
            let z = start[2] + (stecker[2] - 0.02 - start[2]) * s;
            let y = start[1] + (stecker[1] - start[1]) * s - 0.55 * (s * std::f32::consts::PI).sin() * (1.0 - 0.3 * s);
            [x, y.max(0.012), z]
        })
        .collect();
    n.rohr(&punkte, 0.0035, 6, Gummi);
    // Antennenkabel (dicker, grau) zur Antennendose daneben
    let adose = [0.55, 0.32, 1.035];
    n.quader(&Trafo::bei(adose), [0.04, 0.04, 0.012], 0.01, Lack, 6.0);
    let a0 = [0.18, TISCH_HOEHE + 0.2, 0.37];
    let punkte: Vec<V3> = (0..=24)
        .map(|i| {
            let s = i as f32 / 24.0;
            let x = a0[0] + (adose[0] - a0[0]) * s;
            let z = a0[2] + (adose[2] - 0.03 - a0[2]) * s;
            let y = a0[1] + (adose[1] - a0[1]) * s - 0.5 * (s * std::f32::consts::PI).sin();
            [x, y.max(0.014), z]
        })
        .collect();
    n.rohr(&punkte, 0.0045, 6, KunststoffGrau);
}

fn stehlampe(n: &mut Netz) {
    let fuss = [LAMPE[0], 0.0, LAMPE[2]];
    n.drehkoerper(&Trafo::bei(fuss), &[(0.0, 0.0), (0.16, 0.0), (0.15, 0.015), (0.03, 0.03), (0.0, 0.032)], 24, Chrom, 2.0);
    n.zylinder(&Trafo::bei(fuss), 0.012, LAMPE[1] - 0.05, 10, Chrom);
    // Schirm (Kegelstumpf, leuchtet von innen) und Birne
    n.drehkoerper(&Trafo::bei([LAMPE[0], LAMPE[1] - 0.14, LAMPE[2]]), &[(0.24, 0.0), (0.235, 0.004), (0.13, 0.30), (0.125, 0.304)], 32, Lampenschirm, 4.0);
    n.drehkoerper(&Trafo::bei([LAMPE[0], LAMPE[1] - 0.05, LAMPE[2]]), &[(0.0, 0.0), (0.03, 0.03), (0.035, 0.06), (0.02, 0.09), (0.0, 0.095)], 16, Gluehbirne, 1.0);
}

/// Bücherregal an der linken Wand: Seiten, Rückwand, fünf Böden, Bücher als Textur auf
/// schmalen Blöcken (Buchrücken zeigen in den Raum), dazu ein paar liegende Stapel.
fn regal(n: &mut Netz) {
    let (x, z0, z1, hoehe, tiefe) = (-2.4, -1.35, -0.35, 1.9, 0.3);
    let zm = (z0 + z1) / 2.0;
    let xm = x + tiefe / 2.0;
    let halb_z = (z1 - z0) / 2.0;
    let brett = 0.012;
    for z in [z0, z1] {
        n.quader(&Trafo::bei([xm, hoehe / 2.0, z]), [tiefe / 2.0, hoehe / 2.0, brett], 0.002, Furnier, 2.0);
    }
    n.quader(&Trafo::bei([x + 0.006, hoehe / 2.0, zm]), [0.004, hoehe / 2.0, halb_z], 0.0, Furnier, 2.0);
    let boeden = [0.05, 0.42, 0.79, 1.16, 1.53, hoehe - brett];
    for y in boeden {
        n.quader(&Trafo::bei([xm, y, zm]), [tiefe / 2.0, brett, halb_z - brett], 0.002, Furnier, 2.0);
    }
    // Bücher: je Fach eine Reihe; die Fläche liegt in der Ebene der Buchrücken, die Textur
    // schneidet die oberen Kanten unterschiedlich hoch ab
    for (i, y) in boeden.iter().take(5).enumerate() {
        let fach = boeden[i + 1] - y - 2.0 * brett;
        let h = fach * 0.92;
        let reihe = Trafo::bei([x + tiefe - 0.03, y + brett + h / 2.0, zm]).nach(&Trafo::dreh_y(-std::f32::consts::FRAC_PI_2));
        let breite = halb_z - brett - 0.03 * (i % 2) as f32;
        n.flaeche(&reihe, [breite, h / 2.0], Buecher, [i as f32 * 0.37, 0.0, breite * 2.0 / 0.6, 1.0]);
        // Dunkler Block dahinter, damit über kurzen Büchern nicht die Wand durchscheint
        n.quader(&Trafo::bei([x + tiefe / 2.0 - 0.01, y + brett + h * 0.3, zm]), [tiefe / 2.0 - 0.03, h * 0.3, breite], 0.0, KunststoffSchwarz, 1.0);
    }
    // liegender Stapel und eine Kassette obenauf
    for k in 0..3 {
        let t = Trafo::bei([xm + 0.02, hoehe + 0.012 + k as f32 * 0.024, zm + 0.2]).nach(&Trafo::dreh_y(0.1 * k as f32));
        n.quader(&t, [0.11, 0.011, 0.08], 0.003, Furnier, 1.0);
    }
}
