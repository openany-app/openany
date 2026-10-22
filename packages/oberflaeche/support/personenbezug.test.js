import { describe, it, expect } from 'vitest';
import { personenbezug } from './personenbezug';

const arten = (text) => personenbezug(text).map((f) => f.art);

describe('personenbezug', () => {
    it('findet nichts in einer gewöhnlichen Anfrage', () => {
        expect(personenbezug('Seit dem Update vom 01.02.2026 hängt der Abgleich bei 3 von 12 Dateien. Version 1.0.3, Android 14.')).toEqual([]);
        expect(personenbezug('Ich bin verwirrt: Wo finde ich die Einstellungen? Ich bin Lehrer und nutze Notizen.')).toEqual([]);
        expect(personenbezug('')).toEqual([]);
    });

    it('findet E-Mail-Adressen', () => {
        expect(arten('Schreib mir an beispiel@beispiel.test bitte')).toEqual(['email']);
    });

    it('findet Telefonnummern, aber kein Datum und keine Version', () => {
        expect(arten('Ruf an: +49 30 1234567')).toEqual(['telefon']);
        expect(arten('Tel. 0170/1234567')).toEqual(['telefon']);
        expect(arten('0049 (30) 123 45 67')).toEqual(['telefon']);
        expect(arten('am 01.02.2026 um 10:30')).toEqual([]);
        expect(arten('Fehler 0x80070005 in 1.0.3')).toEqual([]);
    });

    it('findet IP-Adressen', () => {
        expect(arten('mein Router hat 192.168.1.20')).toEqual(['ip']);
        expect(arten('IPv6 2001:db8:85a3::8a2e:370:7334 geht nicht')).toEqual(['ip']);
        expect(arten('Version 1.2.3.4000')).toEqual([]);
    });

    it('findet Namen nach einer Vorstellung, in allen Sprachen', () => {
        expect(arten('Hallo, mein Name ist Erika Beispiel und ich habe ein Problem.')).toEqual(['name']);
        expect(arten('Ich heiße Max.')).toEqual(['name']);
        expect(arten('Hi, my name is Jane Example.')).toEqual(['name']);
        expect(arten("Bonjour, je m'appelle Camille.")).toEqual(['name']);
        expect(arten('Hola, me llamo Lucía.')).toEqual(['name']);
        expect(arten('Olá, chamo-me João.')).toEqual(['name']);
        expect(arten('Dzień dobry, nazywam się Anna.')).toEqual(['name']);
        expect(arten('Привіт, мене звати Олена.')).toEqual(['name']);
        expect(arten('Name: Erika Beispiel')).toEqual(['name']);
    });

    it('findet eine Unterschrift unter einem Gruß', () => {
        expect(arten('Danke für die Hilfe!\n\nViele Grüße\nErika Beispiel')).toEqual(['name']);
        expect(arten('Thanks a lot.\nBest regards,\nJane')).toEqual(['name']);
        expect(arten('Pozdrawiam, Anna')).toEqual(['name']);
        expect(arten('LG Max')).toEqual(['name']);
    });

    it('lässt einen Gruß ohne Namen durch', () => {
        expect(arten('Viele Grüße')).toEqual([]);
        expect(arten('Danke!\nViele Grüße und einen schönen Tag')).toEqual([]);
    });

    it('nennt die Stelle, damit die Oberfläche sie zeigen kann', () => {
        const text = 'Mail: beispiel@beispiel.test';
        const [f] = personenbezug(text);
        expect(text.slice(f.von, f.bis)).toBe('beispiel@beispiel.test');
        expect(f.treffer).toBe('beispiel@beispiel.test');
    });

    it('findet mehrere Dinge in Lesereihenfolge', () => {
        expect(arten('Ich heiße Max, Tel. +49 170 1234567, max@beispiel.test')).toEqual(['name', 'telefon', 'email']);
    });
});

describe('personenbezug – Uhrzeiten', () => {
    it('hält eine Uhrzeit nicht für eine IPv6-Adresse', () => {
        expect(arten('Um 10:30:15 kam der Fehler')).toEqual([]);
    });
});
