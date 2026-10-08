<script setup>
/*
 * Die Kachel „Matrix" -- Anmelden und Trennen des Matrix-Kontos dieses Geräts.
 *
 * BIS ZUM 29.09.2026 STAND DAS IN DEN NACHRICHTEN (Schlitz `oben`). Tiffy
 * wollte es neben den E-Mail-Postfächern haben: Verbinden ist Einrichtung,
 * der Verlauf ist zum Lesen und Schreiben.
 *
 * MEHRERE KONTEN, EIN VERLAUF. Das erste ist das Standard-Konto für neue
 * Nachrichten; eine Antwort geht von dem, bei dem die Nachricht ankam.
 *
 * DER UNTERSCHIED ZUR WEBAPP bleibt hervorgehoben: Hier ist dieses Gerät
 * selbst das Matrix-Gerät, mit eigenen Schlüsseln; openany.de liest nicht
 * mit, und der Verlauf reist nicht mit dem Abgleich.
 */
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { MessageSquare, Loader2, AlertTriangle, Unplug, ShieldCheck, Plus, Star } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { nachrichtenQuelle as quelle } from '../quellen/nachrichten';

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();

const lage = ref(null);
const leereForm = () => ({ homeserver: 'https://matrix.org', benutzer: '', passwort: '' });
const form = ref(leereForm());
const verbindet = ref(false);
const anmeldefehler = ref('');
const hinzufuegen = ref(false);
// Welches Konto gerade getrennt wird.
const trennt = ref(null);

async function lesen() {
    try {
        lage.value = await quelle.lage();
    } catch (e) {
        lage.value = { konten: [], mxid: null, fehler: String(e), ungelesen: 0 };
    }
}

async function verbinden() {
    verbindet.value = true;
    anmeldefehler.value = '';
    try {
        const f = form.value;
        lage.value = await quelle.anmelden(f.homeserver, f.benutzer, f.passwort);
        // Sofort vergessen — das Passwort soll nicht im Speicher der Seite
        // stehen bleiben.
        form.value = leereForm();
        hinzufuegen.value = false;
        toast.success(t('settings.matrix.connected'));
    } catch (e) {
        anmeldefehler.value = String(e);
    } finally {
        verbindet.value = false;
    }
}

async function trennen(mxid) {
    if (!(await confirmDelete(t('app.matrix.trennenFrage', { mxid })))) return;
    trennt.value = mxid;
    try {
        const { geraet_abgemeldet } = await quelle.abmelden(mxid);
        if (geraet_abgemeldet) toast.success(t('settings.matrix.disconnected'));
        else toast.info(t('settings.matrix.disconnectedButDeviceRemains'));
    } catch (e) {
        toast.error(String(e));
    } finally {
        trennt.value = null;
    }
    await lesen();
}

async function standard(mxid) {
    try { lage.value = await quelle.standard(mxid); } catch (e) { toast.error(String(e)); }
}

function abbrechen() {
    hinzufuegen.value = false;
    anmeldefehler.value = '';
    form.value = leereForm();
}

onMounted(() => {
    lesen();
    window.addEventListener('openany-nachrichten', lesen);
});
onUnmounted(() => window.removeEventListener('openany-nachrichten', lesen));

const feld = 'mt-1 w-full px-3 py-2 rounded-xl border border-linie bg-vertieft text-schrift text-sm';
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <MessageSquare class="w-5 h-5 text-marke" /> {{ t('app.matrix.titel') }}
    </h3>

    <div v-if="!lage" class="flex justify-center py-4"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>

    <template v-else>
      <ul v-if="lage.konten.length" class="space-y-3">
        <li v-for="(k, i) in lage.konten" :key="k.mxid" class="rounded-xl border border-linie p-3 space-y-2">
          <div class="flex flex-wrap items-center gap-2 min-w-0">
            <strong class="text-schrift break-all">{{ k.mxid }}</strong>
            <span v-if="i === 0 && lage.konten.length > 1"
                  class="px-2 py-0.5 rounded-full font-extrabold text-[10px] shrink-0 bg-marke-leise text-marke">{{ t('app.allgemein.standard') }}</span>
            <span v-if="k.laeuft && !k.fehler" class="text-sm text-leise">{{ t('app.matrix.verbunden') }}</span>
          </div>
          <!-- `min-w-0` UND `break-words`: Ein Fehler vom Homeserver trägt schon
               mal eine Adresse ohne ein einziges Leerzeichen mit sich. -->
          <p v-if="k.fehler" class="text-sm text-warnung flex items-start gap-2">
            <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span class="min-w-0 break-words">{{ k.fehler }}</span>
          </p>
          <div class="flex flex-wrap gap-2">
            <BaseButton v-if="i > 0" variant="secondary" :disabled="trennt !== null" @click="standard(k.mxid)">
              <Star class="w-4 h-4" /> {{ t('app.allgemein.alsStandard') }}
            </BaseButton>
            <BaseButton variant="danger" :disabled="trennt !== null" @click="trennen(k.mxid)">
              <Loader2 v-if="trennt === k.mxid" class="w-4 h-4 animate-spin" /><Unplug v-else class="w-4 h-4" /> {{ t('settings.matrix.disconnect') }}
            </BaseButton>
          </div>
        </li>
      </ul>
      <template v-if="lage.konten.length">
        <p v-if="lage.konten.length > 1" class="text-sm text-leise">{{ t('app.matrix.mehrere') }}</p>
        <!-- Ehrlich über den Stand: Ohne Emoji-Vergleich zeigt Element dieses
             Gerät als „nicht verifiziert". Verschlüsselt ist trotzdem. -->
        <p class="text-xs text-leise">{{ t('app.matrix.keinEmoji') }}</p>
        <BaseButton v-if="!hinzufuegen" variant="secondary" @click="hinzufuegen = true">
          <Plus class="w-4 h-4" /> {{ t('app.matrix.hinzufuegen') }}
        </BaseButton>
      </template>

      <template v-if="!lage.konten.length || hinzufuegen">
      <p class="text-sm text-leise">{{ t('settings.matrix.intro') }}</p>
      <p class="text-sm text-fliess rounded-xl border border-linie bg-auflage p-3 flex items-start gap-2">
        <ShieldCheck class="w-4 h-4 shrink-0 mt-0.5 text-marke" />
        <span>{{ t('app.matrix.geraetSelbst') }}</span>
      </p>

      <form class="space-y-3" @submit.prevent="verbinden">
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('settings.matrix.homeserver') }}</span>
          <input v-model="form.homeserver" type="url" required autocorrect="off" autocapitalize="off" :class="feld" />
        </label>
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('settings.matrix.user') }}</span>
          <input v-model="form.benutzer" type="text" required autocomplete="username" autocorrect="off" autocapitalize="off"
                 :placeholder="t('settings.matrix.userPlaceholder')" :class="feld" />
        </label>
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('settings.matrix.password') }}</span>
          <input v-model="form.passwort" type="password" required autocomplete="current-password" :class="feld" />
          <span class="mt-1 block text-xs text-leise">{{ t('settings.matrix.passwordHint') }}</span>
        </label>

        <p v-if="anmeldefehler" class="text-sm text-warnung flex items-start gap-2">
          <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
          <span class="min-w-0 break-words">{{ anmeldefehler }}</span>
        </p>

        <div class="flex flex-wrap gap-2">
          <BaseButton type="submit" :disabled="verbindet">
            <Loader2 v-if="verbindet" class="w-4 h-4 animate-spin" />
            <MessageSquare v-else class="w-4 h-4" />
            {{ t('settings.matrix.connect') }}
          </BaseButton>
          <BaseButton v-if="hinzufuegen" variant="secondary" :disabled="verbindet" @click="abbrechen">{{ t('common.cancel') }}</BaseButton>
        </div>
      </form>
      </template>
    </template>
  </div>
</template>
