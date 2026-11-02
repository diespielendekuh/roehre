//! Die Szene: ein Wohnzimmer der späten 80er bei Nacht. Ein Röhrenfernseher mit
//! Holzfurnier steht auf einem Holztisch, daneben eine Spielkonsole im Stil des Super Famicom
//! (ohne Logo) mit Controller, Videokassetten,
//! Tasse, Fernbedienung und Zeitschrift; ein Gummibaum, eine Stehlampe, ein Bild an der Wand,
//! ein Fenster mit Vorhang und Heizkörper. Alles selbst modelliert; die Konsole ist in der Form
//! an ein reales Gerät angelehnt, trägt aber kein Logo und keine Schrift.
//!
//! Maße in Metern. Der Fernseher steht auf dem Tisch nahe dem Ursprung, sein Bildschirm zeigt
//! nach −z (zur Kamera).

use crate::geometrie::{add, cross, mul, norm, sub, Netz, Trafo, V3};
use crate::texturen::Material::*;

pub const TISCH_HOEHE: f32 = 0.70;
/// Mitte des Bildschirms (Weltkoordinaten) und halbe Größe der Bildröhre.
pub const BILD_MITTE: V3 = [-0.06, TISCH_HOEHE + 0.035 + 0.245, -0.215];
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
    konsole(&mut n);
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

fn tisch(n: &mut Netz) {
    let mitte = [0.0, TISCH_HOEHE - 0.022, -0.05];
    n.quader(&Trafo::bei(mitte), [0.78, 0.022, 0.40], 0.008, Tischholz, 1.4);
    // Zarge (Brett unter der Platte)
    n.quader(&Trafo::bei([0.0, TISCH_HOEHE - 0.075, -0.05]), [0.70, 0.032, 0.34], 0.004, Tischholz, 1.4);
    // Beine, leicht verjüngt
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let p = [0.70 * sx, 0.0, -0.05 + 0.33 * sz];
        n.drehkoerper(&Trafo::bei(p), &[(0.0, 0.0), (0.02, 0.0), (0.024, 0.04), (0.03, TISCH_HOEHE - 0.06), (0.0, TISCH_HOEHE - 0.06)], 10, Tischholz, 1.0);
    }
}

fn fernseher(n: &mut Netz) {
    let y0 = TISCH_HOEHE + 0.035;           // Unterkante Gehäuse (auf Füßen)
    let mitte = [0.0, y0 + 0.245, 0.0];
    // Gehäuse: Mantel aus Holzfurnier (Deckel, Boden, Seiten), vorn eine eingesetzte schwarze
    // Front mit Bildöffnung und Bedienfeld. Hohl gebaut, damit die Bildröhre sichtbar bleibt.
    let (hb, hh, ht) = (0.33, 0.245, 0.205);
    let w = 0.022;                                      // Stärke des Holzmantels
    n.quader(&Trafo::bei([0.0, mitte[1] + hh - w / 2.0, 0.0]), [hb, w / 2.0, ht], 0.008, Furnier, 2.5);
    n.quader(&Trafo::bei([0.0, mitte[1] - hh + w / 2.0, 0.0]), [hb, w / 2.0, ht], 0.008, Furnier, 2.5);
    n.quader(&Trafo::bei([-hb + w / 2.0, mitte[1], 0.0]), [w / 2.0, hh, ht], 0.008, Furnier, 2.5);
    n.quader(&Trafo::bei([hb - w / 2.0, mitte[1], 0.0]), [w / 2.0, hh, ht], 0.008, Furnier, 2.5);
    // Rückwand und Innenraum (dunkel)
    n.quader(&Trafo::bei([0.0, mitte[1], ht - 0.01]), [hb - w, hh - w, 0.01], 0.0, KunststoffSchwarz, 4.0);
    n.quader(&Trafo::bei([BILD_MITTE[0], BILD_MITTE[1], -0.12]), [BILD_HALB[0] + 0.03, BILD_HALB[1] + 0.03, 0.01], 0.0, Gummi, 4.0);
    let front_z = -ht + 0.006;
    // Schwarze Front: Rahmen um die Bildöffnung und Block für das Bedienfeld
    let (ix0, ix1, iy0, iy1) = (-hb + w, hb - w, mitte[1] - hh + w, mitte[1] + hh - w);
    let (ox0, ox1) = (BILD_MITTE[0] - BILD_HALB[0] - 0.012, BILD_MITTE[0] + BILD_HALB[0] + 0.012);
    let (oy0, oy1) = (BILD_MITTE[1] - BILD_HALB[1] - 0.012, BILD_MITTE[1] + BILD_HALB[1] + 0.012);
    let tafel = |n: &mut Netz, x0: f32, x1: f32, y0: f32, y1: f32| {
        n.quader(&Trafo::bei([(x0 + x1) / 2.0, (y0 + y1) / 2.0, front_z + 0.02]), [(x1 - x0) / 2.0, (y1 - y0) / 2.0, 0.02], 0.002, KunststoffSchwarz, 4.0);
    };
    tafel(n, ix0, ox0, iy0, iy1);                      // links der Öffnung
    tafel(n, ox0, ox1, oy1, iy1);                      // darüber
    tafel(n, ox0, ox1, iy0, oy0);                      // darunter
    tafel(n, ox1, ix1, iy0, iy1);                      // rechts: Bedienfeld
    // Rückteil (Röhrenhals), verjüngt, mit Lüftungsschlitzen
    n.quader(&Trafo::bei([0.0, mitte[1] + 0.01, 0.28]), [0.24, 0.19, 0.09], 0.03, KunststoffSchwarz, 4.0);
    for i in 0..9 {
        n.quader(&Trafo::bei([-0.16 + i as f32 * 0.04, mitte[1] + 0.205, 0.26]), [0.012, 0.002, 0.05], 0.001, Gummi, 8.0);
    }
    // Füße
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        n.zylinder(&Trafo::bei([0.27 * sx, TISCH_HOEHE, 0.15 * sz]), 0.022, 0.036, 12, Gummi);
    }

    // Bildöffnung: Chromring, schräge Fase nach innen, darin die gewölbte Bildröhre
    let (bx, by) = (BILD_MITTE[0], BILD_MITTE[1]);
    let (hx, hy) = (BILD_HALB[0], BILD_HALB[1]);
    let z_front = front_z - 0.013;
    for (x, y, ax, ay) in [
        (bx, by + hy + 0.022, hx + 0.03, 0.007),
        (bx, by - hy - 0.022, hx + 0.03, 0.007),
        (bx - hx - 0.022, by, 0.007, hy + 0.03),
        (bx + hx + 0.022, by, 0.007, hy + 0.03),
    ] {
        n.quader(&Trafo::bei([x, y, z_front]), [ax, ay, 0.004], 0.003, Chrom, 6.0);
    }
    // Fase: vier schräge Streifen von der Front zur Röhre
    let tiefe = 0.03;
    for (seite, drehung) in [(0, 0.0f32), (1, std::f32::consts::FRAC_PI_2), (2, std::f32::consts::PI), (3, -std::f32::consts::FRAC_PI_2)] {
        let laenge = if seite % 2 == 0 { hx + 0.02 } else { hy + 0.02 };
        let abstand = if seite % 2 == 0 { hy + 0.012 } else { hx + 0.012 };
        let t = Trafo::bei([bx, by, z_front + tiefe * 0.5])
            .nach(&Trafo::dreh_z(drehung))
            .nach(&Trafo::bei([0.0, abstand, 0.0]))
            .nach(&Trafo::dreh_x(-1.0));
        n.flaeche(&t, [laenge, tiefe * 0.62], KunststoffSchwarz, [0.0, 0.0, 2.0, 0.2]);
    }
    n.bildroehre(&Trafo::bei([bx, by, z_front + tiefe - 0.004]), [hx, hy], 0.018);

    // Bedienfeld rechts: zwei Drehknöpfe, Programmtasten, Netzschalter, Kontrolllampe,
    // Lautsprechergitter, Plakette (ohne Schrift)
    let px = 0.245;
    for (i, y) in [mitte[1] + 0.17, mitte[1] + 0.09].iter().enumerate() {
        let t = Trafo::bei([px, *y, front_z - 0.012]).nach(&Trafo::dreh_x(-std::f32::consts::FRAC_PI_2));
        n.drehkoerper(&t, &[(0.0, 0.0), (0.026, 0.0), (0.026, 0.004), (0.024, 0.022), (0.02, 0.026), (0.0, 0.027)], 20, Gummi, 6.0);
        n.drehkoerper(&Trafo::bei([px, *y, front_z - 0.039]).nach(&Trafo::dreh_x(-std::f32::consts::FRAC_PI_2)), &[(0.0, 0.0), (0.012, 0.0), (0.0, 0.001)], 16, Chrom, 1.0);
        // Markierungsstrich
        n.quader(&Trafo::bei([px + 0.008 * (i as f32 * 2.0 - 1.0), y + 0.012, front_z - 0.04]), [0.0015, 0.008, 0.001], 0.0, Lack, 1.0);
    }
    for i in 0..8 {
        let (sx, sy) = (i % 2, i / 2);
        n.quader(&Trafo::bei([px - 0.016 + sx as f32 * 0.032, mitte[1] + 0.02 - sy as f32 * 0.024, front_z - 0.014]), [0.012, 0.008, 0.006], 0.002, Chrom, 8.0);
    }
    n.quader(&Trafo::bei([px, mitte[1] - 0.09, front_z - 0.014]), [0.03, 0.01, 0.006], 0.003, KunststoffGrau, 8.0);
    n.zylinder(&Trafo::bei([px + 0.045, mitte[1] - 0.09, front_z - 0.012]).nach(&Trafo::dreh_x(-std::f32::consts::FRAC_PI_2)), 0.004, 0.004, 10, LedRot);
    let gitter = Trafo::bei([px, mitte[1] - 0.165, front_z - 0.0125]);
    n.flaeche(&gitter, [0.055, 0.055], Lautsprecherstoff, [0.0, 0.0, 1.0, 1.0]);
    for (x, y, ax, ay) in [(px, mitte[1] - 0.105, 0.058, 0.003), (px, mitte[1] - 0.225, 0.058, 0.003), (px - 0.058, mitte[1] - 0.165, 0.003, 0.061), (px + 0.058, mitte[1] - 0.165, 0.003, 0.061)] {
        n.quader(&Trafo::bei([x, y, front_z - 0.013]), [ax, ay, 0.003], 0.001, Chrom, 6.0);
    }
    n.quader(&Trafo::bei([bx, by - hy - 0.05, front_z - 0.013]), [0.05, 0.009, 0.002], 0.002, Chrom, 4.0);

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

/// Spielkonsole im Stil des Super Famicom (ohne Logo und Schriftzug): hellgraues, flaches
/// Gehäuse, erhöhter Modulschacht in dunklerem Grau, lila Schieber für Netz und Reset, lila
/// Auswurftaste, zwei Controllerbuchsen vorn, rote Leuchte; davor der passende Controller in
/// Hundeknochenform. Feinheiten (Tasten, Steuerkreuz) als Textur.
fn konsole(n: &mut Netz) {
    let m = [-0.50, TISCH_HOEHE, -0.04];
    let dreh = Trafo::bei(m).nach(&Trafo::dreh_y(0.12));
    // Grundkörper: vorn etwas niedriger als hinten (zwei Stufen)
    n.quader(&dreh.nach(&Trafo::bei([0.0, 0.021, 0.0])), [0.10, 0.021, 0.12], 0.008, KonsoleHell, 4.0);
    n.quader(&dreh.nach(&Trafo::bei([0.0, 0.046, 0.035])), [0.096, 0.006, 0.08], 0.006, KonsoleHell, 4.0);
    // Modulschacht: dunkleres Grau, Öffnung mit Klappe
    n.quader(&dreh.nach(&Trafo::bei([0.0, 0.055, 0.045])), [0.07, 0.006, 0.05], 0.004, KunststoffGrau, 6.0);
    n.quader(&dreh.nach(&Trafo::bei([0.0, 0.0615, 0.045])), [0.055, 0.0012, 0.006], 0.0005, Gummi, 8.0);
    // Schieber (Netz links, Reset rechts) und Auswurftaste davor
    for (x, z) in [(-0.075f32, -0.015f32), (0.075, -0.015)] {
        n.quader(&dreh.nach(&Trafo::bei([x, 0.054, z])), [0.012, 0.004, 0.011], 0.003, Lila, 8.0);
        n.quader(&dreh.nach(&Trafo::bei([x, 0.0525, z - 0.008])), [0.015, 0.0012, 0.022], 0.0005, KunststoffGrau, 8.0);
    }
    n.quader(&dreh.nach(&Trafo::bei([0.0, 0.0535, -0.018])), [0.022, 0.004, 0.008], 0.003, Lila, 8.0);
    // vorn: zwei Buchsen und die Leuchte
    for x in [-0.03f32, 0.03] {
        n.quader(&dreh.nach(&Trafo::bei([x, 0.02, -0.1205])), [0.012, 0.0065, 0.0015], 0.0006, Gummi, 8.0);
    }
    n.zylinder(&dreh.nach(&Trafo::bei([-0.075, 0.03, -0.1205])).nach(&Trafo::dreh_x(-std::f32::consts::FRAC_PI_2)), 0.0018, 0.001, 8, LedRot);

    // Controller: zwei runde Enden und der Steg, oben die Textur mit allen Tasten
    let c = Trafo::bei([-0.30, TISCH_HOEHE, -0.32]).nach(&Trafo::dreh_y(0.28));
    let dicke = 0.016;
    for x in [-0.042f32, 0.042] {
        n.zylinder(&c.nach(&Trafo::bei([x, 0.0, 0.0])), 0.0312, dicke, 24, KonsoleHell);
    }
    n.quader(&c.nach(&Trafo::bei([0.0, dicke / 2.0, 0.0])), [0.044, dicke / 2.0, 0.0248], 0.002, KonsoleHell, 8.0);
    // Schultertasten hinten
    for x in [-0.046f32, 0.046] {
        n.quader(&c.nach(&Trafo::bei([x, dicke * 0.72, 0.021])), [0.014, dicke * 0.26, 0.006], 0.003, KunststoffGrau, 8.0);
    }
    let oben = c.nach(&Trafo::bei([0.0, dicke + 0.0004, 0.0])).nach(&Trafo::dreh_x(std::f32::consts::FRAC_PI_2));
    n.flaeche(&oben, [0.0735, 0.0315], Steuergeraet, [0.0, 0.0, 1.0, 1.0]);
    // Kabel von hinten zur linken Buchse, auf dem Tisch durchhängend
    let a = c.punkt([0.0, dicke * 0.5, 0.03]);
    let b = dreh.punkt([-0.03, 0.02, -0.122]);
    let punkte: Vec<V3> = (0..=20)
        .map(|i| {
            let s = i as f32 / 20.0;
            let x = a[0] + (b[0] - a[0]) * s + 0.05 * (s * std::f32::consts::PI * 1.5).sin();
            let z = a[2] + (b[2] - a[2]) * s - 0.04 * (s * std::f32::consts::PI).sin();
            let y = TISCH_HOEHE + 0.003 + (a[1] - TISCH_HOEHE) * (1.0 - s).powi(3) + (b[1] - TISCH_HOEHE) * s.powi(4);
            [x, y, z]
        })
        .collect();
    n.rohr(&punkte, 0.0026, 6, KunststoffGrau);
}

fn kleinkram(n: &mut Netz) {
    // Videokassetten, gestapelt, leicht verdreht; Rücken zeigen zur Kamera
    for i in 0..3 {
        let t = Trafo::bei([0.52, TISCH_HOEHE + 0.0125 + i as f32 * 0.025, -0.08]).nach(&Trafo::dreh_y(0.08 * (i as f32 - 1.0) + 0.1));
        n.quader(&t, [0.095, 0.012, 0.0935], 0.002, KunststoffSchwarz, 6.0);
        // Rücken mit Etikett (Atlas: vier Etiketten nebeneinander)
        n.flaeche(&t.nach(&Trafo::bei([0.0, 0.0, -0.0937])), [0.09, 0.0115], Kassette, [i as f32 * 0.25, 0.0, 0.25, 1.0]);
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
    // Zeitschrift
    let z = Trafo::bei([-0.62, TISCH_HOEHE + 0.003, -0.32]).nach(&Trafo::dreh_y(0.25)).nach(&Trafo::dreh_x(std::f32::consts::FRAC_PI_2));
    n.flaeche(&z, [0.105, 0.14], Zeitschrift, [0.0, 0.0, 1.0, 1.0]);
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
