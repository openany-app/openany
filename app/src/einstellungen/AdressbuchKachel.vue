<script setup>
/*
 * Adressbuch ein- und ausgeben (vCard) -- seit Oktober 2026 hier und nicht
 * mehr in der Webapp: Das Adressbuch ist Ende-zu-Ende verschlüsselt
 * (docs/plan-adressbuch-e2e.md), der Server kann es weder lesen noch
 * schreiben. Lesen, Schreiben und Zusammenführen: @oberflaeche/shared/vcard.
 *
 * WARUM IN DEN EINSTELLUNGEN UND NICHT IM ADRESSBUCH (wie vorher in der
 * Webapp): Beides macht man selten und mit Bedacht. Ein Knopf über der Liste
 * stünde jeden Tag im Weg und würde einmal im Jahr gebraucht.
 */
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { BookUser, Download, Upload, Loader2 } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { downloadBlob } from '@oberflaeche/shared/download';
import { vcardLesen, vcardSchreiben, zusammenfuehren } from '@oberflaeche/shared/vcard';
import { lokaleAdressbuchQuelle } from '../quellen/adressbuch';

const { t } = useI18n();
const toast = useToast();
const quelle = lokaleAdressbuchQuelle();

const dateifeld = ref(null);
const speichert = ref(false);
const liestEin = ref(false);
const bericht = ref(null);

const ohneVorspann = (url) => (url?.startsWith('data:') ? url.slice(url.indexOf(',') + 1) : null);

async function speichern() {
    speichert.value = true;
    try {
        const { data } = await quelle.getContacts();
        const text = vcardSchreiben(data.map((k) => ({ ...k, foto: ohneVorspann(k.photo?.url) })));
        await downloadBlob(new Blob([text], { type: 'text/vcard' }), 'adressbuch.vcf');
    } catch {
        toast.error(t('app.adressbuch.exportFailed'));
    } finally {
        speichert.value = false;
    }
}

/** Ein Foto aus der Datei -- verkleinert wie eines aus der Kamera. Ein unlesbares reisst den Kontakt nicht mit. */
async function fotoSetzen(id, b64) {
    if (!b64) return;
    try {
        const bytes = Uint8Array.from(atob(b64), (z) => z.charCodeAt(0));
        await quelle.uploadContactPhoto(id, new Blob([bytes]));
    } catch {
        // Name und Wege stehen schon.
    }
}

async function gewaehlt(ereignis) {
    const datei = ereignis.target.files?.[0];
    if (!datei) return;
    liestEin.value = true;
    bericht.value = null;
    try {
        const gelesen = vcardLesen(await datei.text());
        const { data: vorhanden } = await quelle.getContacts();
        const { neu, ergaenzt } = zusammenfuehren(vorhanden, gelesen);
        for (const k of neu) {
            const { data } = await quelle.createContact({ id: k.id, display_name: k.display_name, channels: k.channels });
            await fotoSetzen(data.id, k.foto);
        }
        for (const k of ergaenzt) {
            await quelle.updateContact(k.id, { display_name: k.display_name, channels: k.channels });
            await fotoSetzen(k.id, k.foto);
        }
        bericht.value = { gelesen: gelesen.length, neu: neu.length, ergaenzt: ergaenzt.length };
        toast.success(t('app.adressbuch.importDone'));
    } catch {
        toast.error(t('app.adressbuch.importFailed'));
    } finally {
        liestEin.value = false;
        // Sonst löst dieselbe Datei kein zweites `change` aus.
        ereignis.target.value = '';
    }
}
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <BookUser class="w-5 h-5 text-marke" /> {{ t('app.adressbuch.title') }}
    </h3>

    <p class="text-sm text-leise">{{ t('app.adressbuch.intro') }}</p>

    <div class="flex flex-wrap gap-2">
      <BaseButton type="button" groesse="normal" :disabled="speichert" @click="speichern">
        <Loader2 v-if="speichert" class="w-4 h-4 animate-spin" />
        <Download v-else class="w-4 h-4" />
        {{ t('app.adressbuch.export') }}
      </BaseButton>

      <button type="button" :disabled="liestEin" @click="dateifeld.click()"
        class="flex items-center gap-2 px-4 py-2 rounded-xl border border-linie hover:bg-auflage disabled:opacity-50 text-fliess text-sm font-bold transition-colors cursor-pointer">
        <Loader2 v-if="liestEin" class="w-4 h-4 animate-spin" />
        <Upload v-else class="w-4 h-4" />
        {{ t('app.adressbuch.import') }}
      </button>

      <input ref="dateifeld" type="file" accept=".vcf,text/vcard,text/x-vcard,text/directory" class="hidden" @change="gewaehlt" />
    </div>

    <p v-if="bericht" class="text-sm text-fliess">{{ t('app.adressbuch.report', bericht) }}</p>

    <p class="text-xs text-leise">{{ t('app.adressbuch.mergeHint') }}</p>
    <p class="text-xs text-amber-600 dark:text-amber-500">{{ t('app.adressbuch.lossHint') }}</p>
    <p class="text-xs text-leise">{{ t('app.adressbuch.verschluesselt') }}</p>
  </div>
</template>
